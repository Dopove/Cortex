# Cortex 1.1.2: Hardened Hypergraph Runtime

Cortex 1.1 is a production-grade, Rust-powered execution engine for agentic hypergraphs. Migrated from the Mojo prototype, it delivers a high-concurrency orchestrator, zero-copy memory mapping, industrial-grade security, and **Universal Cross-Platform compatibility (Linux, macOS, Windows)**.

## 📚 Documentation Index

- **[Quickstart Guide](./QUICKSTART.md)** - Get running in 5 minutes.
- **[Usage Reference](./USAGE_REFERENCE.md)** - CLI command detailed guide.
- **[System Requirements](./REQUIREMENTS.md)** - Software & Hardware specifications.

## ⚙️ Installation

### One-Command Install (Recommended)

Install Cortex 1.1 globally with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/Dopove/Cortex/main/install.sh | bash
```

### Manual Install

If you prefer to build from source:

```bash
git clone https://github.com/Dopove/Cortex.git
cd Cortex/rust
cargo build --release
sudo cp target/release/cortex /usr/local/bin/
cortex init
```

## 🚀 Quick Start

### 1. Initialize Environment

```bash
./cortex init
```

### 2. Build a Bundle

```bash
./cortex build path/to/your/agent-dir my-agent.cortex
```

### 3. Run safely

```bash
./cortex run my-agent.cortex
```

## 🔒 Security & Hardening

Cortex 1.1 is **Production Certified** with zero known vulnerabilities.

### AES-256-GCM Encryption

Secure your agent bundles with factory-grade encryption:

```bash
export CORTEX_BUNDLE_PASSWORD="your-secure-passphrase"
./cortex encrypt my-agent.cortex
```

### Memory Threshold Guards

Prevents system freezes by auditing available RAM+Swap before loading large models (e.g., 50GB+ models require specific hardware verification).

## ⚡ Performance Features

- **Turbo Mode**: Spawns agents in parallel utilizing `tokio` multi-threading.
- **p99 Latency Monitoring**: Structured JSON logs provide real-time performance SLIs.
- **Zero-Copy FFI**: High-speed communication between Rust runtime and Python agents via PyO3 0.24.

## 🛠️ Folder Structure

- `/`: Production CLI binary and root assets.
- `/rust`: Source code for the 1.1.2 Runtime, Bundler, and Core.

## 📊 Production Certification Metrics

| Phase           | Metric                | Status      |
| --------------- | --------------------- | ----------- |
| Static Shield   | Trivy Vulnerabilities | ✅ 0 Found  |
| Logic Forge     | Unit Test Pass Rate   | ✅ 100%     |
| Gravity Chamber | Memory Leak (1hr)     | ✅ Stable   |
| Launchpad       | Rollback Guard        | ✅ Verified |

---

**Status**: `CORTEX 1.1.2 CERTIFIED PRODUCTION READY`
