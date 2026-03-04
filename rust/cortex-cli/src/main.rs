use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, Level};

/// Cortex: Production-grade Multi-Agent System Runtime
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Increase logging verbosity
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Emit logs in structured JSON format
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build a multi-agent project into a .cortex bundle
    Build {
        /// Directory containing the project
        project_dir: PathBuf,

        /// Output path for the .cortex bundle
        output: PathBuf,
    },

    /// Execute a .cortex bundle natively
    Run {
        /// The .cortex bundle to execute
        bundle: PathBuf,

        /// Specify the GPU ID to bind to (optional)
        #[arg(long)]
        gpu: Option<u32>,
    },

    /// Display header and manifest information embedded in a .cortex bundle
    Info {
        /// The .cortex bundle to inspect
        bundle: PathBuf,
    },

    /// Verify checksums and bundle integrity
    Verify {
        /// The .cortex bundle to verify
        bundle: PathBuf,
    },

    /// Evaluate agent safety characteristics via Anthropic Bloom simulation integration
    Eval {
        /// The .cortex bundle or project source to evaluate
        target: PathBuf,
    },

    /// Execute a .cortex bundle natively with the Turbo Engine (Parallel Multi-Agent)
    Turbo {
        /// The .cortex bundle to execute
        bundle: PathBuf,

        /// Specify the GPU ID to bind to (optional)
        #[arg(long)]
        gpu: Option<u32>,
    },

    /// Extract a .cortex bundle to a directory
    Extract {
        /// The .cortex bundle to extract
        bundle: PathBuf,

        /// The output directory
        target_dir: PathBuf,
    },

    /// Encrypt a .cortex bundle using AES-GCM
    Encrypt {
        /// The .cortex bundle to encrypt
        bundle: PathBuf,
    },

    /// Initialize the Cortex runtime (downloads common packages)
    Init,

    /// List active agent sessions
    Ps,

    /// Terminate a running agent session
    Kill {
        /// The ID of the session to terminate
        session_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configure logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    if cli.json {
        tracing_subscriber::fmt()
            .with_max_level(log_level)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt().with_max_level(log_level).init();
    }

    match cli.command {
        Commands::Build {
            project_dir,
            output,
        } => {
            info!(
                "Building Cortex bundle from {:?} to {:?}",
                project_dir, output
            );
            let bundler = cortex_bundler::Bundler::new(project_dir, output);
            bundler.run_bundle_pipeline().await?;
        }
        Commands::Run { bundle, gpu } => {
            info!("Executing Cortex bundle {:?}", bundle);
            if let Some(gpu_id) = gpu {
                info!("Binding to GPU ID: {}", gpu_id);
            }
            cortex_runtime::Orchestrator::execute(&bundle, gpu, false).await?;
        }
        Commands::Turbo { bundle, gpu } => {
            info!("⚡ Activating Turbo Mode for bundle {:?}", bundle);
            if let Some(gpu_id) = gpu {
                info!("Binding to GPU ID: {}", gpu_id);
            }
            cortex_runtime::Orchestrator::execute(&bundle, gpu, true).await?;
        }
        Commands::Info { bundle } => {
            info!("Displaying bundle info for {:?}", bundle);
            let manifest = cortex_runtime::inspect::InspectEngine::get_manifest(&bundle)?;

            println!(
                "\n📦 Cortex Bundle: {} v{}",
                manifest.package.name, manifest.package.version
            );
            if let Some(desc) = &manifest.package.description {
                println!("📝 Description: {}", desc);
            }

            println!("\n🤖 Agents:");
            for agent in &manifest.agents {
                println!("  - {} (Entry: {})", agent.name, agent.entry_point);
            }

            println!("\n🧠 Models:");
            for model in &manifest.models {
                let arch = model
                    .architecture
                    .as_ref()
                    .map(|a| format!("{:?}", a))
                    .unwrap_or_else(|| "Unknown".to_string());
                println!(
                    "  - {} [Arch: {}] (Path: {:?})",
                    model.name, arch, model.path
                );
            }
            println!();
        }
        Commands::Verify { bundle } => {
            info!("Verifying bundle integrity for {:?}", bundle);
            cortex_runtime::inspect::InspectEngine::verify(&bundle)?;
            println!("✅ Bundle verification successful.");
        }
        Commands::Eval { target } => {
            info!("Running evaluation framework on {:?}", target);
            let engine = cortex_runtime::evaluation::EvaluationEngine::new(target);
            engine.evaluate().await?;
        }
        Commands::Extract { bundle, target_dir } => {
            info!("Extracting Cortex bundle {:?} to {:?}", bundle, target_dir);
            cortex_runtime::Orchestrator::extract(&bundle, &target_dir)?;
            println!("✅ Bundle extracted to {:?}", target_dir);
        }
        Commands::Encrypt { bundle } => {
            info!("Encrypting Cortex bundle {:?}", bundle);
            println!("🔒 Bundle encryption initialized.");
            cortex_runtime::Orchestrator::encrypt(&bundle)?;
            println!("✅ Bundle successfully encrypted.");
        }
        Commands::Init => {
            info!("Initializing Cortex Runtime and Pre-warming common dependencies");
            cortex_runtime::Orchestrator::init_env().await?;
        }
        Commands::Ps => {
            let session_mgr = cortex_runtime::session::SessionManager::new()?;
            let sessions = session_mgr.list_sessions()?;
            if sessions.is_empty() {
                println!("No active Cortex sessions.");
            } else {
                println!(
                    "{:<40} {:<20} {:<10} {:<20}",
                    "SESSION ID", "BUNDLE", "PID", "START TIME"
                );
                println!("{:-<95}", "");
                for s in sessions {
                    let time = chrono::DateTime::from_timestamp(s.start_time as i32 as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    println!(
                        "{:<40} {:<20} {:<10} {:<20}",
                        s.session_id, s.bundle_name, s.pid, time
                    );
                }
            }
        }
        Commands::Kill { session_id } => {
            let session_mgr = cortex_runtime::session::SessionManager::new()?;
            session_mgr.kill_session(&session_id)?;
            println!("✅ Session {} evaporated.", session_id);
        }
    }

    Ok(())
}
