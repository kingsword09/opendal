#!/bin/bash

success_count=0
fail_count=0

for i in 1 2 3 4 5 6 7 8 9 10; do
    echo "========== Test Run $i =========="
    if make test > /tmp/test_run_$i.log 2>&1; then
        echo "✓ Run $i: SUCCESS"
        success_count=$((success_count + 1))
    else
        echo "✗ Run $i: FAILED"
        fail_count=$((fail_count + 1))
        tail -10 /tmp/test_run_$i.log
    fi
    echo ""
done

echo "=========================================="
echo "Summary: $success_count successes, $fail_count failures out of 10 runs"
echo "=========================================="
