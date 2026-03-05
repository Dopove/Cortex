use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, error, info};
#[cfg(target_os = "linux")]
use std::os::unix::io::{BorrowedFd, RawFd};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub command: String,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub timeout_secs: u64,
    pub allow_network: bool,
    pub session_id: String,
    pub macvlan_iface: Option<String>,
    pub allowed_ips: Vec<String>,
    pub secret_fds: std::collections::HashMap<String, i32>,
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

                // Phase 1: Initialize Sandbox per task for resource limits
                let sandbox = crate::sandbox::Sandbox::new(&task.session_id).unwrap_or(
                    crate::sandbox::Sandbox {
                        cgroup_path: std::path::PathBuf::new(),
                    },
                );

                // Example limits: 50% CPU, 2048 MB RAM (in production, passed from manifest)
                let _ = sandbox.set_cpu_limit(50);
                let _ = sandbox.set_memory_limit(2048);

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

                #[cfg(target_os = "linux")]
                let mut sync_pipe: Option<(RawFd, RawFd)> = None;
                #[cfg(target_os = "linux")]
                if task.macvlan_iface.is_some() {
                    if let Ok((rx, tx)) = nix::unistd::pipe() {
                        use std::os::unix::io::IntoRawFd;
                        sync_pipe = Some((rx.into_raw_fd(), tx.into_raw_fd()));
                    }
                }

                #[cfg(target_os = "linux")]
                {
                    // Establish ZeroCopyBus for 0.05ms hardware latency shared memory message passing
                    if let Ok(bus) = crate::shm::ZeroCopyBus::new(1024 * 1024) {
                        let fd = bus.get_fd();
                        cmd.env("CORTEX_SHM_FD", fd.to_string());
                    }

                    let sandbox_for_child = sandbox.clone();
                    let allow_net = task.allow_network;
                    let macvlan_iface = task.macvlan_iface.clone();
                    
                    unsafe {
                        let inner_sync = sync_pipe;
                        cmd.pre_exec(move || {
                            // Phase 1 (v2.2.1): Assign child to OS cgroups slice
                            let _ = sandbox_for_child.apply_to_pid(std::process::id());

                            // Phase 3/4 Security Boundaries:
                            let mut flags = libc::CLONE_NEWPID;
                            const CLONE_NEWCGROUP: libc::c_int = 0x02000000;
                            flags |= CLONE_NEWCGROUP;

                            if !allow_net || macvlan_iface.is_some() {
                                flags |= libc::CLONE_NEWNET;
                            }

                            // Spawn agent inside isolated PID / CGROUP / NET boundaries
                            if std::env::var("CORTEX_NO_ISOLATION").is_err() {
                                if libc::unshare(flags) != 0 {
                                    return Err(std::io::Error::last_os_error());
                                }
                            }

                            // Phase 5 (v2.5.2): Setup macvlan if provided
                            if let Some(ref iface) = macvlan_iface {
                                if let Some((rx, tx)) = inner_sync {
                                    // 1. Signal READY to parent
                                    let borrow_tx = BorrowedFd::borrow_raw(tx);
                                    let _ = nix::unistd::write(borrow_tx, b"R");
                                    // 2. Wait for GO from parent (reading 1 byte)
                                    let mut buf = [0u8; 1];
                                    let _ = nix::unistd::read(rx, &mut buf);
                                    // 3. Bring interface up
                                    let _ = crate::network::NetworkManager::set_up(iface);
                                }
                            }

                            Ok(())
                        });
                    }
                }

                for (key, fd) in &task.secret_fds {
                    let fd_path = format!("/proc/self/fd/{}", fd);
                    cmd.env(key, fd_path);
                }

                cmd.stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());

                let mut child = match cmd.spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        error!(" ❌ [Task {}] Failed to spawn process: {}", task.id, e);
                        return (format!("EXCEPTION: {}", e), TaskOutcome::Error(0));
                    }
                };

                // Parent-side sync
                #[cfg(target_os = "linux")]
                if let (Some(iface), Some((rx, tx))) = (&task.macvlan_iface, sync_pipe) {
                    // 1. Wait for READY from child
                    let mut buf = [0u8; 1];
                    let _ = nix::unistd::read(rx, &mut buf);
                    // 2. Move interface into child namespace
                    if let Some(pid) = child.id() {
                        let _ = crate::network::NetworkManager::move_to_ns(iface, pid);
                        
                        // 3. Apply Firewall rules (Manifest-Driven)
                        if !task.allowed_ips.is_empty() {
                             let _ = crate::network::NetworkManager::apply_firewall_rules(&task.session_id, task.allowed_ips.clone());
                        }
                    }
                    // 3. Signal GO to child
                    let borrow_tx = unsafe { BorrowedFd::borrow_raw(tx) };
                    let _ = nix::unistd::write(borrow_tx, b"G");
                }

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
            allow_network: false,
            session_id: "test-session".to_string(),
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
            allow_network: false,
            session_id: "test-session".to_string(),
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
            allow_network: false,
            session_id: "test-session".to_string(),
        };

        let (results, metrics) = executor.execute(vec![task]).await?;
        assert!(results[0].contains("TIMEOUT"));
        assert_eq!(metrics.timed_out_tasks, 1);
        Ok(())
    }
}
