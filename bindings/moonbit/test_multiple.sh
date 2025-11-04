#!/bin/bash

cd tests/behavior_tests
export DYLD_LIBRARY_PATH=../../../c/target/release/deps:../../../c/target/release

success=0
fail=0

for i in 1 2 3 4 5 6 7 8 9 10; do
    echo "=== Run $i ==="
    if moon test --target native 2>&1 | grep -q "Total tests: 14, passed: 14"; then
        echo "✓ SUCCESS"
        success=$((success + 1))
    else
        echo "✗ FAILED"
        fail=$((fail + 1))
    fi
done

echo ""
echo "Results: $success successes, $fail failures"
