# OpenDAL MoonBit Bindings

MoonBit bindings for [Apache OpenDAL](https://opendal.apache.org/).

## Features

- ✅ **Stable** - Uses blocking API, no Tokio runtime issues
- ✅ **Simple** - Clean and intuitive API
- ✅ **Fast** - Native performance with Rust FFI
- ✅ **Safe** - Memory-safe with proper resource management

## Architecture

This binding uses a custom Rust FFI layer (`bindings/moonbit-rust`) that:
- Uses OpenDAL's blocking API (no async/Tokio complexity)
- Provides a simple C-compatible interface
- Avoids the TLS issues found in the C bindings

## Quick Start

### Build

```bash
make build-all
```

### Test

```bash
make test
```

### Example

```moonbit
let op = @opendal.Operator::new("memory")
defer op.free()

// Write
let data = Bytes::from_array([b'H', b'e', b'l', b'l', b'o'])
match op.write("hello.txt", data) {
  Ok(_) => println("Write successful")
  Err(e) => println("Write failed: \{e}")
}

// Read
match op.read("hello.txt") {
  Ok(data) => println("Read \{data.length()} bytes")
  Err(e) => println("Read failed: \{e}")
}

// Delete
match op.delete("hello.txt") {
  Ok(_) => println("Delete successful")
  Err(e) => println("Delete failed: \{e}")
}
```

## API Reference

### Operator

- `Operator::new(scheme: String) -> Operator` - Create a new operator
- `Operator::free(self) -> Unit` - Free the operator (use with `defer`)
- `Operator::write(self, path: String, data: Bytes) -> Result[Unit, String]` - Write data
- `Operator::read(self, path: String) -> Result[Bytes, String]` - Read data
- `Operator::delete(self, path: String) -> Result[Unit, String]` - Delete file
- `Operator::stat(self, path: String) -> Result[Metadata, String]` - Get metadata
- `Operator::exists(self, path: String) -> Result[Bool, String]` - Check if exists

### Metadata

- `Metadata::content_length(self) -> UInt64` - Get content length
- `Metadata::is_file(self) -> Bool` - Check if is file
- `Metadata::is_dir(self) -> Bool` - Check if is directory
- `Metadata::free(self) -> Unit` - Free metadata

## Supported Services

Currently supports:
- `memory` - In-memory storage (for testing)

More services can be added by updating `bindings/moonbit-rust/Cargo.toml`.

## Development

### Project Structure

```
bindings/moonbit/
├── src/                    # MoonBit source code
│   ├── ffi.mbt            # FFI declarations
│   ├── types.mbt          # Type definitions
│   ├── operator.mbt       # Operator implementation
│   ├── write.mbt          # Write operations
│   ├── read.mbt           # Read operations
│   ├── delete.mbt         # Delete operations
│   ├── stat.mbt           # Stat operations
│   └── metadata.mbt       # Metadata implementation
├── tests/                 # Tests
│   └── behavior_tests/    # Behavior tests
└── Makefile              # Build automation

bindings/moonbit-rust/     # Rust FFI layer
├── src/
│   └── lib.rs            # Rust FFI implementation
└── Cargo.toml
```

### Adding New Operations

1. Add FFI declaration in `src/ffi.mbt`
2. Implement Rust function in `../moonbit-rust/src/lib.rs`
3. Add MoonBit wrapper in appropriate file (e.g., `src/read.mbt`)
4. Add tests in `tests/behavior_tests/`

## Troubleshooting

### Build Errors

If you encounter build errors:

```bash
# Clean and rebuild
make clean
make build-all
```

### Test Failures

The new Rust FFI approach should have 100% test success rate. If tests fail:

1. Check that the Rust library is built: `ls ../moonbit-rust/target/release/libopendal_moonbit.a`
2. Verify the library path in `src/moon.pkg.json`
3. Run tests with verbose output: `cd tests/behavior_tests && moon test --target native -v`

## License

Licensed under the Apache License, Version 2.0.
