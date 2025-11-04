#!/bin/bash

# Build the test
cd tests/behavior_tests
export DYLD_LIBRARY_PATH=../../../c/target/release/deps:../../../c/target/release
moon test --target native --build-only

# Run with lldb to catch the crash
TEST_EXE="target/native/debug/test/behavior_tests.blackbox_test.exe"

if [ -f "$TEST_EXE" ]; then
    echo "Running test with lldb..."
    lldb -o "run" -o "bt" -o "quit" -- "$TEST_EXE"
else
    echo "Test executable not found: $TEST_EXE"
fi
