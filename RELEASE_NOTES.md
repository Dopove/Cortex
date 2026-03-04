# Release Notes: Cortex v1.1.2

Cortex v1.1.2 marks the transition from our experimental Mojo prototype to a production-hardened, multi-platform Rust engine. This release focuses on absolute security, industrial-grade performance, and high-concurrency execution for Multi-Agent Systems.

## 🛡️ Industrial Security

- **AES-256-GCM Bundle Encryption**: Added the ability to secure agent bundles (`.cortex` files) with factory-grade encryption.
- **Argon2id Key Derivation**: Professional password-to-key mapping to resist brute-force attacks.
- **Absolute Vulnerability Shield**: Upgraded `pyo3` to 0.24.2 and cleared all dependency vulnerabilities (Trivy Certified: 0 Findings).
- **Secure Runtime Isolation**: Enhanced virtual environment management for Python agents.

## ⚡ High-Performance Runtime

- **Rust Orchestrator**: Replaced the Mojo core with a high-concurrency Rust engine powered by `tokio`.
- **Parallel Turbo Mode**: Execute agent sequences in parallel with high-performance task scheduling.
- **Zero-Copy Memory Mapping**: Utilized `memmap2` for ultra-fast loading of large GGUF/Safetensors weights without OOM.
- **Telemetry & Monitoring**: Structured JSON logs provide real-time p99 latency tracking for mission-critical deployments.

## ⚙️ Universal Portability

- **Cross-Platform Compilation**: The 100% Rust-native core is now fully compatible with **macOS** and **Windows**, enabling consistent agent behavior across diverse operating environments.
- **Global Installation**: New `install.sh` for single-command global deployment via GitHub (sh-compatible environments).
- **Flattened Documentation**: Moved Quickstart, Usage, and requirements to root for instant accessibility.
- **Lean Distribution**: Purged legacy prototype bloat and example templates for a 100% Rust-native core.

---

**Status**: `CORTEX v1.1.2 PRODUCTION READY`
