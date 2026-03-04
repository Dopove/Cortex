use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Sandbox {
    pub cgroup_path: PathBuf,
}

impl Sandbox {
    pub fn new(session_id: &str) -> Result<Self> {
        let cortex_cg = PathBuf::from("/sys/fs/cgroup/cortex");

        // Ensure the root cortex cgroup exists (may require sudo)
        if !cortex_cg.exists() {
            if let Err(e) = fs::create_dir_all(&cortex_cg) {
                tracing::warn!("Could not create /sys/fs/cgroup/cortex. Cgroup isolation may not work without root. Error: {}", e);
            }
        }

        let cgroup_path = cortex_cg.join(session_id);
        if !cgroup_path.exists() {
            if let Err(e) = fs::create_dir_all(&cgroup_path) {
                tracing::warn!("Could not create session cgroup {:?}: {}", cgroup_path, e);
            }
        }

        Ok(Self { cgroup_path })
    }

    pub fn set_cpu_limit(&self, max_percent: u32) -> Result<()> {
        if !self.cgroup_path.exists() {
            return Ok(());
        }
        let cpu_max_path = self.cgroup_path.join("cpu.max");
        let quota = max_percent * 1000;
        let limit_str = format!("{} 100000", quota);
        if let Err(e) = fs::write(&cpu_max_path, limit_str) {
            tracing::warn!("Failed to set cpu.max: {}", e);
        }
        Ok(())
    }

    pub fn set_memory_limit(&self, max_mb: u64) -> Result<()> {
        if !self.cgroup_path.exists() {
            return Ok(());
        }
        let mem_max_path = self.cgroup_path.join("memory.max");
        let limit_bytes = max_mb * 1024 * 1024;
        if let Err(e) = fs::write(&mem_max_path, limit_bytes.to_string()) {
            tracing::warn!("Failed to set memory.max: {}", e);
        }
        Ok(())
    }

    pub fn apply_to_pid(&self, pid: u32) -> Result<()> {
        if !self.cgroup_path.exists() {
            return Ok(());
        }
        let procs_path = self.cgroup_path.join("cgroup.procs");
        if let Err(e) = fs::write(&procs_path, pid.to_string()) {
            tracing::warn!("Failed to apply pid to cgroup.procs: {}", e);
        }
        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        if self.cgroup_path.exists() {
            let _ = fs::remove_dir(&self.cgroup_path);
        }
        Ok(())
    }
}
