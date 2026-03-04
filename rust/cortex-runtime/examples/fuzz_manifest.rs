use cortex_core::BundleManifest;
use rand::Rng;
use tracing::info;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Phase 5: The Adversarial Arena - Fuzzing Bundle Manifest Parsing");

    let mut rng = rand::thread_rng();

    for i in 0..1000 {
        // Generate random bytes
        let len = rng.gen_range(1..1024);
        let random_bytes: Vec<u8> = (0..len).map(|_| rng.gen()).collect();
        let payload = String::from_utf8_lossy(&random_bytes);

        // Attempt to parse
        let _ = serde_json::from_str::<BundleManifest>(&payload);

        if i % 200 == 0 {
            info!("Fuzzing iteration {}...", i);
        }
    }

    info!("✅ Fuzzing Complete. Survived 1000 iterations of malformed JSON input.");

    Ok(())
}
