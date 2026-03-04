use anyhow::{Result, anyhow};
use std::process::Command;
use tracing::{info, warn};

pub struct NetworkManager;

impl NetworkManager {
    /// Create a macvlan interface in bridge mode
    pub fn create_macvlan(name: &str, parent: &str) -> Result<()> {
        info!("Creating macvlan interface {} on parent {}", name, parent);
        
        let status = Command::new("ip")
            .args(["link", "add", name, "link", parent, "type", "macvlan", "mode", "bridge"])
            .status()?;

        if !status.success() {
            return Err(anyhow!("Failed to create macvlan interface"));
        }

        Ok(())
    }

    /// Move an interface into a specific network namespace (by PID)
    pub fn move_to_ns(name: &str, pid: u32) -> Result<()> {
        info!("Moving interface {} to netns of PID {}", name, pid);
        
        let status = Command::new("ip")
            .args(["link", "set", name, "netns", &pid.to_string()])
            .status()?;

        if !status.success() {
            return Err(anyhow!("Failed to move interface to netns"));
        }

        Ok(())
    }

    /// Bring an interface UP inside a namespace
    /// Note: This must be called from WITHIN the namespace or via nsenter
    pub fn set_up(name: &str) -> Result<()> {
        let status = Command::new("ip")
            .args(["link", "set", name, "up"])
            .status()?;

        if !status.success() {
            return Err(anyhow!("Failed to bring interface up"));
        }
        Ok(())
    }

    /// Delete a macvlan interface
    pub fn delete_interface(name: &str) -> Result<()> {
        info!("Deleting interface {}", name);
        let status = Command::new("ip")
            .args(["link", "delete", name])
            .status()?;

        if !status.success() {
            warn!("Failed to delete interface {}", name);
        }
        Ok(())
    }

    /// Attempt to detect the default outbound interface
    pub fn detect_default_interface() -> Result<String> {
        let output = Command::new("sh")
            .args(["-c", "ip route | grep default | awk '{print $5}' | head -n1"])
            .output()?;
        
        let iface = String::from_utf8(output.stdout)?.trim().to_string();
        if iface.is_empty() {
             return Err(anyhow!("Could not detect default network interface"));
        }
        Ok(iface)
    }

    /// Garbage collector to sweep for danling macvlan interfaces
    pub fn run_garbage_collector() {
        std::thread::spawn(|| {
            info!("Network Garbage Collector started.");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                
                // Sweep for mc_* interfaces
                let output = Command::new("ip")
                    .args(["-o", "link", "show"])
                    .output();
                
                if let Ok(out) = output {
                    let lines = String::from_utf8_lossy(&out.stdout);
                    for line in lines.lines() {
                        if line.contains("mc_") {
                            // Extract interface name (e.g., "mc_abcdefgh")
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() > 1 {
                                let iface = parts[1].trim_end_matches(':');
                                // Check if it's orphaned (we could store active sessions in shared state, 
                                // but for now, if it still exists in root netns, it likely failed migration or session ended)
                                // Actually, if it's in the root netns, we can delete it.
                                info!("Cleaning up orphaned interface {}", iface);
                                let _ = Self::delete_interface(iface);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Apply manifest-driven allowlist rules via nftables
    pub fn apply_firewall_rules(session_id: &str, allowed_ips: Vec<String>) -> Result<()> {
        info!("Applying firewall rules for session {}", session_id);
        
        let mut rules = String::new();
        rules.push_str("table inet cortex_filter {\
");
        rules.push_str(&format!("  chain session_{} {{ ", &session_id[..8]));
        rules.push_str("    type filter hook output priority 0; policy drop;\
");
        
        for ip in allowed_ips {
             rules.push_str(&format!("    ip daddr {} accept\
", ip));
        }
        
        rules.push_str("  }\
}");

        let mut child = Command::new("nft")
            .args(["-f", "-"])
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to open nft stdin"))?;
        use std::io::Write;
        stdin.write_all(rules.as_bytes())?;
        drop(stdin);

        let status = child.wait()?;
        if !status.success() {
            return Err(anyhow!("Failed to apply nftables rules"));
        }

        Ok(())
    }
}
