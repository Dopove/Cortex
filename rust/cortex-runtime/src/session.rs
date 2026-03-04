use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub bundle_name: String,
    pub pid: u32,
    pub start_time: u64,
}

pub struct SessionManager {
    root_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let root_dir = std::env::temp_dir().join("cortex").join("sessions");
        if !root_dir.exists() {
            fs::create_dir_all(&root_dir)?;
        }
        Ok(Self { root_dir })
    }

    pub fn record_session(&self, info: SessionInfo) -> Result<()> {
        let path = self.root_dir.join(format!("{}.json", info.session_id));
        let content = serde_json::to_string(&info)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let mut sessions = Vec::new();
        if !self.root_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(&self.root_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(info) = serde_json::from_str::<SessionInfo>(&content) {
                        // Check if process still exists
                        if sysinfo::System::new_all()
                            .process(sysinfo::Pid::from(info.pid as usize))
                            .is_some()
                        {
                            sessions.push(info);
                        } else {
                            // Cleanup stale session
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
        Ok(sessions)
    }

    pub fn remove_session(&self, session_id: &str) -> Result<()> {
        let path = self.root_dir.join(format!("{}.json", session_id));
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn kill_session(&self, session_id: &str) -> Result<()> {
        let path = self.root_dir.join(format!("{}.json", session_id));
        if !path.exists() {
            return Err(anyhow::anyhow!("Session {} not found", session_id));
        }

        let content = fs::read_to_string(&path)?;
        let info: SessionInfo = serde_json::from_str(&content)?;

        info!("Evaporating session {} (PID: {})...", session_id, info.pid);

        // Send SIGKILL to the root process
        #[cfg(unix)]
        {
            let pid = nix::unistd::Pid::from_raw(info.pid as i32);
            let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGKILL);

            // Also cleanup cgroup if exists
            let cgroup_path = PathBuf::from("/sys/fs/cgroup/cortex").join(session_id);
            if cgroup_path.exists() {
                let _ = fs::remove_dir(&cgroup_path);
            }
        }

        let _ = fs::remove_file(path);
        Ok(())
    }
}
