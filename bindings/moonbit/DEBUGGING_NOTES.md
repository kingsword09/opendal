# OpenDAL MoonBit Bindings - Debugging Notes

## Issue: Intermittent SIGSEGV in Tests

### Symptoms
- Tests fail randomly with SIGSEGV (signal 11)
- Success rate: approximately 50%
- Failure occurs during `op.delete()` calls
- Error message: `The test executable exited with signal: 11 (SIGSEGV)`

### Root Cause Analysis

#### Stack Trace
```
frame #0: tokio::util::rand::rt::RngSeedGenerator::next_seed
frame #1: tokio::runtime::context::runtime::enter_runtime
frame #2: opendal::blocking::operator::Operator::delete
frame #3: opendal_operator_delete
frame #4: moonbit_opendal_operator_delete (wrapper.c:150)
frame #5: Operator::delete (MoonBit code)
```

#### Root Cause
The crash occurs in the Tokio runtime's random number generator initialization. The issue is related to:

1. **Global Tokio Runtime**: OpenDAL C bindings use a global `LazyLock<tokio::runtime::Runtime>` (see `bindings/c/src/operator.rs:28`)

2. **Thread-Local Storage (TLS) Corruption**: When tests run in quick succession, Tokio's thread-local RNG state can become corrupted or point to invalid memory

3. **Race Condition**: The crash is intermittent because it depends on:
   - Timing of test execution
   - State of Tokio's internal TLS
   - Memory layout at runtime

#### Evidence
- Crash address: `0x4652a09040208` (invalid memory)
- Crash location: `ldapr x0, [x0]` - attempting to load from corrupted pointer
- Occurs specifically in `RngSeedGenerator::next_seed` which accesses TLS

### Potential Solutions

#### Option 1: Fix in OpenDAL C Bindings (Upstream)
The issue should be reported to OpenDAL project. Potential fixes:
- Use `OnceCell` instead of `LazyLock` for runtime initialization
- Implement proper TLS cleanup
- Add runtime guards to prevent premature destruction

#### Option 2: Workaround in MoonBit Tests
- Add delays between tests
- Ensure proper cleanup order
- Use a single Operator instance across tests (not recommended)

#### Option 3: Rebuild C Library with Different Tokio Configuration
- Try single-threaded runtime instead of multi-threaded
- Disable certain Tokio features

### Recommended Action
1. Report this issue to Apache OpenDAL project
2. Include stack trace and reproduction steps
3. Mention it's related to the global LazyLock runtime and TLS

### Related Code Locations
- `bindings/c/src/operator.rs:28-33` - Global RUNTIME definition
- `bindings/c/src/operator.rs:94-96` - Runtime handle acquisition
- `bindings/moonbit/src/wrapper.c:146-159` - delete wrapper
- `bindings/moonbit/src/ffi.mbt:59-63` - FFI binding for delete

### Reproduction
Run `make test` multiple times. Approximately 50% will fail with SIGSEGV.

```bash
for i in {1..10}; do make test; done
```
