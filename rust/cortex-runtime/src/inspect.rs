use anyhow::{Context, Result};
use cortex_core::BundleManifest;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tar::Archive;
use tracing::info;
use zstd::stream::read::Decoder;

pub struct InspectEngine;

impl InspectEngine {
    /// Extracts the `bundle.json` manifest from a `.cortex` archive
    /// without inflating the entire bundle.
    pub fn get_manifest(bundle_path: &PathBuf) -> Result<BundleManifest> {
        let bundle_data = crate::crypto::EncryptionEngine::read_bundle(bundle_path)?;
        let decoder = Decoder::new(std::io::Cursor::new(bundle_data)).context("Failed to initialize ZSTD decoder")?;
        let mut archive = Archive::new(decoder);

        for entry in archive.entries()? {
            let mut entry = entry?;
            if entry.path()?.to_str() == Some("bundle.json") {
                let mut json_data = Vec::new();
                entry.read_to_end(&mut json_data)?;
                let manifest: BundleManifest = serde_json::from_slice(&json_data)?;
                return Ok(manifest);
            }
        }

        Err(anyhow::anyhow!(
            "Manifest `bundle.json` not found in archive"
        ))
    }

    /// Performs a structural and integrity check on the bundle.
    pub fn verify(bundle_path: &PathBuf) -> Result<()> {
        info!("Verifying structural integrity for: {:?}", bundle_path);

        let manifest = Self::get_manifest(bundle_path)?;
        let bundle_data = crate::crypto::EncryptionEngine::read_bundle(bundle_path)?;
        let decoder = Decoder::new(std::io::Cursor::new(bundle_data)).context("Invalid ZSTD header in bundle")?;
        let mut archive = Archive::new(decoder);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_str().unwrap_or("").to_string();

            // Check agents
            if let Some(agent) = manifest.agents.iter().find(|a| a.entry_point == path) {
                if let Some(expected) = &agent.checksum {
                    Self::validate_checksum(&mut entry, expected, &path)?;
                }
            }

            // Check models
            if let Some(model) = manifest.models.iter().find(|m| m.path == path) {
                if let Some(expected) = &model.checksum {
                    Self::validate_checksum(&mut entry, expected, &path)?;
                }
            }
        }

        info!("✅ Structural integrity verified (ZSTD/TAR streams and internal checksums OK).");

        Ok(())
    }

    fn validate_checksum<R: Read>(reader: &mut R, expected: &str, name: &str) -> Result<()> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let count = reader
                .read(&mut buffer)
                .context("Failed to read stream for verification")?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        let actual = hex::encode(hasher.finalize());
        if actual != expected {
            return Err(anyhow::anyhow!(
                "Checksum mismatch for {}: expected {}, found {}",
                name,
                expected,
                actual
            ));
        }

        info!("Checksum OK for {}", name);
        Ok(())
    }
}
