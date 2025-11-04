# Bug Report: Intermittent SIGSEGV in MoonBit Bindings Tests

## Summary
The MoonBit bindings for OpenDAL experience intermittent segmentation faults (SIGSEGV) when running tests. The failure rate is approximately 50%, making it a critical reliability issue.

## Environment
- **OS**: macOS (Darwin 24.6.0)
- **Architecture**: ARM64 (Apple Silicon)
- **MoonBit**: Latest version
- **OpenDAL**: Current main branch
- **Tokio**: Multi-threaded runtime (via OpenDAL C bindings)

## Reproduction Steps
1. Navigate to `bindings/moonbit`
2. Run `make test` multiple times
3. Observe that approximately 50% of runs fail with SIGSEGV

```bash
# Run this to see the intermittent failures
for i in {1..10}; do
    echo "Run $i"
    make test
done
```

## Symptoms
- **Error Message**: `The test executable exited with signal: 11 (SIGSEGV)`
- **Failure Point**: During `op.delete()` calls in test cleanup
- **Consistency**: Intermittent - approximately 50% failure rate
- **Pattern**: Failures occur more frequently when tests run in quick succession

## Root Cause Analysis

### Stack Trace
```
* thread #1, stop reason = EXC_BAD_ACCESS (code=1, address=0x4652a09040208)
  * frame #0: tokio::util::rand::rt::RngSeedGenerator::next_seed + 28
    frame #1: tokio::runtime::context::runtime::enter_runtime + 180
    frame #2: opendal::blocking::operator::Operator::delete + 128
    frame #3: opendal_operator_delete + 84
    frame #4: moonbit_opendal_operator_delete (wrapper.c:150)
    frame #5: Operator::delete (MoonBit)
```

### Technical Analysis

#### 1. Global Tokio Runtime
The OpenDAL C bindings use a global `LazyLock<tokio::runtime::Runtime>`:

```rust
// bindings/c/src/operator.rs:28-33
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});
```

#### 2. Thread-Local Storage Corruption
The crash occurs in `RngSeedGenerator::next_seed`, which accesses thread-local storage (TLS):
- **Crash Address**: `0x4652a09040208` (invalid memory address)
- **Crash Instruction**: `ldapr x0, [x0]` - attempting to load from corrupted pointer
- **Location**: Tokio's internal RNG initialization code

#### 3. Race Condition
The issue manifests as a race condition:
- When tests run sequentially with proper timing → SUCCESS
- When tests run in quick succession → TLS state becomes corrupted → SIGSEGV
- The LazyLock initialization may not be thread-safe in this specific context

### Why This Happens
1. **First Test**: Initializes the global RUNTIME successfully
2. **Subsequent Tests**: Reuse the same runtime
3. **Problem**: Tokio's TLS for RNG state gets corrupted when:
   - Multiple operators are created/destroyed rapidly
   - The runtime's internal state is accessed from different contexts
   - TLS cleanup doesn't happen properly between test runs

## Impact
- **Severity**: HIGH - Makes tests unreliable
- **Scope**: Affects all MoonBit binding tests
- **User Impact**: Cannot reliably validate MoonBit bindings
- **CI/CD Impact**: Would cause intermittent CI failures

## Proposed Solutions

### Solution 1: Fix in OpenDAL C Bindings (Recommended)
**Location**: `bindings/c/src/operator.rs`

**Option A**: Use `OnceCell` with explicit initialization
```rust
use std::sync::OnceLock;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}
```

**Option B**: Use single-threaded runtime
```rust
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()  // Instead of new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});
```

**Option C**: Add proper synchronization
```rust
use std::sync::Mutex;

static RUNTIME_LOCK: Mutex<()> = Mutex::new(());

// In build_operator function:
let _lock = RUNTIME_LOCK.lock().unwrap();
let runtime = tokio::runtime::Handle::try_current()
    .unwrap_or_else(|_| RUNTIME.handle().clone());
```

### Solution 2: Workaround in MoonBit Tests
**Location**: `bindings/moonbit/tests/behavior_tests/test_helper.mbt`

Add a small delay between operations:
```moonbit
pub fn test_delay() -> Unit {
    // Add a small delay to allow Tokio runtime to stabilize
    // This is a workaround, not a proper fix
}
```

### Solution 3: Rebuild with Different Configuration
Modify `bindings/c/Cargo.toml` to use different Tokio features or configuration.

## Recommended Action Plan

1. **Immediate**: Document this issue and workarounds
2. **Short-term**: Report to Apache OpenDAL project with this analysis
3. **Long-term**: Implement Solution 1 (Option A or B) in OpenDAL C bindings

## Additional Notes

### Related Files
- `bindings/c/src/operator.rs` - Global runtime definition
- `bindings/moonbit/src/wrapper.c` - C wrapper functions
- `bindings/moonbit/src/ffi.mbt` - FFI bindings
- `bindings/moonbit/tests/behavior_tests/*.mbt` - Test files

### Testing Commands
```bash
# Test multiple times to observe failures
cd bindings/moonbit
./run_tests.sh

# Debug with lldb
./debug_crash.sh

# Simulate make test behavior
./simulate_make_test.sh
```

### References
- Tokio Runtime Documentation: https://docs.rs/tokio/latest/tokio/runtime/
- Rust LazyLock: https://doc.rust-lang.org/std/sync/struct.LazyLock.html
- OpenDAL C Bindings: https://opendal.apache.org/docs/c/

## Conclusion
This is a **critical bug** in the interaction between:
1. OpenDAL's global Tokio runtime (C bindings)
2. Tokio's thread-local RNG state
3. Rapid creation/destruction of operators in tests

The fix should be implemented in the OpenDAL C bindings layer to ensure thread-safety and proper TLS management.
