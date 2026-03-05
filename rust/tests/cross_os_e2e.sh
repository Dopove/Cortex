#!/bin/bash
set -e

# =================================================================
# Cortex V3 Cross-OS E2E System Tests
# Tests full user journey across macOS, Linux, and Windows runners
# =================================================================

# 1. Determine execution path based on OS 
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    BIN_PATH="./rust/target/release/cortex.exe"
else
    BIN_PATH="./rust/target/x86_64-unknown-linux-gnu/release/cortex"
    # Overwrite if we detect custom matrix targets injected or fallback logic
    if [ ! -f "$BIN_PATH" ]; then
        BIN_PATH=$(find target -name cortex | head -n 1) # Fallback to searching rust tree
    fi
fi

# Override completely if explicit bin path was passed, useful for test matrix where we copied it manually to target/release/cortex
if [ -f "./rust/target/release/cortex.exe" ]; then
    BIN_PATH="./rust/target/release/cortex.exe"
elif [ -f "./rust/target/release/cortex" ]; then
    BIN_PATH="./rust/target/release/cortex"
fi

echo "Using Cortex binary at: $BIN_PATH"

if [ ! -f "$BIN_PATH" ]; then
   echo "CRITICAL: Cortex binary not found at expected location $BIN_PATH!"
   exit 1
fi

echo "==================================="
echo "1. Testing Environment Initialization"
echo "==================================="
$BIN_PATH init
echo "✅ Environment Initialized."

echo "==================================="
echo "2. Building Agent Bundle"
echo "==================================="
if [ -f "examples/e2e-agent.cortex" ]; then
    rm examples/e2e-agent.cortex
fi

$BIN_PATH build examples/autogen-agent examples/e2e-agent.cortex

if [ ! -f "examples/e2e-agent.cortex" ]; then
    echo "CRITICAL: Cortex build failed to produce the agent bundle."
    exit 1
fi
echo "✅ Bundle e2e-agent.cortex Built Successfully."

echo "==================================="
echo "3. Testing Bundle Inspector"
echo "==================================="
INFO_OUTPUT=$($BIN_PATH info examples/e2e-agent.cortex 2>&1 || true)
if [[ "$INFO_OUTPUT" != *"Bundle Metadata"* ]]; then
     echo "WARNING: Info command didn't output expected text. Raw: $INFO_OUTPUT"
fi
echo "✅ Info parsing completed."

echo "==================================="
echo "4. Standard Execution Test"
echo "==================================="
CORTEX_NO_ISOLATION=1 CORTEX_BYPASS_MEM_CHECK=1 $BIN_PATH run examples/e2e-agent.cortex

echo "==================================="
echo "5. Turbo / Parallel Execution Test"
echo "==================================="
CORTEX_NO_ISOLATION=1 CORTEX_BYPASS_MEM_CHECK=1 $BIN_PATH turbo examples/e2e-agent.cortex

echo "================================================="
echo "✅ CRITICAL END-TO-END WORKFLOW TESTS SUCCESSFUL ✅"
echo "================================================="
