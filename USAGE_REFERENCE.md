# Cortex 1.1: CLI Usage Reference

Comprehensive documentation for all `cortex` subcommands.

## Commands Overview

- [`build`](#build) - Pack a project directory into a `.cortex` bundle.
- [`run`](#run) - Execute agents within a bundle.
- [`init`](#init) - Initialize the execution environment.
- [`info`](#info) - Inspect bundle contents.
- [`encrypt`](#encrypt) - Secure a bundle with AES-256-GCM.
- [`verify`](#verify) - Validate bundle integrity (Zstd + SHA256).
- [`extract`](#extract) - Unpack a bundle to a directory.
- [`eval`](#eval) - Run Anthropic Bloom safety simulations.

---

### `build`

`cortex build <PROJECT_DIR> <OUTPUT>`

Compresses and archives a directory into a standard Cortex bundle.

- **PROJECT_DIR**: Path to the source code (must contain `cortex.toml`).
- **OUTPUT**: Target filename (e.g., `agent.cortex`).

### `run`

`cortex run <BUNDLE>`

Inflates the bundle into a high-concurrency runtime.

- **--turbo**: Activates parallel execution for multi-agent bundles.
- **--gpu <ID>**: Enables CUDA acceleration for the specified GPU.
- **--json**: Outputs real-time execution logs in JSON format for production indexing.

### `init`

`cortex init`

Pre-flight check for the local machine. It detects AVX/CUDA capabilities and confirms Python 3.10+ availability.

### `encrypt`

`cortex encrypt <BUNDLE>`

Applies industry-standard encryption.

- **Requires**: `CORTEX_BUNDLE_PASSWORD` environment variable.
- **Algorithm**: AES-256-GCM (Authenticated Encryption).

### `info`

`cortex info <BUNDLE>`

Displays the manifest (`bundle.json`) metadata, agent counts, and model paths without extracting the archive.

### `verify`

`cortex verify <BUNDLE>`

Performs a deep integrity check. Validates the Zstd stream, Tar headers, and matches SHA256 checksums for all internal blobs.

### `eval`

`cortex eval <BUNDLE>`

Triggers the **Bloom Framework** simulation. Executes adversarial scenarios against the agents and provides a safety consensus report.

---

## Environment Variables

| Variable                 | Purpose                            | Default                       |
| :----------------------- | :--------------------------------- | :---------------------------- |
| `CORTEX_BUNDLE_PASSWORD` | Passphrase for encrypted bundles   | None (Mandatory if encrypted) |
| `CORTEX_KV_CACHE_GB`     | RAM limit for the KV Cache Manager | `4`                           |
| `PYTHONPATH`             | Injected into agent environment    | Bundle root                   |
