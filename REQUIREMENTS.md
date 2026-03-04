# Cortex 1.1.2: System Requirements

This document outlines the hardware and software specifications required to run Cortex 1.1.2 reliably and efficiently.

## 💻 Software Prerequisites

| Requirement          | Version                           | Notes                                            |
| :------------------- | :-------------------------------- | :----------------------------------------------- |
| **Operating System** | Linux (Ubuntu 22.04+ recommended) | MacOS and Windows support (Best Effort)          |
| **Python**           | 3.10 - 3.12                       | Required for agent execution via PyO3            |
| **Pip**              | 23.0+                             | Required for automatic dependency installation   |
| **Nvidia Driver**    | 535+                              | Required only for CUDA/GPU acceleration          |
| **OpenSSL**          | 1.1.1+                            | Required for secure bundle encryption/decryption |

## ⚙️ Hardware Specifications

Cortex 1.1 implements dynamic **Memory Threshold Guards** to prevent system lockups. Requirements scale based on the complexity of the agents and models being executed.

### Tier 1: Small Agents & APIs (e.g., Flask, Scrappers)

- **CPU**: 2+ Cores (AVX2 support recommended)
- **RAM**: 4GB Baseline (1GB required per active bundle)
- **Storage**: 500MB for runtime + bundle size

### Tier 2: Local LLM Inference (e.g., Llama-3 8B, Mistral)

- **CPU**: 4+ Cores (AVX2/AVX512)
- **RAM**: 16GB+
- **GPU**: 8GB VRAM+ (Recommended for low latency)
- **Storage**: 10GB+ (For model weights and KV Cache)

### Tier 3: Large Scale / BLOOM Models (176B+)

- **CPU**: 16+ Cores
- **RAM**: 55GB+ (Enforced via Memory Guard)
- **GPU**: Multiple A100/H100 recommended for BLOOM Full
- **Storage**: 350GB+ (For model weights)

## 🛡️ Production Security Requirements

- **Environment Variables**: Root access to set `CORTEX_BUNDLE_PASSWORD` if using encrypted bundles.
- **Disk Permissions**: Write access to the `/tmp` directory (used for isolated bundle inflation).
- **Networking**: Outbound access to registry if agents require `pip install` during runtime init.
