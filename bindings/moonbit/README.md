# Apache OpenDAL™ MoonBit Binding

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Status](https://img.shields.io/badge/status-working-brightgreen)](README.md)

> **✅ Working**: Basic operations are fully functional! All tests passing.

MoonBit bindings for [Apache OpenDAL](https://opendal.apache.org/), providing unified access to various storage services.

## Overview

OpenDAL MoonBit binding allows you to use OpenDAL's powerful storage abstraction layer in MoonBit applications. It provides a type-safe, idiomatic MoonBit API that wraps OpenDAL's C bindings.

## Status

### ✅ Implemented and Working

- ✅ Operator creation and lifecycle management
- ✅ Write operation
- ✅ Read operation
- ✅ Delete operation
- ✅ Exists check
- ✅ Stat operation (metadata)
- ✅ Error handling with Result types
- ✅ All tests passing

### 🚧 Not Yet Implemented

- ⚠️ Configuration options (only default configs work)
- ⚠️ List operations

### ⚠️ Note: Synchronous API Only

- ✅ Synchronous (blocking) operations
- ❌ Async API removed (see [ASYNC_REMOVAL_SUMMARY.md](ASYNC_REMOVAL_SUMMARY.md) for details)
- 💡 Reason: OpenDAL C bindings are synchronous, MoonBit async is single-threaded
- 📖 For concurrency patterns, wait for future MoonBit multi-threading support

## Features

- **Unified API**: Access different storage services through a single interface
- **Type Safety**: Leverages MoonBit's type system for safer storage operations
- **Result-based Error Handling**: Idiomatic error handling with Result types
- **Direct FFI**: Efficient C bindings for optimal performance
- **Synchronous API**: Simple, blocking operations that are easy to understand and use

## Quick Reference

```bash
# Build everything (C library + MoonBit library) - Recommended
cd bindings/moonbit && make build-all

# Or build separately
cd bindings/c && cargo build --release
cd bindings/moonbit && make build

# Run tests
make test                    # Memory backend (default)
OPENDAL_TEST=fs make test    # Filesystem backend
OPENDAL_TEST=s3 make test    # S3 backend

# Other commands
make check                   # Check code
make format                  # Format code
make clean                   # Clean generated files
```

## Prerequisites

To build and use OpenDAL MoonBit bindings, you need:

- **MoonBit compiler**: Install from [moonbitlang.com](https://www.moonbitlang.com/)
- **Rust toolchain**: Required to build the OpenDAL C library (install from [rustup.rs](https://rustup.rs/))
- **C compiler**: clang or gcc (usually pre-installed on macOS/Linux)

## Installation

### 1. Build OpenDAL C Bindings

The C library is built automatically when you run `make build-all` or `make test`, but you can also build it manually:

```bash
cd bindings/c
cargo build --release
```

**Note**: The C bindings use OpenDAL core's default features, which include the `memory` backend. The C bindings don't expose service-specific features directly - all backends are available through the unified API at runtime.

This will create the OpenDAL C library in `bindings/c/target/release/`:
- **Linux**: `libopendal_c.so` (dynamic) and `libopendal_c.a` (static)
- **macOS**: `libopendal_c.dylib` (dynamic) and `libopendal_c.a` (static)
- **Windows**: `opendal_c.dll` (dynamic) and `opendal_c.lib` (static)

### 2. Build MoonBit Bindings

**Recommended: Build everything at once**

```bash
cd bindings/moonbit
make build-all
```

This will:
1. Build the C library with the specified backend (default: memory)
2. Build the MoonBit static library (`liblib.a`)

**Or build MoonBit library only:**

```bash
cd bindings/moonbit
make build
```

This builds only the MoonBit static library. The MoonBit bindings use relative paths and `-lopendal_c` flag, which automatically finds the correct library for your platform.

## Quick Start

Here's a simple example using the memory backend:

```moonbit
fn main {
  // Create an operator for memory storage
  let op = @opendal/lib.Operator::new("memory")
  defer op.free()  // Automatic cleanup

  // Write data
  let data = Bytes::from_array([b'H', b'e', b'l', b'l', b'o'])
  match op.write("hello.txt", data) {
    Ok(_) => println("Write successful")
    Err(e) => println("Write failed: " + e)
  }

  // Read data
  match op.read("hello.txt") {
    Ok(bytes) => println("Read " + bytes.length().to_string() + " bytes")
    Err(e) => println("Read failed: " + e)
  }

  // Check if file exists
  match op.exists("hello.txt") {
    Ok(true) => println("File exists")
    Ok(false) => println("File does not exist")
    Err(e) => println("Check failed: " + e)
  }

  // Delete file
  match op.delete("hello.txt") {
    Ok(_) => println("Delete successful")
    Err(e) => println("Delete failed: " + e)
  }
}
```

## Building and Testing

### Check Code

```bash
make check
```

This validates the MoonBit code without building executables.

### Build Library

```bash
make build
```

This builds the MoonBit static library (`target/native/release/build/liblib.a`).

### Format Code

```bash
make format
```

This formats all MoonBit source files according to the standard style.

### Run Tests

Behavior tests are located in `tests/behavior_tests/` directory, organized by operation type:
- `write_test.mbt` - Write operation tests
- `read_test.mbt` - Read operation tests
- `delete_test.mbt` - Delete operation tests
- `stat_test.mbt` - Stat and exists operation tests

**Test Coverage:**
- 14 synchronous tests
- All tests use standard MoonBit `test "name" { }` syntax
- **Success rate: 100% (14/14 passing)**

**Running Tests:**

The Makefile automatically builds the required C library with the appropriate backend before running tests.

```bash
# Run tests with default backend (memory)
make test

# Run tests with filesystem backend
OPENDAL_TEST=fs make test

# Run tests with S3 backend
OPENDAL_TEST=s3 make test

# Convenience commands
make test-memory  # Test with memory backend
make test-fs      # Test with filesystem backend
```

**Manual Testing (Advanced):**

If you need to run tests manually without the Makefile:

```bash
# First, ensure the C library is built with the desired backend
cd ../c
cargo build --release --features "services-memory"
cd ../moonbit

# Then run tests with the correct library path
cd tests/behavior_tests

# macOS
DYLD_LIBRARY_PATH=../../../c/target/release/deps:../../../c/target/release moon test --target native

# Linux
LD_LIBRARY_PATH=../../../c/target/release/deps:../../../c/target/release moon test --target native

# Windows (PowerShell)
$env:PATH="../../../c/target/release;../../../c/target/release/deps;$env:PATH"
moon test --target native
```

**Note**: The library path must include both `release` and `release/deps` directories, with `deps` first, because Rust places the actual dynamic library in the `deps` subdirectory.

**Test Results:**
```
Total tests: 14, passed: 14, failed: 0
```

## Cross-Platform Support

The MoonBit bindings work on **Linux, macOS, and Windows** with the same configuration:

### Library Linking

The configuration uses `-lopendal_c` which automatically finds the correct library:
- **Linux**: Links to `libopendal_c.so` (dynamic library)
- **macOS**: Links to `libopendal_c.dylib` (dynamic library)
- **Windows**: Links to `opendal_c.dll` or `opendal_c.lib`

### Platform-Specific Library Paths

The Makefile automatically sets the correct library path for your platform:
- **macOS**: Uses `DYLD_LIBRARY_PATH`
- **Linux**: Uses `LD_LIBRARY_PATH`
- **Windows**: Uses `PATH`

**Note**: The library path must include both `release` and `release/deps` directories because Rust places the actual dynamic library in the `deps` subdirectory. **The `deps` directory should come first** in the path.

## Supported Backends

Currently, only the `memory` backend is tested and working with default configuration:

- **Memory** (`memory`): In-memory storage (fully tested)

Other backends like `fs`, `s3`, `azblob`, etc. should work but require configuration options which are not yet implemented in these bindings.

## API Reference

### Operator

The main entry point for all operations.

#### Methods

- `pub fn Operator::new(scheme: String) -> Operator`
  - Create a new operator with the specified backend (default configuration)

- `pub fn Operator::write(self: Operator, path: String, data: Bytes) -> Result[Unit, String]`
  - Write data to the specified path

- `pub fn Operator::read(self: Operator, path: String) -> Result[Bytes, String]`
  - Read data from the specified path

- `pub fn Operator::delete(self: Operator, path: String) -> Result[Unit, String]`
  - Delete the object at the specified path

- `pub fn Operator::stat(self: Operator, path: String) -> Result[Metadata, String]`
  - Get metadata for the object at the specified path

- `pub fn Operator::exists(self: Operator, path: String) -> Result[Bool, String]`
  - Check if an object exists at the specified path

- `pub fn Operator::free(self: Operator) -> Unit`
  - Free the operator resources (must be called when done)

### Metadata

Contains information about a file or directory.

#### Methods

- `pub fn Metadata::content_length(self: Metadata) -> UInt64` - Get the content length
- `pub fn Metadata::is_file(self: Metadata) -> Bool` - Check if this is a file
- `pub fn Metadata::is_dir(self: Metadata) -> Bool` - Check if this is a directory
- `pub fn Metadata::free(self: Metadata) -> Unit` - Free metadata resources

## Examples

The test suite in `tests/behavior_tests/` serves as comprehensive examples of all operations. Each test file demonstrates specific functionality:

- `write_test.mbt` - Examples of writing data
- `read_test.mbt` - Examples of reading data
- `delete_test.mbt` - Examples of deleting objects
- `stat_test.mbt` - Examples of checking metadata and existence

## Architecture

The MoonBit binding uses a three-layer architecture:

```
┌─────────────────────────────────┐
│   MoonBit Application Code      │
├─────────────────────────────────┤
│   MoonBit API (lib.mbt)         │
│   - Operator, Metadata          │
│   - Result-based error handling │
├─────────────────────────────────┤
│   C Wrapper (wrapper.c)         │
│   - Simplified C helpers        │
│   - Buffer-based I/O            │
├─────────────────────────────────┤
│   OpenDAL C Bindings            │
│   - libopendal_c                │
├─────────────────────────────────┤
│   OpenDAL Rust Core             │
└─────────────────────────────────┘
```

### Key Design Decisions

1. **Buffer-Based I/O**: Read operations use pre-allocated buffers to avoid complex pointer management
2. **Manual Resource Management**: Operators and Metadata must be explicitly freed
3. **Type Safety**: MoonBit's type system with Result types for error handling
4. **Simplified C Layer**: C wrapper functions handle complex struct returns and error codes
5. **FFI Best Practices**: Following [MoonBit official async library](https://github.com/moonbitlang/async) patterns:
   - `#borrow` annotations for read-only parameters
   - Simple return types (Int for error codes)
   - Clear naming conventions with project prefix
   - See [FFI_BEST_PRACTICES.md](FFI_BEST_PRACTICES.md) for details

## Makefile Commands

The project includes a comprehensive Makefile for common tasks:

```bash
make help            # Show all available commands
make build-all       # Build C library and MoonBit library (recommended)
make build-c-lib     # Build the C library only
make build           # Build the MoonBit static library only
make check           # Check MoonBit code for errors
make format          # Format MoonBit code
make test            # Run tests (default: memory backend)
make test-memory     # Run tests with memory backend
make test-fs         # Run tests with filesystem backend
make clean           # Clean generated files
make clean-all       # Clean all files including C library
make config          # Show current configuration
make package         # Package the module for publishing
make publish-dry-run # Test publish without actually publishing
make publish         # Publish to mooncakes.io
```

**Environment Variables:**

- `OPENDAL_TEST` - Specify the backend to test (default: `memory`)
  - Supported values: `memory`, `fs`, `s3`, `azblob`, `gcs`, etc.

**Examples:**

```bash
# Test with filesystem backend
OPENDAL_TEST=fs make test

# Test with S3 backend
OPENDAL_TEST=s3 make test
```

## Development

### Project Structure

Following [MoonBit async library](https://github.com/moonbitlang/async) structure:

```
bindings/moonbit/
├── src/                      # Library source code
│   ├── ffi.mbt              # FFI declarations
│   ├── types.mbt            # Type definitions
│   ├── operator.mbt         # Operator lifecycle
│   ├── write.mbt            # Write operation
│   ├── read.mbt             # Read operation
│   ├── delete.mbt           # Delete operation
│   ├── stat.mbt             # Stat/Exists operations
│   ├── metadata.mbt         # Metadata implementation
│   ├── wrapper.c            # C wrapper functions
│   └── moon.pkg.json        # Package configuration
├── tests/
│   └── behavior_tests/      # Behavior tests
│       ├── write_test.mbt   # Write operation tests
│       ├── read_test.mbt    # Read operation tests
│       ├── delete_test.mbt  # Delete operation tests
│       ├── stat_test.mbt    # Stat operation tests
│       ├── test_helper.mbt  # Test utilities
│       └── moon.pkg.json    # Test package config
├── moon.mod.json            # Main module configuration
├── Makefile                 # Build automation
├── PUBLISHING.md            # Publishing guide
└── README.md                # This file
```

### Implementation Details

#### String Handling

- MoonBit strings are UTF-8 encoded using `@encoding/utf8.encode()`
- Null terminators are automatically added for C compatibility
- Use `@encoding/utf8.decode()` to convert bytes back to strings

#### Memory Management

- **Operators**: Must call `.free()` when done
- **Metadata**: Must call `.free()` when done
- **Read data**: Automatically managed by MoonBit after copying from C

#### Error Handling

All operations return `Result[T, String]`:
- `Ok(value)` on success
- `Err(message)` on failure with error description

## Troubleshooting

### Library Not Found Error

If you get a "library not found" error when running tests:

1. Ensure the C library is built:
   ```bash
   cd bindings/moonbit
   make build-c-lib
   ```

   Or build manually:
   ```bash
   cd bindings/c
   cargo build --release
   ```

2. Check that the library exists:
   ```bash
   ls -la bindings/c/target/release/libopendal_c.*
   ls -la bindings/c/target/release/deps/libopendal_c.*
   ```

3. Use the Makefile which handles library paths automatically:
   ```bash
   make test
   ```

### Build Errors

If you encounter build errors:

1. Ensure you have the latest MoonBit compiler:
   ```bash
   moon version
   moon upgrade
   ```

2. Clean and rebuild:
   ```bash
   make clean-all
   make build
   ```

### Test Failures

If tests fail:

1. Check that you're using the correct backend:
   ```bash
   make config  # Show current configuration
   ```

2. Ensure the C library is built:
   ```bash
   cd bindings/moonbit
   make build-c-lib
   ```

## Contributing

Contributions are welcome! Please follow the [OpenDAL contributing guidelines](../../CONTRIBUTING.md).

When contributing to the MoonBit binding:

1. Follow MoonBit coding conventions
2. Add tests for new features in `tests/behavior_tests/`
3. Update documentation
4. Ensure all tests pass with `make test`
5. Format code with `make format`
6. Check code with `make check`

## License

Licensed under the Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0

## Resources

- [OpenDAL Documentation](https://opendal.apache.org/)
- [MoonBit Documentation](https://docs.moonbitlang.com/)
- [OpenDAL GitHub](https://github.com/apache/opendal)
- [MoonBit Website](https://www.moonbitlang.com/)

## Acknowledgments

This binding is built on top of:
- [Apache OpenDAL](https://opendal.apache.org/) - The unified data access layer
- [MoonBit](https://www.moonbitlang.com/) - The programming language
- OpenDAL C bindings - The FFI foundation
