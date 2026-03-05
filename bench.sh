#!/bin/bash
set -e

echo "================================================="
echo " Cortex V3 Benchmark: Multi-Framework Validation "
echo "================================================="

echo "[0] Compiling cortex-cli release..."
cargo build --manifest-path rust/Cargo.toml --release -p cortex-cli
CORTEX_BIN="./rust/target/release/cortex"

# Create log file
LOGFILE="examples/benchmark_results.log"
echo "CORTEX V3 BENCHMARK RESULTS" > $LOGFILE
echo "=========================" >> $LOGFILE

declare -a frameworks=("autogen-agent" "crewai-agent" "langgraph-agent" "smolagents-agent" "custom-agent" "bloom-agent")

for fw in "${frameworks[@]}"
do
    echo "------------------------------------------------"
    echo "Benchmarking: $fw"
    echo "------------------------------------------------"
    
    # Clean previous if exists
    rm -f examples/$fw.cortex
    
    # Bundle the agent
    echo "[1] Building Bundle..."
    $CORTEX_BIN build examples/$fw examples/$fw.cortex
    
    echo "[2] Running and collecting Metrics..."
    # We use /usr/bin/time to get RSS memory and execution time
    CORTEX_BYPASS_MEM_CHECK=1 /usr/bin/time -v $CORTEX_BIN run examples/$fw.cortex > temp_$fw.out 2> temp_$fw.err
    
    # Parse max RSS and elapsed time
    RSS=$(grep "Maximum resident set size" temp_$fw.err | awk '{print $6}')
    TIME=$(grep "Elapsed (wall clock) time" temp_$fw.err | awk '{print $8}')
    
    echo "$fw | Peak Memory: ${RSS}KB | Exec Time: $TIME" | tee -a $LOGFILE
    
    rm -f temp_$fw.out temp_$fw.err
done

echo "================================================="
echo "Benchmarking Complete. See examples/benchmark_results.log"
echo "================================================="
