#!/bin/bash
for i in {1..200}; do
    output=$(cargo test test_integration_complex_dag_execution 2>&1)
    if [ $? -ne 0 ]; then
        echo "=== FAILED on iteration $i ==="
        echo "$output" | tail -100
        exit 1
    fi
    echo "Iteration $i: PASSED"
done
echo "All 200 iterations passed"
