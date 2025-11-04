# Cortex v0.4.0 🚀

**Production-grade AI agent packaging with blazing-fast (Beta Release).**

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

### `cortex.toml` Example

```toml
[package]
name = "my-agent"
version = "1.0.0"
description = "My AI agent system"

[runtime]
entry_point = "src/main.py"
working_dir = "src"
```

---

## 🤝 Contributing

Contributions are welcome! Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for details on how to get started.

## 📝 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
