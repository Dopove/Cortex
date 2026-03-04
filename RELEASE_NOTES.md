# Release Notes: Cortex v2.5.2

Cortex v2.5.2 introduces **Split Networking Architecture** and **Kubernetes Super-Pod** alignment. This release focuses on ultra-low latency egress for cloud model providers and enterprise-grade isolation.

## 🌐 Split Networking (Macvlan Egress)

- **Direct-to-Host Egress**: Agent namespaces now bridge directly to host interfaces via `macvlan`, bypassing the standard bridge bottleneck.
- **Latency Reduction**: Benchmarked **-12.1%** reduction in egress latency for external API calls (HuggingFace, OpenAI, Anthropic).
- **Zero-Copy Inter-Agent Sync**: Maintained **0.05ms** internal speed for agent-to-agent memory passing.

## 🛡️ Super-Pod Security & Isolation

- **Dynamic Firewall (nftables)**: Implemented manifest-driven network allowlisting directly in the runtime.
- **Memory-Only Secret Injection**: Sensitive credentials are now injected via `memfd` segments, leaving zero disk artifacts.
- **Hardware-Aware Status**: Improved status emission for K8s orchestrators, including real-time cgroup limit awareness.

## 📊 Performance Statistics

| Component            | Metric            | Result |
| :------------------- | :---------------- | :----- |
| **Inter-Agent Sync** | p50 Latency       | 0.05ms |
| **Network Egress**   | Latency vs Bridge | -12.1% |
| **Memory RSS**       | Baseline          | 21.4MB |

---

**Status**: `CORTEX v2.5.2 PRODUCTION READY`

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
