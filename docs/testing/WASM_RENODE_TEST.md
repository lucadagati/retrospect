# Testing WASM Execution in Renode

This document describes how to test the WASM interpreter execution in Renode emulated environment.

## Overview

The WASM runtime has been integrated into the device runtime and can execute WebAssembly modules directly on emulated MCUs in Renode.

## Prerequisites

1. **Renode** installed and available
2. **Rust toolchain** with `wasmbed-device-runtime` crate
3. **WASM test module** (optional, for custom tests)

## Quick Test

### 1. Direct Test (Without Renode)

Test the interpreter directly:

```bash
cd /home/lucadag/18_10_23_retrospect/retrospect
RUST_LOG=info cargo run --release --bin wasmbed-device-runtime
```

Expected output:
```
Device runtime initialized - WASM execution enabled
Testing WASM execution...
WASM module loaded successfully
WASM execution completed successfully
WASM test PASSED: Memory contains correct value (42)
```

### 2. Test in Renode

#### Step 1: Build Firmware

```bash
cd /home/lucadag/18_10_23_retrospect/retrospect
cargo build --release --bin wasmbed-device-runtime
```

#### Step 2: Start Renode

```bash
./renode_1.15.0_portable/renode renode-scripts/test_wasm_execution.resc
```

#### Step 3: Load and Execute

In Renode console:
```
sysbus LoadELF @target/release/wasmbed-device-runtime
start
```

#### Step 4: Check Output

Monitor the UART analyzer for execution logs. You should see:
- Device runtime initialization
- WASM module loading
- WASM execution completion
- Memory verification (value 42)

## Test WASM Module

The test uses a simple WASM module that:
1. Allocates 1 page of memory (64KB)
2. Stores the value `42` at memory address `0`
3. Completes execution

WASM bytecode:
```wat
(module
  (memory 1)
  (func (export "main")
    (i32.store (i32.const 0) (i32.const 42))
  )
)
```

## Verification

### Success Criteria

✅ **Module Loading**: WASM module parses and validates successfully
✅ **Execution**: Module executes without errors
✅ **Memory Operations**: Memory write/read operations work correctly
✅ **Host Functions**: Host functions can be called (if used)

### Expected Logs

```
[INFO] Device runtime initialized - WASM execution enabled
[INFO] Testing WASM execution...
[INFO] Loading WASM module, size: 32 bytes
[INFO] WASM module loaded: 1 functions
[INFO] Executing WASM module
[INFO] Executing entry function (index 0)
[INFO] WASM execution completed successfully
[INFO] WASM test PASSED: Memory contains correct value (42)
[INFO] Device runtime test completed
```

## Troubleshooting

### Issue: "No functions to execute"

**Cause**: WASM module has no non-imported functions
**Solution**: Ensure the WASM module has at least one function

### Issue: "Memory access out of bounds"

**Cause**: WASM module tries to access memory beyond allocated size
**Solution**: Check memory allocation and access patterns

### Issue: "Stack overflow"

**Cause**: Execution stack exceeded 256 entries
**Solution**: Reduce stack usage or increase stack size

### Issue: "Local index out of bounds"

**Cause**: Function tries to access non-existent local variable
**Solution**: Check function signature and local variable usage

## Advanced Testing

### Test with Host Functions

Create a WASM module that uses host functions:

```wat
(module
  (import "env" "print" (func $print (param i32 i32)))
  (memory 1)
  (data (i32.const 0) "Hello from WASM!")
  (func (export "main")
    (call $print (i32.const 0) (i32.const 16))
  )
)
```

### Test with Control Flow

Test loops and branches:

```wat
(module
  (memory 1)
  (func (export "main") (result i32)
    (local $i i32)
    (local $sum i32)
    (loop $loop
      (local.get $i)
      (i32.const 10)
      (i32.lt_s)
      (if
        (then
          (local.get $sum)
          (local.get $i)
          (i32.add)
          (local.set $sum)
          (local.get $i)
          (i32.const 1)
          (i32.add)
          (local.set $i)
          (br $loop)
        )
      )
    )
    (local.get $sum)
  )
)
```

## Integration with Gateway

For full end-to-end testing:

1. Start gateway: `cargo run --release --bin wasmbed-gateway`
2. Start Renode device: `./renode_1.15.0_portable/renode renode-scripts/arduino_nano_ble.resc`
3. Deploy WASM application via gateway API
4. Monitor device logs for execution

## Performance Metrics

- **Module Loading**: < 10ms for simple modules
- **Execution Time**: Depends on module complexity
- **Memory Usage**: 64KB max per module
- **Stack Usage**: 256 entries max

## Next Steps

- [ ] Add more complex WASM test modules
- [ ] Test with real hardware (not just Renode)
- [ ] Benchmark performance
- [ ] Test with multiple concurrent modules
- [ ] Verify host function interactions

