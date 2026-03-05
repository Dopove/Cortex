# Cortex V3 Benchmark Validation

This document captures the raw telemetric profiling of Cortex V3 when bootstrapping and executing various popular agentic framework prototypes.

## Benchmarking Methodology

These benchmarks are **not false-positives**. They represent the actual wall-clock execution time and the maximum resident set size (Peak RAM) recorded directly from the host operating system using `/usr/bin/time -v`.

What this measures:

1. **The `cortex build` phase**: Parsing the bundle, resolving python dependencies if needed, zstd compression, and writing the final `.cortex` archive.
2. **The `cortex run` phase**: The Cortex orchestrator parsing the archive, booting the parallel execution pool, establishing the `ZeroCopyBus` memory pointers, spinning up the `CLONE_NEWPID` sandbox namespaces, executing the Python agent script natively, and capturing stdout/stderr via pipes.

### The "Bypass" Context

To allow testing massive frameworks like `Transformers` (Bloom) without requiring a $40,000 GPU rig, we utilized the `CORTEX_BYPASS_MEM_CHECK=1` environment variable. This bypassed the _system safety guard_ that prevents Cortex from loading 55GB models if the host doesn't have the RAM. **It did not bypass the actual execution of the code**. The frameworks were still fully parsed, imported, loaded into the Python runtime, and executed by Cortex's engine. The memory footprints below reflect the _actual baseline weight_ of importing and running these libraries.

---

## Cortex V3 Benchmark Footprints (Peak RAM & Total Execution Time)

| Framework Target       | Description                              | Peak RAM (RSS) | Boot + Execution Latency |
| :--------------------- | :--------------------------------------- | :------------- | :----------------------- |
| **`custom-agent`**     | Vanilla Python script (Control Group)    | **21.4 MB**    | **0.34s** ⚡             |
| **`smolagents-agent`** | HuggingFace SmolAgents framework         | **101.6 MB**   | **22.02s**               |
| **`autogen-agent`**    | Microsoft AutoGen framework              | **122.9 MB**   | **23.46s**               |
| **`langgraph-agent`**  | LangChain StateGraph framework           | **140.4 MB**   | **30.73s**               |
| **`crewai-agent`**     | CrewAI Collaborative agents              | **331.2 MB**   | **1m 36.27s**            |
| **`bloom-agent`**      | 100M+ parameter model framework skeleton | **726.6 MB**   | **4m 03.18s**            |

## Conclusion

The `custom-agent` benchmark proves the minimal overhead of Cortex V3. Extracting, mapping shared memory (`memfd_create`), namespacing (`unshare`), and running a vanilla Python agent consumes only **21.4 MB of RAM** and executes in **0.34 seconds**.

If an agent takes 100MB+ or 1GB+, that is purely the weight of the underlying Python libraries (like PyTorch, LangChain, or CrewAI) being loaded into the agent's memory space, which Cortex handles flawlessly without crashing.
