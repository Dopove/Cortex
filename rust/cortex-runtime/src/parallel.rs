use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub command: String,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Default, Clone)]
pub struct ExecutionMetrics {
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
    pub timed_out_tasks: usize,
    pub p50_latency_ms: u64,
    pub p90_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub total_duration_ms: u64,
}

pub struct ParallelExecutor {
    num_workers: usize,
}

impl ParallelExecutor {
    pub fn new(num_workers: usize) -> Self {
        Self { num_workers }
    }

    pub async fn execute(&self, tasks: Vec<Task>) -> Result<(Vec<String>, ExecutionMetrics)> {
        let start_time = std::time::Instant::now();
        info!(
            "⚡ ParallelExecutor: Executing {} tasks across {} workers...",
            tasks.len(),
            self.num_workers
        );

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.num_workers));
        let mut futures = Vec::new();

        let tasks_count = tasks.len();
        for task in tasks {
            let permit = semaphore.clone().acquire_owned().await?;

            let future = tokio::spawn(async move {
                let _permit = permit; // Hold permit until task drops

                debug!(" - Spawning task [{}]: {}", task.id, task.name);

                // For Windows compatibility vs Unix shell, it's safer to execute via sh or cmd
                // but since Mojo code used `shell=True`, we will emulate that.
                let is_windows = cfg!(windows);
                let mut cmd = if is_windows {
                    let mut c = Command::new("cmd");
                    c.args(["/C", &task.command]);
                    c
                } else {
                    let mut c = Command::new("sh");
                    c.args(["-c", &task.command]);
                    c
                };

                cmd.current_dir(&task.cwd).envs(&task.env);
                
                cmd.stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());

                let mut child = match cmd.spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        error!(" ❌ [Task {}] Failed to spawn process: {}", task.id, e);
                        return (format!("EXCEPTION: {}", e), TaskOutcome::Error(0));
                    }
                };

                let mut stdout = child.stdout.take().unwrap();
                let mut stderr = child.stderr.take().unwrap();

                use tokio::io::AsyncReadExt;
                let task_start = std::time::Instant::now();
                let mut captured_output = String::new();
                let task_id = task.id;
                let timeout_dur = Duration::from_secs(task.timeout_secs);
                let sleep = tokio::time::sleep(timeout_dur);
                tokio::pin!(sleep);

                let mut stdout_buf = [0u8; 1024];
                let mut stderr_buf = [0u8; 1024];

                let result = loop {
                    tokio::select! {
                        res = stdout.read(&mut stdout_buf) => {
                            match res {
                                Ok(0) => {}, // EOF
                                Ok(n) => {
                                    let data = String::from_utf8_lossy(&stdout_buf[..n]);
                                    print!("{}", data);
                                    captured_output.push_str(&data);
                                }
                                Err(_) => {},
                            }
                        }
                        res = stderr.read(&mut stderr_buf) => {
                            match res {
                                Ok(0) => {}, // EOF
                                Ok(n) => {
                                    let data = String::from_utf8_lossy(&stderr_buf[..n]);
                                    eprint!("{}", data);
                                    captured_output.push_str(&data);
                                }
                                Err(_) => {},
                            }
                        }
                        status_res = child.wait() => {
                            let duration = task_start.elapsed().as_millis() as u64;
                            match status_res {
                                Ok(status) => {
                                    if status.success() {
                                        info!(" ✓ [Task {}] Completed successfully", task_id);
                                        break (captured_output, TaskOutcome::Success(duration));
                                    } else {
                                        error!(
                                            " ❌ [Task {}] Exit code {}",
                                            task_id,
                                            status.code().unwrap_or(-1)
                                        );
                                        break (format!("{}\nEXIT_CODE:{}", captured_output, status.code().unwrap_or(-1)), TaskOutcome::Error(duration));
                                    }
                                }
                                Err(e) => {
                                    error!(" ❌ [Task {}] Execution error: {}", task_id, e);
                                    break (format!("{}\nEXCEPTION: {}", captured_output, e), TaskOutcome::Error(duration));
                                }
                            }
                        }
                        _ = &mut sleep => {
                            let duration = task_start.elapsed().as_millis() as u64;
                            error!(" ❌ [Task {}] Timed out after {} seconds", task_id, task.timeout_secs);
                            let _ = child.kill().await;
                            break (format!("{}\nTIMEOUT: Task failed", captured_output), TaskOutcome::Timeout(duration));
                        }
                    }
                };
                result
            });

            futures.push(future);
        }

        let mut results = Vec::new();
        let mut outcomes = Vec::new();
        for fut in futures {
            let res = fut.await?;
            results.push(res.0);
            outcomes.push(res.1);
        }

        let mut metrics = ExecutionMetrics {
            total_tasks: tasks_count,
            total_duration_ms: start_time.elapsed().as_millis() as u64,
            ..Default::default()
        };

        let mut latencies = Vec::new();
        for outcome in outcomes {
            match outcome {
                TaskOutcome::Success(d) => {
                    metrics.successful_tasks += 1;
                    latencies.push(d);
                }
                TaskOutcome::Error(d) => {
                    metrics.failed_tasks += 1;
                    latencies.push(d);
                }
                TaskOutcome::Timeout(d) => {
                    metrics.timed_out_tasks += 1;
                    latencies.push(d);
                }
            }
        }

        if !latencies.is_empty() {
            latencies.sort_unstable();
            metrics.p50_latency_ms = latencies[latencies.len() / 2];
            metrics.p90_latency_ms = latencies[(latencies.len() as f64 * 0.9) as usize];
            metrics.p99_latency_ms = latencies[(latencies.len() as f64 * 0.99) as usize];
        }

        info!("✅ Parallel execution complete. Metrics: {:?}", metrics);
        Ok((results, metrics))
    }
}

enum TaskOutcome {
    Success(u64),
    Error(u64),
    Timeout(u64),
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_parallel_execution() -> Result<()> {
        let executor = ParallelExecutor::new(2);
        let task = Task {
            id: 1,
            name: "test-echo".to_string(),
            command: "echo 'hello world'".to_string(),
            cwd: std::env::current_dir()?,
            env: HashMap::new(),
            timeout_secs: 60,
        };

        let (results, metrics) = executor.execute(vec![task]).await?;
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("hello world"));
        assert_eq!(metrics.total_tasks, 1);
        assert_eq!(metrics.successful_tasks, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_parallel_env_vars() -> Result<()> {
        let executor = ParallelExecutor::new(2);
        let mut env = HashMap::new();
        env.insert("CORTEX_TEST_VAR".to_string(), "cortex-value".to_string());

        let task = Task {
            id: 2,
            name: "test-env".to_string(),
            command: "echo $CORTEX_TEST_VAR".to_string(),
            cwd: std::env::current_dir()?,
            env,
            timeout_secs: 60,
        };

        let (results, metrics) = executor.execute(vec![task]).await?;
        assert!(results[0].contains("cortex-value"));
        assert_eq!(metrics.successful_tasks, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_timeout_handling() -> Result<()> {
        let executor = ParallelExecutor::new(1);
        let task = Task {
            id: 3,
            name: "test-timeout".to_string(),
            command: "sleep 10".to_string(), // Exceeds 2s timeout
            cwd: std::env::current_dir()?,
            env: HashMap::new(),
            timeout_secs: 2,
        };

        let (results, metrics) = executor.execute(vec![task]).await?;
        assert!(results[0].contains("TIMEOUT"));
        assert_eq!(metrics.timed_out_tasks, 1);
        Ok(())
    }
}
