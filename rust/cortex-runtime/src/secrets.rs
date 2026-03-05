use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use tracing::info;

pub struct SecretManager;

impl SecretManager {
    /// Create a memory-backed file (memfd) containing secrets
    pub fn create_secret_fd(label: &str, secret: &str) -> Result<i32> {
        info!("Creating secret storage for: {}", label);
        
        #[cfg(target_os = "linux")]
        {
            let name = std::ffi::CString::new(label)?;
            let fd = unsafe { libc::memfd_create(name.as_ptr(), 0) };
            
            if fd < 0 {
                return Err(anyhow!("Failed to create memfd: {}", std::io::Error::last_os_error()));
            }

            let mut file = unsafe { File::from_raw_fd(fd) };
            file.write_all(secret.as_bytes())?;
            
            let raw_fd = file.as_raw_fd();
            std::mem::forget(file); 
            Ok(raw_fd)
        }
        #[cfg(all(unix, not(target_os = "linux")))]
        {
            // Fallback for macOS/BSD: Use a temporary file in /tmp or shm_open
            // For now, we'll use a temporary file and return its FD.
            // In a future hardening pass, shm_open will be used.
            let mut temp_file = tempfile::NamedTempFile::new()?;
            temp_file.write_all(secret.as_bytes())?;
            let file = temp_file.into_file();
            let fd = file.as_raw_fd();
            std::mem::forget(file);
            Ok(fd)
        }
        #[cfg(windows)]
        {
            // Windows: Use temporary files as there is no direct memfd equivalent for handle passing in this logic.
            let mut path = std::env::temp_dir();
            path.push(format!("cortex_secret_{}.tmp", label));
            let mut file = File::create(&path)?;
            file.write_all(secret.as_bytes())?;
            
            // We return a handle as i32 for consistency (limited to 32-bit processes/handles if strict i32)
            // but for builds, we just need it to compile.
            let handle = file.as_raw_handle();
            std::mem::forget(file);
            Ok(handle as i32)
        }
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
