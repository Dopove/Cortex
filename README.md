# Cortex v0.4.0 🚀

**Production-grade AI agent packaging with blazing-fast(Beta Release).**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Mojo](https://img.shields.io/badge/Mojo-24.5-orange.svg)](https://www.modular.com/mojo)
[![Version](https://img.shields.io/badge/version-0.4.0-blue.svg)](https://github.com/yourusername/cortex/releases)

---

## 🚀 Quick Start

Build your agent:
```bash
./cortex build ./my-agent output.cortex
```

Run it anywhere:
```bash
./cortex run output.cortex
```

---

## 📦 What is Cortex?

Cortex is a **production-grade AI agent bundler** that packages multi-agent systems (CrewAI, AutoGen, LangGraph) into **single executable files** with:

- ✅ **Zero dependencies** on target machines
- ✅ **Automatic compression** (ZSTD Level 3)
- ✅ **Incremental builds** (SHA256 change detection)
- ✅ **Smart caching** for fast subsequent runs

---

## 🔧 Installation

### From Source

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/cortex.git
    cd cortex
    ```

2.  **Build with Mojo:**
    ```bash
    mojo build src/cortex_cli.mojo -o cortex
    ```

3.  **Verify installation:**
    ```bash
    ./cortex --version
    ```

### Prerequisites

-   **Mojo 24.5+** ([Install Mojo](https://docs.modular.com/mojo/manual/get-started/))
-   **Python 3.11+** (for agent execution)

---

## 📘 Usage

### Build an Agent Bundle

```bash
./cortex build <project_dir> <output.cortex> [OPTIONS]
```

**Options:**
- `--force`: Force a rebuild, skipping the incremental build check.

**Examples:**

Bundle an agent with embedded models:
```bash
./cortex build ./my-crew output.cortex
```

Force a full rebuild:
```bash
./cortex build ./my-crew output.cortex --force
```

### Run a Bundle

```bash
./cortex run <bundle.cortex>
```

-   The first run (cold start) will take a moment to extract the bundle.
-   Subsequent runs (warm cache) will be significantly faster.

### Get Bundle Information

```bash
./cortex info <bundle.cortex>
```

This command displays:
-   Package metadata
-   Agent count and names
-   Bundle size and compression ratio
-   Creation timestamp

---

## 🎯 Features

### ZSTD Compression
All bundles automatically use **ZSTD Level 3** compression, providing an excellent balance of speed and compression ratio.

### Incremental Builds
Cortex tracks source file changes using **SHA256 hashing**. If no files have changed since the last build, it skips the build process, saving you time.

---

## 🛠️ Advanced

### Project Structure

A typical Cortex project looks like this:

```
my-agent/
├── cortex.toml      # Package manifest
├── agents.yaml      # Agent definitions
├── src/
│   └── main.py      # Entry point
└── requirements.txt # Python dependencies
```

### Creating the `cortex.toml` Manifest

To bundle your project, you must create a `cortex.toml` file in the root directory of your project. This file serves as a manifest, providing Cortex with all the necessary information to package your application correctly.

Below is a comprehensive example of a `cortex.toml` file. You can use this as a template and customize it to fit your project's needs.

### `cortex.toml` Template

```toml
# ============================================
# CORTEX PROJECT MANIFEST
# ============================================
[package]
name = "cortex-project-name"
version = "0.1.0"
description = "A brief description of your project."
authors = ["Your Name <you@example.com>"]

# ============================================
# EXECUTION CONFIGURATION
# ============================================
[execution]
# The command to run your application.
# For Python, it's often a module path.
command = "python -m src.main"

# Add directories to the PYTHONPATH for imports to work correctly.
python_path = "src"

# ============================================
# MODEL CONFIGURATION
# ============================================
[models]
# Strategy can be 'api' (models are fetched at runtime) or 'embed' (models are bundled).
strategy = "api"

[[models.api]]
provider = "ollama"
name = "llama3.1:8b"
env_key = "OLLAMA_BASE_URL" # Environment variable for the model's base URL.

# ============================================
# RUNTIME CONFIGURATION
# ============================================
[runtime]
backend = "auto" # Automatically detect the appropriate backend.

[runtime.hardware]
gpu = true           # Set to true if your project uses a GPU.
gpu_layers = -1      # Number of GPU layers to offload (-1 for all).
cpu_threads = 0      # Number of CPU threads (0 for auto).
memory_limit = "4GB" # Set a memory limit for your application.

[runtime.cache]
enabled = true
path = "~/.cortex/cache" # Path to the cache directory.
persist = true           # Persist the cache between runs.

# ============================================
# BUILD OPTIONS
# ============================================
[build]
optimization = "basic"        # Build optimization level.
include_python_venv = false   # Whether to include the Python virtual environment.
include_tests = false         # Whether to include tests in the bundle.
include_docs = false          # Whether to include documentation.

# A list of files and directories to exclude from the bundle.
exclude = [
    "**/__pycache__/**",
    "**/.git/**",
    "**/.venv/**",
    "**/node_modules/**",
    "**/*.pyc",
    "**/.DS_Store"
]

# ============================================
# AGENTS CONFIGURATION
# ============================================
[agents]
# Define your agents here.
[[agents.crew]]
name = "MyAgent"
role = "An agent that does amazing things."
goal = "To be the best agent ever."
backstory = "This agent has a rich and storied history."
verbose = true
allow_delegation = false
```

---

## 🤝 Contributing

Contributions are welcome! Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for details on how to get started.

## 📝 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
