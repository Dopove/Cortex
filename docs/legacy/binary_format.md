# Cortex Binary Format Specification

## Overview

`.cortex` files use a structured binary format optimized for fast loading and memory-mapped execution.

## Structure

+-------------------+
| Header (128B) | Fixed-size header with magic, version, offsets
+-------------------+
| Manifest | TOML/JSON agent configuration
+-------------------+
| Agent Descriptors | Metadata for each agent
+-------------------+
| Model Data | GGUF models (memory-mappable)
+-------------------+
| Runtime Code | Compiled orchestration logic
+-------------------+
| Checksums | Verification data
+-------------------+



## Header Layout (128 bytes)

See `src/core/format.mojo` for detailed struct definition.
