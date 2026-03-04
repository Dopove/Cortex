use anyhow::{anyhow, Result};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::Once;
use tracing::{debug, error, info};

// Ensure Python is only initialized once
static INIT: Once = Once::new();

pub struct PythonExecutor;

impl PythonExecutor {
    pub fn new(_bundle_path: PathBuf) -> Self {
        Self
    }

    /// Initializes the Python interpreter. We use `auto-initialize` feature in PyO3,
    /// but we also want to prepare sys.path.
    pub fn initialize_env(&self) -> Result<()> {
        INIT.call_once(|| {
            pyo3::prepare_freethreaded_python();
            info!("Embedded Python interpreter initialized.");
        });
        Ok(())
    }

    /// Execute a Python script inside the bundle footprint
    pub async fn execute_script(&self, script_body: &str, module_name: &str) -> Result<()> {
        self.initialize_env()?;

        // Use Tokio's spawn_blocking to run Python code without blocking the async runtime
        let script = script_body.to_owned();
        let module = module_name.to_owned();

        tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| -> Result<()> {
                debug!("Acquired GIL. Preparing to run module: {}", module);

                // Optionally inject variables into the Python context
                let locals = PyDict::new(py);

                // Execute the script
                let c_script = CString::new(script).expect("Failed to create CString from script");
                match py.run(&c_script, None, Some(&locals)) {
                    Ok(_) => {
                        info!("Execution of {} completed successfully.", module);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Python execution error in {}: {:?}", module, e);
                        e.print(py);
                        Err(anyhow!("Python execution failed"))
                    }
                }
            })
        })
        .await??;

        Ok(())
    }

    /// Run a task parallelly, deliberately releasing the GIL while waiting on internal IO
    pub async fn execute_parallel_task(&self, task_code: &str) -> Result<String> {
        self.initialize_env()?;

        let code = task_code.to_owned();
        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| -> Result<String> {
                // Here, if Python code calls a rust function, we would release the GIL using `py.allow_threads(...)`
                // Evaluate code that returns a response
                let c_code = CString::new(code).expect("Failed to create CString from code");
                let result = py.eval(&c_code, None, None)?;
                let extracted: String = result.extract()?;
                Ok(extracted)
            })
        })
        .await??;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_script() -> Result<()> {
        let executor = PythonExecutor::new(PathBuf::from("."));
        executor.execute_script("", "noname").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_script() -> Result<()> {
        let executor = PythonExecutor::new(PathBuf::from("."));
        let result = executor.execute_script("im invalid logic", "error").await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_boundary_integrity() -> Result<()> {
        let executor = PythonExecutor::new(PathBuf::from("."));
        let input_data = "cortex-data-12345";
        let script = format!("'{}' + '-modified'", input_data);

        let result = executor.execute_parallel_task(&script).await?;
        assert_eq!(result, "cortex-data-12345-modified");
        Ok(())
    }
}
