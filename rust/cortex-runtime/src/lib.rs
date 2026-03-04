use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::{info, warn};

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

        // 1. Resource Availability Check (Cortex 2.0 Guard)
        // Adjust default requirement for examples (1GB) vs defaults (50GB)
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

        // 1. Detect hardware
        let profile = cortex_core::hardware::HardwareProfile::detect();
        info!("Hardware detected: {:?}", profile);

        // 2. Setup KV Cache
        let cache_limit_gb = std::env::var("CORTEX_KV_CACHE_GB")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(4);

        let _kv_manager = kv_cache::KVCacheManager::new(cache_limit_gb);
        info!("KV Cache Manager initialized ({}GB limit)", cache_limit_gb);

        // 3. Extract Bundle to Temporary Directory
        let manifest = inspect::InspectEngine::get_manifest(bundle_path)?;
        if manifest.agents.is_empty() {
            info!("No agents found in bundle to execute.");
            return Ok(());
        }

        let temp_dir = tempfile::tempdir()?;
        info!("Unpacking bundle to temporary execution environment at {:?} ...", temp_dir.path());

        let bundle_data = crypto::EncryptionEngine::read_bundle(bundle_path)?;
        let decoder = zstd::stream::read::Decoder::new(std::io::Cursor::new(bundle_data))?;
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(temp_dir.path())?;

        // DEBUG: List files in temp_dir
        if let Ok(entries) = std::fs::read_dir(temp_dir.path()) {
            for entry in entries.flatten() {
                info!("Extracted bundle file: {:?}", entry.file_name());
            }
        }

        // 4. Setup Networking if requested
    let mut macvlan_iface = None;
    if manifest.package.allow_network {
        match network::NetworkManager::detect_default_interface() {
            Ok(parent) => {
                let ifname = format!("mc_{}", &session_id[..8]);
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

    // 5. Setup Dependencies if needed
        let req_path = temp_dir.path().join("requirements.txt");
        let mut python_cmd = "python3".to_string(); // Default to python3

        if req_path.exists() {
            info!("Found requirements.txt, setting up isolated Python environment...");
            let venv_path = temp_dir.path().join(".venv");
            info!("Venv target path: {:?}", venv_path);

            // Use python3 if available, otherwise python
            let base_python = if cfg!(windows) { "python" } else { "python3" };
            info!("Using base python: {}", base_python);

            let status = std::process::Command::new(base_python)
                .args(["-m", "venv", ".venv"]) // Just use relative path since we set current_dir
                .current_dir(temp_dir.path())
                .status()?;

            info!("Venv creation command finished with status: {:?}", status);

            if !status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to create Python virtual environment: {:?}", status
                ));
            }

            let pip_cmd = if cfg!(windows) {
                venv_path.join("Scripts").join("pip")
            } else {
                venv_path.join("bin").join("pip")
            };
            info!("Pip command path: {:?}", pip_cmd);

            info!("Installing bundle dependencies...");
            let pip_status = std::process::Command::new(&pip_cmd)
                .args(["install", "-r", "requirements.txt"])
                .current_dir(temp_dir.path())
                .status()?;

            info!("Pip installation finished with status: {:?}", pip_status);

            if !pip_status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to install dependencies from requirements.txt"
                ));
            }

            python_cmd = if cfg!(windows) {
                venv_path
                    .join("Scripts")
                    .join("python")
                    .to_str()
                    .unwrap()
                    .to_string()
            } else {
                venv_path
                    .join("bin")
                    .join("python")
                    .to_str()
                    .unwrap()
                    .to_string()
            };

            info!(
                "Dependencies installed. Using isolated python at: {}",
                python_cmd
            );
        }

        let mut common_env = std::collections::HashMap::new();
        let pypath = if cfg!(windows) {
            format!(
                "{};{}",
                temp_dir.path().display(),
                temp_dir.path().join("src").display()
            )
        } else {
            format!(
                "{}:{}",
                temp_dir.path().display(),
                temp_dir.path().join("src").display()
            )
        };
        common_env.insert("PYTHONPATH".to_string(), pypath);

        // Atomic Secret Redaction (Phase 4)
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

        // 5. Initialize Models (Ollama sidecar and automatic pulling)
        Self::setup_models(temp_dir.path()).await?;

        // 6. Initialize Executors based on Mode
        if is_turbo {
            info!(
                "Spawning {} agents in Parallel Turbo Mode...",
                manifest.agents.len()
            );
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
                    cwd: temp_dir.path().to_path_buf(),
                    env: common_env.clone(),
                    timeout_secs: 600,
                    allow_network: agent.allow_network,
                    session_id: session_id.clone(),
                    macvlan_iface: macvlan_iface.clone(),
                    allowed_ips: agent.allowed_ips.clone(),
                    secret_fds: secret_fds.clone(),
                });
            }

            let (results, metrics) = parallel_executor.execute(tasks).await?;
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
                cwd: temp_dir.path().to_path_buf(),
                env: common_env.clone(),
                timeout_secs: 600,
                allow_network: primary_agent.allow_network,
                session_id: session_id.clone(),
                macvlan_iface: macvlan_iface.clone(),
                allowed_ips: primary_agent.allowed_ips.clone(),
                secret_fds: secret_fds.clone(),
            };

            let (results, metrics) = parallel_executor.execute(vec![task]).await?;
            info!("=== Agent Execution Output ===\n{}", results[0]);
            info!("Final Execution Metrics: {:?}", metrics);
        }

        // Temp dir is automatically cleaned up when dropped
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

    /// Initialize the Cortex runtime (downloads common packages)
    pub async fn init_env() -> Result<()> {
        info!("Initializing Cortex 2.0 Base Environment...");

        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let stdlib_dir = home_dir.join(".cortex").join("stdlib");

        info!("Setting up Cortex standard library at {:?}", stdlib_dir);
        std::fs::create_dir_all(&stdlib_dir)?;

        let venv_path = stdlib_dir.join(".venv");

        if !venv_path.exists() {
            info!("Creating central virtual environment...");
            std::process::Command::new("python3")
                .args(["-m", "venv", venv_path.to_str().unwrap()])
                .status()?;
        }

        let pip_cmd = if cfg!(windows) {
            venv_path.join("Scripts").join("pip")
        } else {
            venv_path.join("bin").join("pip")
        };

        info!("Downloading and pre-warming common AI packages...");

        let common_packages = vec![
            "crewai",
            "crewai-tools",
            "litellm",
            "pydantic",
            "requests",
            "beautifulsoup4",
            "duckduckgo-search",
        ];

        let status = std::process::Command::new(&pip_cmd)
            .arg("install")
            .args(&common_packages)
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to pre-warm packages."));
        }

        // Ensure python base deps
        let py_executor = executor::PythonExecutor::new(PathBuf::from("."));
        py_executor.initialize_env()?;

        info!(
            "Cortex environment is primed and ready. Common packages cached in {:?}",
            venv_path
        );
        Ok(())
    }
}
