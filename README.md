# Cortex 2.5.9: Hardened Multi-Agent System Bundler & Runtime

Cortex 2.5.9 is a production-grade, Rust-powered execution engine for complex Multi-Agent Systems. It delivers a high-concurrency orchestrator, zero-copy memory mapping, industrial-grade security, and **Universal Cross-Platform compatibility (Linux, macOS, Windows) backed by full CI/CD E2E testing**.

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

Download the pre-compiled binary or archive for your system from **[GitHub Releases](https://github.com/Dopove/Cortex/releases)**. Raw binaries and compressed archives are generated for every release.

#### 🐧 Linux (Ubuntu / Fedora / Arch)

1. Download the raw binary `cortex_v2.5.9_x64_ubuntu` or archive `cortex_v2.5.9_x64_ubuntu.tar.gz`.
2. Make it executable: `chmod +x cortex_v2.5.9_x64_ubuntu`.
3. Install: `sudo mv cortex_v2.5.9_x64_ubuntu /usr/local/bin/cortex`.

#### 🍎 macOS (Apple Silicon & Intel)

1. Download `cortex_v2.5.9_arm64_macos` (Apple Silicon) or `cortex_v2.5.9_x64_macos` (Intel).
2. Make it executable: `chmod +x cortex_v2.5.9_*_macos`.
3. Move to path: `mv cortex_v2.5.9_*_macos /usr/local/bin/cortex`.
4. Allow the binary in **System Settings > Privacy & Security** if prompted by Gatekeeper.

#### 🪟 Windows

1. Download the raw binary `cortex_v2.5.9_x64_windows.exe` or archive `cortex_v2.5.9_x64_windows.zip`.
2. Move it to a folder (e.g., `C:\Program Files\Cortex\`).
3. Add that folder to your **System PATH** environment variable.

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

Cortex 2.5 is **Production Certified** with zero known vulnerabilities.

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
- `/rust`: Source code for the 2.5.9 Runtime, Bundler, Core, and comprehensive Multi-OS test suites.

## 📊 Production Certification Metrics

| Phase           | Metric                | Status      |
| --------------- | --------------------- | ----------- |
| Static Shield   | Trivy Vulnerabilities | ✅ 0 Found  |
| Logic Forge     | Unit Test Pass Rate   | ✅ 100%     |
| Gravity Chamber | Memory Leak (1hr)     | ✅ Stable   |
| Multi-OS Tests  | E2E/Adversarial       | ✅ Verified |
| Launchpad       | Rollback Guard        | ✅ Verified |

---

**Status**: `CORTEX 2.5.9 CERTIFIED PRODUCTION READY`
