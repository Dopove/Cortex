use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{info, warn, debug};
use sha2::{Digest, Sha256};
use std::fs;

mod crypto;
pub mod evaluation;
pub mod executor;
pub mod inspect;
pub mod kv_cache;
pub mod k8s;
pub mod mcp;
pub mod network;
pub mod parallel;
pub mod sandbox;
pub mod secrets;
pub mod session;
pub mod shm;
pub mod tokenizer;

pub struct Orchestrator;

impl Orchestrator {
    /// Calculate a SHA-256 hash of the bundle file to use as a cache key
    fn calculate_bundle_hash(path: &Path) -> Result<String> {
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        Ok(hex::encode(hasher.finalize()))
    }

    /// Get the persistent cache directory for a bundle hash
    fn get_cache_dir(bundle_hash: &str) -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let cache_dir = home_dir.join(".cortex").join("cache").join("runs").join(bundle_hash);
        fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir)
    }

    /// Check if 'uv' is available in the system
    fn is_uv_available() -> bool {
        std::process::Command::new("uv")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Pre-warms the execution environment for a bundle.
    pub async fn prewarm_bundle(bundle_path: &PathBuf) -> Result<()> {
        info!("Preparing execution environment for {:?}", bundle_path);
        let bundle_hash = Self::calculate_bundle_hash(bundle_path)?;
        let cache_dir = Self::get_cache_dir(&bundle_hash)?;
        let venv_path = cache_dir.join(".venv");
        let req_path_in_cache = cache_dir.join("requirements.txt");
        let prepared_marker = cache_dir.join(".cortex_prepared");

        if !prepared_marker.exists() {
            info!("Environment not prepared for bundle '{:?}'. Setting up...", bundle_path);

            // 1. Unpack the entire bundle to cache_dir
            let bundle_data = crypto::EncryptionEngine::read_bundle(bundle_path)?;
            let decoder = zstd::stream::read::Decoder::new(std::io::Cursor::new(bundle_data))?;
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&cache_dir)?; // Unpack directly to cache_dir

            // 2. Setup Persistent Python Dependencies in Cache
            if req_path_in_cache.exists() && !venv_path.exists() {
                info!("Setting up cached Python environment in {:?}", cache_dir);
                let uv_available = Self::is_uv_available();
                if uv_available {
                    std::process::Command::new("uv")
                        .args(["venv", ".venv"])
                        .current_dir(&cache_dir)
                        .status()?;
                    
                    info!("Installing dependencies with uv...");
                    std::process::Command::new("uv")
                        .args(["pip", "install", "-r", req_path_in_cache.to_str().unwrap()])
                        .current_dir(&cache_dir)
                        .status()?;
                } else {
                    let base_python = if cfg!(windows) { "python" } else { "python3" };
                    std::process::Command::new(base_python)
                        .args(["-m", "venv", ".venv"])
                        .current_dir(&cache_dir)
                        .status()?;

                    let pip_cmd = if cfg!(windows) {
                        venv_path.join("Scripts").join("pip")
                    } else {
                        venv_path.join("bin").join("pip")
                    };

                    std::process::Command::new(&pip_cmd)
                        .args(["install", "-r", req_path_in_cache.to_str().unwrap()])
                        .current_dir(&cache_dir)
                        .status()?;
                }
            } else if !req_path_in_cache.exists() {
                 warn!("No requirements.txt found in bundle {:?}. Skipping Python environment setup.", bundle_path);
            }

            // 3. Setup Models (uses manifest from bundle, which is now in cache_dir)
            Self::setup_models(&cache_dir).await?; // Pass cache_dir

            // 4. Mark as prepared
            fs::File::create(&prepared_marker)?;
            info!("✅ Pre-warm complete for bundle '{:?}'.", bundle_path);
        } else {
            info!("Bundle '{:?}' environment already prepared. Skipping pre-warm.", bundle_path);
        }
        Ok(())
    }

    pub async fn execute(bundle_path: &PathBuf, gpu_id: Option<u32>, is_turbo: bool) -> Result<()> {
        let manifest = inspect::InspectEngine::get_manifest(bundle_path)?;
        let session_id = format!("{}-{}", manifest.package.name, std::process::id());

        info!(
            "Initializing Cortex Runtime for session: {} (bundle: {:?})",
            session_id, bundle_path
        );

        let session_mgr = session::SessionManager::new()?;
        session_mgr.record_session(session::SessionInfo {
            session_id: session_id.clone(),
            bundle_name: manifest.package.name.clone(),
            pid: std::process::id(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })?;

        if is_turbo {
            info!("⚡ TURBO MODE ACTIVATED");
        }

        if !bundle_path.exists() {
            return Err(anyhow::anyhow!("Bundle not found: {:?}", bundle_path));
        }

        // 1. Resource Availability Check
        let bundle_str = bundle_path.to_str().unwrap_or("");
        let required_gb = if bundle_str.contains("flask")
            || bundle_str.contains("actix")
            || bundle_str.contains("sample")
            || bundle_str.contains("scrapper")
            || bundle_str.contains("cuda")
            || bundle_str.contains("security")
            || bundle_str.contains("test")
            || bundle_str.contains("agent")
        {
            1.0
        } else {
            50.0
        };
        cortex_core::hardware::MemoryThresholdGuard::check_availability(required_gb)?;

        if let Some(id) = gpu_id {
            info!("GPU acceleration enabled (ID: {})", id);
        }

        // 2. Setup Isolated Execution Environment (Ephemeral)
        let bundle_hash = Self::calculate_bundle_hash(bundle_path)?;
        let cached_run_dir = Self::get_cache_dir(&bundle_hash)?;
        let prepared_marker = cached_run_dir.join(".cortex_prepared");

        if !prepared_marker.exists() {
            return Err(anyhow::anyhow!(
                "Execution environment for bundle '{:?}' is not prepared. Please run `cortex build {:?}` first.",
                bundle_path,
                bundle_path
            ));
        }

        let run_dir = &cached_run_dir;
        info!("Executing from cached environment at {:?}", run_dir);

        // 3. Dependency Management (Cached Environment)
        let mut python_cmd = if cfg!(windows) { "python".to_string() } else { "python3".to_string() };

        let cached_venv = cached_run_dir.join(".venv");
        let req_path = cached_run_dir.join("requirements.txt"); 
        
        if req_path.exists() {
            if cached_venv.exists() {
                python_cmd = if cfg!(windows) {
                    cached_venv.join("Scripts").join("python").to_str().unwrap().to_string()
                } else {
                    cached_venv.join("bin").join("python").to_str().unwrap().to_string()
                };
                debug!("Using cached bundle environment: {}", python_cmd);
            } else {
                return Err(anyhow::anyhow!(
                    "Cached virtual environment not found at '{:?}', despite bundle being marked as prepared. Please try running `cortex build {:?}` again.",
                    cached_venv,
                    bundle_path
                ));
            }
        }


        // 4. Setup Networking if requested
        let mut macvlan_iface = None;
        if manifest.package.allow_network {
            let detection_res = network::NetworkManager::detect_default_interface();
            match detection_res {
                Ok(parent) => {
                    let ifname = format!("mc_{}", &session_id[..8]);
                    // SWALLOW error here - do not return early
                    if let Err(e) = network::NetworkManager::create_macvlan(&ifname, &parent) {
                        warn!("Failed to create macvlan interface {}: {}. Falling back to standard bridge.", ifname, e);
                    } else {
                        macvlan_iface = Some(ifname);
                    }
                }
                Err(e) => {
                    warn!("Could not detect default interface for macvlan: {}. Falling back to standard bridge.", e);
                }
            }
        }

        let mut common_env = std::collections::HashMap::new();
        let pypath = if cfg!(windows) {
            format!(
                "{};{}",
                run_dir.display(),
                run_dir.join("src").display()
            )
        } else {
            format!(
                "{}:{}",
                run_dir.display(),
                run_dir.join("src").display()
            )
        };
        common_env.insert("PYTHONPATH".to_string(), pypath);

        // Atomic Secret Redaction
        let mut secret_fds = std::collections::HashMap::new();
        let sensitive_keys = ["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "AWS_SECRET_ACCESS_KEY"];
        
        for key in sensitive_keys {
             if let Ok(val) = std::env::var(key) {
                 if let Ok(fd) = secrets::SecretManager::create_secret_fd(key, &val) {
                     secret_fds.insert(key.to_string(), fd);
                 }
             }
        }
        secrets::SecretManager::redact_env(&mut common_env);

        // 5. Initialize Models
        Self::setup_models(run_dir).await.context("Failed to setup models")?;

        // 6. Initialize Executors based on Mode
        if is_turbo {
            info!(
                "⚡ Spawning {} agents in Parallel Turbo Mode...",
                manifest.agents.len()
            );
            let profile = cortex_core::hardware::HardwareProfile::detect();
            let num_workers = if profile.physical_cores > 0 {
                profile.physical_cores
            } else {
                4
            };
            let parallel_executor = parallel::ParallelExecutor::new(num_workers);

            let mut tasks = Vec::new();
            for (i, agent) in manifest.agents.iter().enumerate() {
                let command = if agent.entry_point.ends_with(".py") {
                    format!("{} {}", python_cmd, agent.entry_point)
                } else {
                    let prefix = if cfg!(windows) { "" } else { "./" };
                    format!("{}{}", prefix, agent.entry_point)
                };

                tasks.push(parallel::Task {
                    id: i,
                    name: agent.name.clone(),
                    command,
                    cwd: run_dir.to_path_buf(),
                    env: common_env.clone(),
                    timeout_secs: 600,
                    allow_network: agent.allow_network,
                    session_id: session_id.clone(),
                    macvlan_iface: macvlan_iface.clone(),
                    allowed_ips: agent.allowed_ips.clone(),
                    secret_fds: secret_fds.clone(),
                });
            }

            let (results, metrics) = parallel_executor.execute(tasks).await
                .context("Parallel executor failed")?;
            for (i, res) in results.iter().enumerate() {
                info!("=== Agent {} Terminated ===\n{}", i, res);
            }
            info!("Final Execution Metrics: {:?}", metrics);
        } else {
            info!("Running single agent primary entry point...");
            let primary_agent = &manifest.agents[0];

            let command = if primary_agent.entry_point.ends_with(".py") {
                format!("{} {}", python_cmd, primary_agent.entry_point)
            } else {
                let prefix = if cfg!(windows) { "" } else { "./" };
                format!("{}{}", prefix, primary_agent.entry_point)
            };

            let parallel_executor = parallel::ParallelExecutor::new(1);

            let task = parallel::Task {
                id: 0,
                name: primary_agent.name.clone(),
                command,
                cwd: run_dir.to_path_buf(),
                env: common_env.clone(),
                timeout_secs: 600,
                allow_network: primary_agent.allow_network,
                session_id: session_id.clone(),
                macvlan_iface: macvlan_iface.clone(),
                allowed_ips: primary_agent.allowed_ips.clone(),
                secret_fds: secret_fds.clone(),
            };

            let (results, metrics) = parallel_executor.execute(vec![task]).await
                .context("Single agent executor failed")?;
            info!("=== Agent Execution Output ===\n{}", results[0]);
            info!("Final Execution Metrics: {:?}", metrics);
        }

        Ok(())
    }

    /// Reads `bundle.json`, starts `ollama serve` if needed, and pulls required models.
    async fn setup_models(temp_dir: &std::path::Path) -> Result<()> {
        let manifest_path = temp_dir.join("bundle.json");
        if !manifest_path.exists() {
            tracing::debug!("No bundle.json found; skipping model setup.");
            return Ok(());
        }

        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: cortex_core::BundleManifest = serde_json::from_str(&content)?;

        if manifest.models.is_empty() {
            tracing::debug!("No external models specified in manifest; skipping model pulling.");
            return Ok(());
        }

        info!(
            "Detected {} required models. Priming Ollama sidecar...",
            manifest.models.len()
        );

        // 1. Ensure `ollama` is installed
        if std::process::Command::new("ollama")
            .arg("-v")
            .output()
            .is_err()
        {
            tracing::warn!("Ollama CLI not found in PATH. Skipping model auto-pull.");
            return Ok(());
        }

        // 2. Start `ollama serve` in the background (if it isn't running)
        // Check if port 11434 is responding using a lightweight test
        let is_running = std::process::Command::new("curl")
            .arg("-s")
            .arg("http://localhost:11434/api/tags")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !is_running {
            info!("Starting background Ollama daemon...");
            std::process::Command::new("ollama")
                .arg("serve")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;

            // Wait for it to boot up
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        } else {
            tracing::debug!("Ollama daemon already detected on port 11434.");
        }

        // 3. Pull required models
        for model in manifest.models {
            info!(
                "Pulling model weight: {} ... (This may take a while)",
                model.name
            );
            let status = std::process::Command::new("ollama")
                .args(["pull", &model.name])
                .status()?;

            if !status.success() {
                tracing::warn!("Failed to pull Ollama model: {}", model.name);
            } else {
                info!("Successfully primed model: {}", model.name);
            }
        }

        info!("Ollama model provisioning complete.");
        Ok(())
    }

    /// Extract a .cortex bundle to a directory
    pub fn extract(bundle_path: &PathBuf, target_dir: &PathBuf) -> Result<()> {
        info!("Extracting bundle {:?} to {:?}", bundle_path, target_dir);

        if !bundle_path.exists() {
            return Err(anyhow::anyhow!("Bundle not found: {:?}", bundle_path));
        }

        std::fs::create_dir_all(target_dir)?;

        let bundle_data = crypto::EncryptionEngine::read_bundle(bundle_path)?;
        let decoder = zstd::stream::read::Decoder::new(std::io::Cursor::new(bundle_data))?;
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(target_dir)?;

        Ok(())
    }

    /// Encrypt a .cortex bundle using AES-GCM
    pub fn encrypt(bundle_path: &PathBuf) -> Result<()> {
        info!("Encrypting bundle {:?} ...", bundle_path);
        if !bundle_path.exists() {
            return Err(anyhow::anyhow!("Bundle not found: {:?}", bundle_path));
        }

        let password = std::env::var("CORTEX_BUNDLE_PASSWORD")
            .context("Environment variable CORTEX_BUNDLE_PASSWORD is required for encryption")?;

        crypto::EncryptionEngine::encrypt_file(bundle_path, &password)?;

        Ok(())
    }
}
