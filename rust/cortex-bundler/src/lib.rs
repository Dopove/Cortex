use anyhow::{anyhow, Context, Result};
use cortex_core::BundleManifest;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Builder;
use tracing::{debug, info};
use zstd::stream::write::Encoder;

pub struct Bundler {
    project_dir: PathBuf,
    output_path: PathBuf,
}

impl Bundler {
    pub fn new(project_dir: PathBuf, output_path: PathBuf) -> Self {
        Self {
            project_dir,
            output_path,
        }
    }

    /// Bundles the directory structure into a compressed `.cortex` archive.
    pub async fn run_bundle_pipeline(&self) -> Result<()> {
        info!("Starting Universal Bundler Pipeline...");

        if !self.project_dir.exists() {
            return Err(anyhow!(
                "Project directory does not exist: {:?}",
                self.project_dir
            ));
        }

        let manifest = self.validate_project().await?;

        info!("Phase 1: Validated. Beginning Archiving & Compression (TAR+ZSTD)...");

        // Spawn blocking for CPU-intensive compression task
        let p_dir = self.project_dir.clone();
        let o_path = self.output_path.clone();

        tokio::task::spawn_blocking(move || Bundler::create_archive(&p_dir, &o_path, manifest))
            .await??;

        info!("✅ Bundle successfully created at: {:?}", self.output_path);

        Ok(())
    }

    async fn validate_project(&self) -> Result<BundleManifest> {
        let mut agents = Vec::new();
        let mut models = Vec::new();

        // 0. Check for existing cortex_manifest.json (Legacy Support)
        let legacy_manifest_path = self.project_dir.join("cortex_manifest.json");
        let mut discovered_entry = "main.py".to_string();

        if legacy_manifest_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&legacy_manifest_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(entry) = json.get("entry_point").and_then(|v| v.as_str()) {
                        discovered_entry = entry.to_string();
                    }
                }
            }
        }

        // 1. Scan for Agents
        let entries = [
            "main.py",
            "microblog.py",
            "app.py",
            "wsgi.py",
            "src/main.py",
            "src/scrapper/main.py",
            &discovered_entry,
        ];
        for entry_name in entries {
            let entry_path = self.project_dir.join(entry_name);
            if entry_path.exists() {
                let checksum = Some(Self::calculate_checksum(&entry_path)?);
                agents.push(cortex_core::AgentInfo {
                    name: format!("agent_{}", entry_name.replace(".py", "").replace("/", "_")),
                    entry_point: entry_name.to_string(),
                    checksum,
                });
                break; // Use the first one found
            }
        }

        // 2. Scan for Models
        // A: Check `models/` directory for explicitly provided .gguf files
        let model_dir = self.project_dir.join("models");
        if model_dir.exists() {
            for entry in std::fs::read_dir(model_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let checksum = Some(Self::calculate_checksum(&path)?);
                    models.push(cortex_core::ModelInfo {
                        name: path.file_name().unwrap().to_string_lossy().into_owned(),
                        path: format!("models/{}", path.file_name().unwrap().to_string_lossy()),
                        architecture: Some(cortex_core::ModelArchitecture::Bloom), // Assume Bloom for 2.0
                        quantization: Some("q4_k_m".to_string()),
                        vocab_size: None,
                        checksum,
                    });
                }
            }
        }

        // B: AST / Regex scan over Python files for Ollama/LiteLLM model requests
        let re = regex::Regex::new(r#"model\s*=\s*['"]ollama/(.*?)['"]"#)?;
        for entry in walkdir::WalkDir::new(&self.project_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("py") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for cap in re.captures_iter(&content) {
                        if let Some(model_name) = cap.get(1) {
                            let name = model_name.as_str().to_string();
                            info!("Detected model requirement in source: {}", name);
                            // Avoid duplicates
                            if !models.iter().any(|m| m.name == name) {
                                models.push(cortex_core::ModelInfo {
                                    name,
                                    path: "".to_string(), // Fetched dynamically during build injection later
                                    architecture: Some(cortex_core::ModelArchitecture::Llama),
                                    quantization: None,
                                    vocab_size: None,
                                    checksum: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        debug!(
            "Generating bundle manifest with {} agents and {} models...",
            agents.len(),
            models.len()
        );

        Ok(BundleManifest {
            package: cortex_core::PackageInfo {
                name: "cortex-bundle".to_string(),
                version: "2.0.0".to_string(),
                description: Some("Cortex 2.0 Hardened Bundle".to_string()),
            },
            agents,
            models,
        })
    }

    fn calculate_checksum(path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};
        use std::io::{BufReader, Read};

        let file = File::open(path).context("Failed to open file for checksum")?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let count = reader
                .read(&mut buffer)
                .context("Failed to read for checksum")?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    fn create_archive(src: &Path, dst: &Path, manifest: BundleManifest) -> Result<()> {
        let file = File::create(dst).context("Failed to create output bundle file")?;

        // Wrap the file in a Zstd Encoder. Compression level 3.
        let encoder = Encoder::new(file, 3).context("Failed to initialize ZSTD encoder")?;
        let mut tar_builder = Builder::new(encoder);

        debug!("Appending directory {:?} to archive...", src);
        tar_builder
            .append_dir_all(".", src)
            .context("Failed to append directory to tar")?;

        // Also append the manifest as bundle.json
        let manifest_json = serde_json::to_vec_pretty(&manifest)?;
        let mut header = tar::Header::new_gnu();
        header.set_size(manifest_json.len() as u64);
        header.set_path("bundle.json")?;
        header.set_cksum();
        tar_builder.append(&header, &manifest_json[..])?;

        let final_encoder = tar_builder
            .into_inner()
            .context("Failed to finalize tar builder")?;
        final_encoder
            .finish()
            .context("Failed to finish ZSTD compression")?;

        Ok(())
    }
}
