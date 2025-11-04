#!/bin/bash

cd tests/behavior_tests
export DYLD_LIBRARY_PATH=../../../c/target/release/deps:../../../c/target/release

for i in 1 2 3; do
    echo "=========================================="
    echo "Run $i"
    echo "=========================================="
    moon test --target native 2>&1
    echo ""
done
