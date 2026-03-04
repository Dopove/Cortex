# Cortex 1.1.2: Hardened Multi-Agent System Bundler & Runtime

Cortex 1.1 is a production-grade, Rust-powered execution engine for complex Multi-Agent Systems. Migrated from the Mojo prototype, it delivers a high-concurrency orchestrator, zero-copy memory mapping, industrial-grade security, and **Universal Cross-Platform compatibility (Linux, macOS, Windows)**.

## 📚 Documentation Index

- **[Quickstart Guide](./QUICKSTART.md)** - Get running in 5 minutes.
- **[Usage Reference](./USAGE_REFERENCE.md)** - CLI command detailed guide.
- **[System Requirements](./REQUIREMENTS.md)** - Software & Hardware specifications.

## ⚙️ Installation

### One-Command Install (Recommended)

#### 🐧 Linux & 🍎 macOS

```bash
curl -sSL https://raw.githubusercontent.com/Dopove/Cortex/main/install.sh | bash
```

#### 🪟 Windows (PowerShell)

```powershell
iwr https://raw.githubusercontent.com/Dopove/Cortex/main/install.ps1 -useb | iex
```

### Manual Install (Binary)

Download the pre-compiled binary for your system from **[GitHub Releases](https://github.com/Dopove/Cortex/releases)**:

#### 🐧 Linux

1. Download `Cortex-v1.1.2-linux-x86_64.tar.gz`.
2. Unpack: `tar -xzvf Cortex-v1.1.2-linux-x86_64.tar.gz`.
3. Install: `sudo mv cortex /usr/local/bin/`.

#### 🍎 macOS

1. Download `Cortex-v1.1.2-macos-arm64.tar.gz` (Apple Silicon) or `x86_64` (Intel).
2. Unpack and move the binary to `/usr/local/bin/`.
3. Allow the binary in **System Settings > Privacy & Security** if prompted.

#### 🪟 Windows

1. Download `Cortex-v1.1.2-windows-x86_64.zip`.
2. Extract the `cortex.exe` file.
3. Move it to a folder (e.g., `C:\Program Files\Cortex\`).
4. Add that folder to your **System PATH** environment variable.

---

Finally, run `cortex init` to prepare your environment.

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
