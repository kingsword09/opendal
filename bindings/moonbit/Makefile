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

.PHONY: build build-all check test test-memory test-fs clean clean-all help build-c-lib format example config package publish publish-dry-run

# Detect current OS and architecture
OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
ARCH := $(shell uname -m)

# Normalize architecture names
ifeq ($(ARCH),x86_64)
    ARCH := amd64
endif
ifeq ($(ARCH),aarch64)
    ARCH := arm64
endif

# Define file extensions and environment variables for different OS
ifeq ($(OS),linux)
    LIB_EXT := so
    LIB_PATH_VAR := LD_LIBRARY_PATH
else ifeq ($(OS),darwin)
    LIB_EXT := dylib
    LIB_PATH_VAR := DYLD_LIBRARY_PATH
else
    LIB_EXT := dll
    LIB_PATH_VAR := PATH
    OS := windows
endif

# Default backend for testing
OPENDAL_TEST ?= memory

# Service name (convert underscores to hyphens for cargo features)
SERVICE := $(OPENDAL_TEST)
SERVICE_FEATURE := $(shell echo $(SERVICE) | tr '_' '-')

# Library paths
C_LIB_DIR := ../c/target/release
C_LIB_DEPS_DIR := ../c/target/release/deps
C_LIB_FILE := $(C_LIB_DIR)/libopendal_c.$(LIB_EXT)

# Set library path based on OS
ifeq ($(OS),windows)
    LIB_PATH := $(C_LIB_DIR);$(C_LIB_DEPS_DIR)
else
    LIB_PATH := $(C_LIB_DEPS_DIR):$(C_LIB_DIR)
endif

# Colors for output
COLOR_RESET := \033[0m
COLOR_BOLD := \033[1m
COLOR_RED := \033[31m
COLOR_GREEN := \033[32m
COLOR_YELLOW := \033[33m
COLOR_BLUE := \033[34m

# Default target
.DEFAULT_GOAL := help

# Help target
help:
	@echo "$(COLOR_BOLD)OpenDAL MoonBit Bindings - Makefile$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_BOLD)Available targets:$(COLOR_RESET)"
	@echo "  $(COLOR_GREEN)build-all$(COLOR_RESET)    - Build C library and MoonBit library (recommended)"
	@echo "  $(COLOR_GREEN)build$(COLOR_RESET)        - Build the MoonBit library only (static library)"
	@echo "  $(COLOR_GREEN)build-c-lib$(COLOR_RESET)  - Build the C library only"
	@echo "  $(COLOR_GREEN)check$(COLOR_RESET)        - Check the MoonBit code"
	@echo "  $(COLOR_GREEN)format$(COLOR_RESET)       - Format the MoonBit code"
	@echo "  $(COLOR_GREEN)test$(COLOR_RESET)         - Run tests with specified backend (default: memory)"
	@echo "  $(COLOR_GREEN)test-memory$(COLOR_RESET)  - Run tests with memory backend"
	@echo "  $(COLOR_GREEN)test-fs$(COLOR_RESET)      - Run tests with filesystem backend"
	@echo "  $(COLOR_GREEN)clean$(COLOR_RESET)        - Clean MoonBit generated files"
	@echo "  $(COLOR_GREEN)clean-all$(COLOR_RESET)    - Clean all files including C library"
	@echo "  $(COLOR_GREEN)config$(COLOR_RESET)       - Show current configuration"
	@echo "  $(COLOR_GREEN)package$(COLOR_RESET)      - Package the module"
	@echo "  $(COLOR_GREEN)publish-dry-run$(COLOR_RESET) - Test publish without actually publishing"
	@echo "  $(COLOR_GREEN)publish$(COLOR_RESET)      - Publish to mooncakes.io"
	@echo "  $(COLOR_GREEN)help$(COLOR_RESET)         - Show this help message"
	@echo ""
	@echo "$(COLOR_BOLD)Environment variables:$(COLOR_RESET)"
	@echo "  $(COLOR_YELLOW)OPENDAL_TEST$(COLOR_RESET)     - Backend to test (default: memory)"
	@echo "                     Supported: memory, fs, s3, azblob, gcs, etc."
	@echo ""
	@echo "$(COLOR_BOLD)Platform information:$(COLOR_RESET)"
	@echo "  OS:              $(OS)"
	@echo "  Architecture:    $(ARCH)"
	@echo "  Library ext:     $(LIB_EXT)"
	@echo "  Lib path var:    $(LIB_PATH_VAR)"
	@echo ""
	@echo "$(COLOR_BOLD)Example usage:$(COLOR_RESET)"
	@echo "  make test                    # Test with memory backend"
	@echo "  OPENDAL_TEST=fs make test    # Test with filesystem backend"
	@echo "  OPENDAL_TEST=s3 make test    # Test with S3 backend"

# Check if C library exists, build if needed
$(C_LIB_FILE):
	@echo "$(COLOR_YELLOW)C library not found, building...$(COLOR_RESET)"
	@$(MAKE) build-c-lib

# Build C library with specified service
build-c-lib:
	@echo "$(COLOR_BLUE)Building C library for service: $(SERVICE)$(COLOR_RESET)"
	@if [ "$(SERVICE)" = "memory" ]; then \
		echo "$(COLOR_YELLOW)Note: Building with default features (includes memory backend)$(COLOR_RESET)"; \
		cd ../c && cargo build --release; \
	else \
		echo "$(COLOR_YELLOW)Note: C bindings use core's default features. Building without specific service feature.$(COLOR_RESET)"; \
		cd ../c && cargo build --release; \
	fi
	@echo "$(COLOR_GREEN)✓ C library built successfully$(COLOR_RESET)"

# Check MoonBit code
check:
	@echo "$(COLOR_BLUE)Checking MoonBit code...$(COLOR_RESET)"
	@moon check --target native
	@echo "$(COLOR_GREEN)✓ Code check passed$(COLOR_RESET)"

# Build both C library and MoonBit library
build-all: build-c-lib build
	@echo ""
	@echo "$(COLOR_GREEN)✓ All libraries built successfully!$(COLOR_RESET)"
	@echo "  - C library: $(C_LIB_FILE)"
	@echo "  - MoonBit library: target/native/release/build/liblib.a"

# Build MoonBit library (library only, linking will fail but library is built)
build:
	@echo "$(COLOR_BLUE)Building MoonBit library...$(COLOR_RESET)"
	@moon build --target native > /dev/null 2>&1 || true
	@if [ -f target/native/release/build/liblib.a ]; then \
		echo "$(COLOR_GREEN)✓ Library built successfully (liblib.a)$(COLOR_RESET)"; \
	else \
		echo "$(COLOR_RED)✗ Library build failed$(COLOR_RESET)"; \
		exit 1; \
	fi

# Format MoonBit code
format:
	@echo "$(COLOR_BLUE)Formatting MoonBit code...$(COLOR_RESET)"
	@moon fmt
	@echo "$(COLOR_GREEN)✓ Code formatted$(COLOR_RESET)"

# Run tests with specified backend
test: $(C_LIB_FILE)
	@echo "$(COLOR_BOLD)Running tests with backend: $(SERVICE)$(COLOR_RESET)"
	@echo "$(COLOR_BLUE)Setting $(LIB_PATH_VAR)=$(LIB_PATH)$(COLOR_RESET)"
	@cd tests/behavior_tests && \
		$(LIB_PATH_VAR)=$(LIB_PATH) moon test --target native
	@echo ""
	@echo "$(COLOR_GREEN)✓ Tests completed successfully!$(COLOR_RESET)"

# Convenience target for memory backend
test-memory:
	@$(MAKE) test OPENDAL_TEST=memory

# Convenience target for filesystem backend
test-fs:
	@$(MAKE) test OPENDAL_TEST=fs

# Clean all generated files
clean:
	@echo "$(COLOR_BLUE)Cleaning generated files...$(COLOR_RESET)"
	moon clean
	rm -rf target
	rm -rf tests/behavior_tests/target
	rm -rf examples/target
	@echo "$(COLOR_GREEN)✓ Clean completed$(COLOR_RESET)"

# Clean everything including C library
clean-all: clean
	@echo "$(COLOR_BLUE)Cleaning C library...$(COLOR_RESET)"
	cd ../c && cargo clean
	@echo "$(COLOR_GREEN)✓ Full clean completed$(COLOR_RESET)"

# Show current configuration
config:
	@echo "$(COLOR_BOLD)Current Configuration:$(COLOR_RESET)"
	@echo "  OPENDAL_TEST:    $(OPENDAL_TEST)"
	@echo "  SERVICE:         $(SERVICE)"
	@echo "  SERVICE_FEATURE: $(SERVICE_FEATURE)"
	@echo "  OS:              $(OS)"
	@echo "  ARCH:            $(ARCH)"
	@echo "  LIB_EXT:         $(LIB_EXT)"
	@echo "  LIB_PATH_VAR:    $(LIB_PATH_VAR)"
	@echo "  LIB_PATH:        $(LIB_PATH)"
	@echo "  C_LIB_FILE:      $(C_LIB_FILE)"

# Package the module
package: check format build test
	@echo "$(COLOR_BLUE)Packaging module...$(COLOR_RESET)"
	moon package
	@echo "$(COLOR_GREEN)✓ Package created$(COLOR_RESET)"

# Publish to mooncakes.io (dry run)
publish-dry-run:
	@echo "$(COLOR_BLUE)Dry run publish...$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Current version: $(shell grep version moon.mod.json | head -1 | cut -d'"' -f4)$(COLOR_RESET)"
	moon publish --dry-run
	@echo "$(COLOR_GREEN)✓ Dry run completed$(COLOR_RESET)"

# Publish to mooncakes.io
publish: check format build test
	@echo "$(COLOR_BOLD)Publishing to mooncakes.io$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Current version: $(shell grep version moon.mod.json | head -1 | cut -d'"' -f4)$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_YELLOW)⚠️  This will publish the package to mooncakes.io$(COLOR_RESET)"
	@read -p "Continue? [y/N] " confirm && [ "$$confirm" = "y" ] || (echo "$(COLOR_RED)Cancelled$(COLOR_RESET)" && exit 1)
	@echo ""
	moon publish
	@echo ""
	@echo "$(COLOR_GREEN)✓ Published successfully!$(COLOR_RESET)"
