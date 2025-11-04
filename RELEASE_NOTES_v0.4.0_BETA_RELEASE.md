# Release Notes - Cortex v0.4.0 (Beta)

**Release Date:** November 5, 2025  
**Status:** Beta Release

This is the first public beta of Cortex v0.4.0, a universal AI agent bundler focused on stability, broad language support, and a fast, efficient sequential execution runtime. As a beta, this version is intended for testing and feedback.

---

## 🚀 Major Features

### 1. Universal Language Bundler
Cortex now officially supports bundling applications written in a wide range of programming languages, including:
- **Python**
- **Go**
- **Rust**
- **TypeScript / JavaScript (Node.js)**
- **Java**
- **Ruby**

The runtime automatically detects the language and uses the appropriate command to execute the bundle.

### 2. Automatic Dependency Management
- ✅ **Python:** Automatically finds and bundles dependencies from `requirements.txt` into the `.cortex` file, eliminating `ModuleNotFoundError` issues at runtime.
- ✅ **Node.js:** Automatically detects `package.json` and bundles the necessary `node_modules` directory.

### 3. Optimized Sequential Runtime
- **Fast Extraction:** Utilizes a highly parallelized ZSTD decompression and file extraction process, achieving speeds of over 3,000 files/sec on modern hardware.
- **Smart Caching:** Bundles are extracted to a cache directory. Subsequent runs are nearly instant, skipping the extraction process entirely if no changes are detected.
- **Streamed Output:** The runtime now streams the stdout/stderr of the running agent directly to the terminal, preventing the application from appearing to hang and providing real-time feedback.

### 4. Comprehensive Project Configuration (`cortex.toml`)
- A single `cortex.toml` file allows for detailed configuration of your project, including package metadata, execution commands, model strategies, and build options.

---

## ✅ Key Bug Fixes & Improvements

- **Build System:** All Mojo compilation warnings have been resolved, and unused code related to the experimental parallel executor has been removed for a cleaner, more stable build.
- **Runtime Stability:**
    - **Fixed Hanging Execution:** The agent's output is now streamed in real-time.
    - **Dependency Errors:** The Python dependency bundler has been made more robust, fixing `ModuleNotFoundError` issues for packages like `flask_sqlalchemy`.
- **CLI Experience:** The CLI has been simplified to focus on the core `build`, `run`, `info`, and `verify` commands.

---

## ⚠️ Breaking Changes

### 1. License Change
- The project license has been changed from the **MIT License** to the **GNU General Public License v3.0 (GPLv3)**. This change ensures that Cortex and its derivatives remain free and open-source. Please review the new `LICENSE` file for details.

### 2. Removal of Experimental Features
- All experimental flags and features related to "Turbo Mode" and parallel execution (`--turbo`, `--workers`, `test-turbo`) have been **removed**. The focus of v0.4.0 is to provide a stable and fast *sequential* runtime.

---

## 📋 Getting Started

### 1. Create `cortex.toml`
Create a `cortex.toml` file in the root of your project. See the main `README.md` for a detailed template.

### 2. Build Your Project
```bash
cortex build /path/to/your-project my-agent.cortex
```

### 3. Run Your Bundle
```bash
cortex run my-agent.cortex
```

---

## 🐛 Known Issues & Limitations

- **Beta Software:** This is a beta release and may contain bugs. Please report any issues on our GitHub page.
- **Port Conflicts:** When running web servers (e.g., Flask, Express, Go HTTP server), the application may fail if the default port is already in use. This is expected behavior and not a Cortex bug.

---

## 📄 License

Cortex v0.4.0 (Beta) is released under the **GNU General Public License v3.0**. See the `LICENSE` file for details.

---

**Thank you for testing the Cortex v0.4.0 Beta!**
