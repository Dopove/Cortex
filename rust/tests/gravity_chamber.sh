#!/bin/bash
BIN="./target/release/cortex"
BUNDLE="cuda_mock.cortex"
LOG="memory_stability.log"
OUTPUT_LOG="gravity_chamber_output.log"
END_TIME=$((SECONDS + 3600))

echo "Timestamp,RSS_KB" > $LOG
echo "Starting 1-hour stability test at $(date)" > $OUTPUT_LOG

ITER=0
while [ $SECONDS -lt $END_TIME ]; do
    ITER=$((ITER + 1))
    echo "Iteration $ITER starting..." >> $OUTPUT_LOG
    
    # Run in background
    $BIN turbo $BUNDLE >> $OUTPUT_LOG 2>&1 &
    PID=$!
    
    # Capture RSS once during execution (since it's short)
    sleep 2
    RSS=$(ps -o rss= -p $PID 2>/dev/null | tr -d ' ')
    if [ ! -z "$RSS" ]; then
        echo "$(date +%H:%M:%S),$RSS" >> $LOG
    fi
    
    wait $PID
    EXIT_CODE=$?
    if [ $EXIT_CODE -ne 0 ]; then
        echo "CRITICAL: Process failed with exit code $EXIT_CODE at iter $ITER" >> $OUTPUT_LOG
        # Don't exit, just continue to see if it's a one-off
    fi
    echo "Iteration $ITER complete." >> $OUTPUT_LOG
done
echo "Stability test complete at $(date)" >> $OUTPUT_LOG
