#!/bin/bash

cd tests/behavior_tests
export DYLD_LIBRARY_PATH=../../../c/target/release/deps:../../../c/target/release

# Build first
moon test --target native --build-only 2>&1 > /dev/null

TEST_EXE="target/native/debug/test/behavior_tests.blackbox_test.exe"

# Try to trigger a crash
for i in 1 2 3 4 5 6 7 8 9 10; do
    echo "Attempt $i..."

    # Run the test
    if ! $TEST_EXE "write_test.mbt:0-4/stat_test.mbt:0-3/read_test.mbt:0-4/delete_test.mbt:0-3" > /tmp/test_output.txt 2>&1; then
        echo "CRASH DETECTED! Running with lldb..."

        # Create lldb command file
        cat > /tmp/lldb_commands.txt <<EOF
run write_test.mbt:0-4/stat_test.mbt:0-3/read_test.mbt:0-4/delete_test.mbt:0-3
bt
frame info
quit
EOF

        lldb -s /tmp/lldb_commands.txt -- "$TEST_EXE"
        break
    else
        echo "Run $i: OK"
    fi
done
