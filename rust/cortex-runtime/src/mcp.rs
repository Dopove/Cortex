use anyhow::Result;
use std::process::{Child, Command, Stdio};
use tracing::info;

pub struct McpSidecar {
    pub name: String,
    pub command: String,
    process: Option<Child>,
}

impl McpSidecar {
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            process: None,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        info!("Starting local MCP sidecar: {}", self.name);
        let child = Command::new("sh")
            .args(["-c", &self.command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        self.process = Some(child);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            info!("Stopping MCP sidecar: {}", self.name);
            child.kill()?;
            child.wait()?;
        }
        Ok(())
    }
}
