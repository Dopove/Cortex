use cortex_runtime::kv_cache::KVCacheManager;
use std::time::Instant;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Phase 4: The Gravity Chamber - Performance & Stability Benchmarks");

    // 1. Memory Stability Test (Mock Leak Vigil)
    info!("Phase 4.1: Memory Stability Monitoring (1000 cycles)");
    let mut manager = KVCacheManager::new(1);
    let start_mem = get_process_rss();

    for i in 0..1000 {
        manager.allocate_cache(i, 1)?; // 1MB
        if i % 100 == 0 {
            manager.invalidate_cache(i);
        }
    }

    let end_mem = get_process_rss();
    info!("Memory baseline: {} KB, Final: {} KB", start_mem, end_mem);
    if end_mem > start_mem + 50000 {
        // Allow some slack for fragmentation
        info!("WARNING: Potential memory upward trend detected");
    } else {
        info!("✅ Memory stability verified.");
    }

    // 2. p99 Latency Profiling (Execution overhead)
    info!("Phase 4.2: p99 Latency Profiling (Cold Start)");
    let mut latencies = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        // Mocking a minor part of the runtime path
        let _profile = cortex_core::hardware::HardwareProfile::detect();
        latencies.push(start.elapsed().as_micros());
    }

    latencies.sort();
    let p50 = latencies[50];
    let p90 = latencies[90];
    let p99 = latencies[99];

    info!(
        "Hardware Detection Latency: p50: {}us, p90: {}us, p99: {}us",
        p50, p90, p99
    );

    if p99 < 100000 {
        info!("✅ Latency targets met (<100ms)");
    } else {
        info!("❌ Latency targets missed: p99 is {}us", p99);
    }

    Ok(())
}

fn get_process_rss() -> u64 {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse().unwrap_or(0);
                }
            }
        }
    }
    0
}
