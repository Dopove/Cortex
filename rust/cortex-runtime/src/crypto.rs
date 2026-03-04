use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::fs;
use std::path::Path;
use tracing::info;

pub struct EncryptionEngine;

impl EncryptionEngine {
    /// Derive a 32-byte key from a password and salt using Argon2id
    fn derive_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
        let mut key = [0u8; 32];
        let argon2 = Argon2::default();
        
        // We use a simplified derivation for CLI use
        // In a real production app, we'd use the password_hash crate's full flow
        // but here we just need a deterministic 32-byte key from Argon2.
        // Argon2::hash_password_into is suitable here.
        argon2.hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| anyhow::anyhow!("Key derivation failed: {}", e))?;
            
        Ok(key)
    }

    pub fn encrypt_file(file_path: &Path, password: &str) -> Result<()> {
        info!("🔐 Encrypting {:?} with AES-256-GCM...", file_path);
        
        let data = fs::read(file_path).context("Failed to read file for encryption")?;
        
        // Generate random salt (16 bytes) and nonce (12 bytes)
        let mut salt_bytes = [0u8; 16];
        rand::Rng::fill(&mut rand::thread_rng(), &mut salt_bytes);
        
        let mut nonce_bytes = [0u8; 12];
        rand::Rng::fill(&mut rand::thread_rng(), &mut nonce_bytes);
        
        let key = Self::derive_key(password, &salt_bytes)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!("Cipher init failed: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, data.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
            
        // Construct final payload: [SALT][NONCE][CIPHERTEXT]
        let mut final_payload = Vec::with_capacity(16 + 12 + ciphertext.len());
        final_payload.extend_from_slice(&salt_bytes);
        final_payload.extend_from_slice(&nonce_bytes);
        final_payload.extend_from_slice(&ciphertext);
        
        fs::write(file_path, final_payload).context("Failed to write encrypted file")?;
        
        Ok(())
    }

    pub fn decrypt_data(encrypted_data: &[u8], password: &str) -> Result<Vec<u8>> {
        if encrypted_data.len() < 28 {
            return Err(anyhow::anyhow!("Invalid encrypted data format"));
        }
        
        let salt: [u8; 16] = encrypted_data[0..16].try_into().unwrap();
        let nonce_bytes: [u8; 12] = encrypted_data[16..28].try_into().unwrap();
        let ciphertext = &encrypted_data[28..];
        
        let key = Self::derive_key(password, &salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!("Cipher init failed: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed (Likely invalid password): {}", e))?;
            
        Ok(plaintext)
    }

    /// Reads bundle data and decrypts if it looks encrypted and password is provided.
    pub fn read_bundle(path: &Path) -> Result<Vec<u8>> {
        let data = fs::read(path).context("Failed to read bundle file")?;
        
        // Zstd magic: 0xFD2FB528 (little-endian: 0x28, 0xB5, 0x2F, 0xFD)
        if data.len() >= 4 && data[0] == 0x28 && data[1] == 0xB5 && data[2] == 0x2F && data[3] == 0xFD {
            // Looks like a regular Zstd archive
            return Ok(data);
        }

        // Try decryption if password env var is set
        if let Ok(password) = std::env::var("CORTEX_BUNDLE_PASSWORD") {
            info!("🔓 Encrypted bundle detected. Attempting decryption...");
            return Self::decrypt_data(&data, &password);
        }

        if data.len() >= 28 {
            return Err(anyhow::anyhow!("Bundle appears encrypted but CORTEX_BUNDLE_PASSWORD is not set."));
        }

        Ok(data)
    }
}
