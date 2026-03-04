use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::Write;
use std::os::unix::io::{AsRawFd, FromRawFd};
use tracing::info;

pub struct SecretManager;

impl SecretManager {
    /// Create a memory-backed file (memfd) containing secrets
    pub fn create_secret_fd(label: &str, secret: &str) -> Result<i32> {
        info!("Creating memfd for secret: {}", label);
        
        // memfd_create(name, flags)
        let name = std::ffi::CString::new(label)?;
        let fd = unsafe { libc::memfd_create(name.as_ptr(), 0) };
        
        if fd < 0 {
            return Err(anyhow!("Failed to create memfd: {}", std::io::Error::last_os_error()));
        }

        let mut file = unsafe { File::from_raw_fd(fd) };
        file.write_all(secret.as_bytes())?;
        
        // We don't want to close it yet, as we need to pass it to the child
        // However, FromRawFd will close it when dropped.
        // We must leak it or use a handle.
        let raw_fd = file.as_raw_fd();
        std::mem::forget(file); 

        Ok(raw_fd)
    }

    /// Redact environment variables that might contain secrets
    pub fn redact_env(env: &mut std::collections::HashMap<String, String>) {
        let sensitive_keys = ["API_KEY", "SECRET", "PASSWORD", "TOKEN"];
        for key in env.keys().cloned().collect::<Vec<String>>() {
            for sensitive in &sensitive_keys {
                if key.to_uppercase().contains(sensitive) {
                    info!("Redacting sensitive environment variable: {}", key);
                    env.remove(&key);
                }
            }
        }
    }
}
