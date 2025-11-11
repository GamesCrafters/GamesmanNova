#!/bin/bash
ITERATIONS=500
FAILED_COUNT=0

echo "Running $ITERATIONS iterations..."
for i in $(seq 1 $ITERATIONS); do
    if ! cargo test test_integration_complex_dag_execution --quiet 2>&1 > /dev/null; then
        echo "FAILURE on iteration $i"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        cargo test test_integration_complex_dag_execution 2>&1 | tail -50
        echo "---"
    fi
    if [ $((i % 50)) -eq 0 ]; then
        echo "Completed $i iterations, $FAILED_COUNT failures"
    fi
done

echo "=== FINAL RESULTS ==="
echo "Total iterations: $ITERATIONS"
echo "Failures: $FAILED_COUNT"
echo "Success rate: $(( (ITERATIONS - FAILED_COUNT) * 100 / ITERATIONS ))%"
