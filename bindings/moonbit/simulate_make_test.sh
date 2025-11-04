#!/bin/bash

# Simulate what make test does
LIB_PATH="../c/target/release/deps:../c/target/release"

echo "Current directory: $(pwd)"
echo "Setting DYLD_LIBRARY_PATH=$LIB_PATH"

cd tests/behavior_tests
DYLD_LIBRARY_PATH=$LIB_PATH moon test --target native
