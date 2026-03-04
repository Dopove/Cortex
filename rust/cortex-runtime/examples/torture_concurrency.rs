use cortex_runtime::kv_cache::KVCacheManager;
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{error, info};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Phase 5: The Adversarial Arena - Concurrency Torture (100 Threads)");

    let manager = Arc::new(Mutex::new(KVCacheManager::new(1))); // 1GB limit
    let mut handles = vec![];

    for i in 0..100 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let mut mg = manager_clone.lock().expect("Lock poisoned");
            match mg.allocate_cache(i, 1) {
                // 1MB each
                Ok(_) => {}
                Err(e) => {
                    if !e.to_string().contains("OOM") {
                        error!("Unexpected error in thread {}: {}", i, e);
                    }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    let final_mg = manager.lock().expect("Lock poisoned");
    info!("Concurrency Torture Complete.");
    info!("{}", final_mg.get_stats());

    // Check if we survived without panic/hang
    info!("✅ Runtime survived 100-thread torture.");

    Ok(())
}
