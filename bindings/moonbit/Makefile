# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

.PHONY: build test clean help build-rust

# Colors for output
COLOR_RESET := \033[0m
COLOR_BOLD := \033[1m
COLOR_GREEN := \033[32m
COLOR_BLUE := \033[34m
COLOR_YELLOW := \033[33m

# Default target
.DEFAULT_GOAL := help

# Help target
help:
	@echo "$(COLOR_BOLD)OpenDAL MoonBit Bindings - Makefile$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_BOLD)Available targets:$(COLOR_RESET)"
	@echo "  $(COLOR_GREEN)build$(COLOR_RESET)       - Build the MoonBit library"
	@echo "  $(COLOR_GREEN)build-rust$(COLOR_RESET)  - Build the Rust FFI library"
	@echo "  $(COLOR_GREEN)build-all$(COLOR_RESET)   - Build both Rust FFI and MoonBit library"
	@echo "  $(COLOR_GREEN)test$(COLOR_RESET)        - Run tests"
	@echo "  $(COLOR_GREEN)clean$(COLOR_RESET)       - Clean generated files"
	@echo "  $(COLOR_GREEN)help$(COLOR_RESET)        - Show this help message"

# Build Rust FFI library
build-rust:
	@echo "$(COLOR_BLUE)Building Rust FFI library...$(COLOR_RESET)"
	@cd rust-ffi && cargo build --release
	@echo "$(COLOR_GREEN)✓ Rust FFI library built successfully$(COLOR_RESET)"

# Build MoonBit library
build: build-rust
	@echo "$(COLOR_BLUE)Building MoonBit library...$(COLOR_RESET)"
	@moon build --target native
	@echo "$(COLOR_GREEN)✓ MoonBit library built successfully$(COLOR_RESET)"

# Build both
build-all: build-rust build
	@echo ""
	@echo "$(COLOR_GREEN)✓ All libraries built successfully!$(COLOR_RESET)"

# Run tests
test: build-rust
	@echo "$(COLOR_BOLD)Running tests...$(COLOR_RESET)"
	@cd tests/behavior_tests && moon test --target native
	@echo ""
	@echo "$(COLOR_GREEN)✓ Tests completed successfully!$(COLOR_RESET)"

# Clean all generated files
clean:
	@echo "$(COLOR_BLUE)Cleaning generated files...$(COLOR_RESET)"
	@moon clean
	@rm -rf target
	@rm -rf tests/behavior_tests/target
	@cd rust-ffi && cargo clean
	@echo "$(COLOR_GREEN)✓ Clean completed$(COLOR_RESET)"
