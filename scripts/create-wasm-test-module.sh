#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Create a simple WASM test module for WAMR testing

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Creating WASM Test Module ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

WASM_DIR="$PROJECT_ROOT/wasm-test-modules"
mkdir -p "$WASM_DIR"

# Create a simple C test program
TEST_C="$WASM_DIR/test_simple.c"

cat > "$TEST_C" << 'EOF'
// Simple WASM test module
// Compile with: wasi-sdk or emscripten

int add(int a, int b) {
    return a + b;
}

int multiply(int a, int b) {
    return a * b;
}

int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

int main() {
    // Test functions
    int result1 = add(5, 3);
    int result2 = multiply(4, 7);
    int result3 = fibonacci(10);
    return 0;
}
EOF

echo -e "${GREEN}✓${NC} Created test C source: $TEST_C"
echo ""

# Create Wat (WebAssembly Text) format
TEST_WAT="$WASM_DIR/test_simple.wat"

cat > "$TEST_WAT" << 'EOF'
(module
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
  (export "add" (func $add))
  
  (func $multiply (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )
  (export "multiply" (func $multiply))
  
  (func $fibonacci (param $n i32) (result i32)
    (if (result i32)
      (i32.le_s (local.get $n) (i32.const 1))
      (then (local.get $n))
      (else
        (i32.add
          (call $fibonacci (i32.sub (local.get $n) (i32.const 1)))
          (call $fibonacci (i32.sub (local.get $n) (i32.const 2)))
        )
      )
    )
  )
  (export "fibonacci" (func $fibonacci))
)
EOF

echo -e "${GREEN}✓${NC} Created test WAT source: $TEST_WAT"
echo ""

# Create compilation instructions
README="$WASM_DIR/README.md"

cat > "$README" << 'EOF'
# WASM Test Modules

Simple test modules for WAMR integration testing.

## Compilation

### Using wat2wasm (from WABT)

```bash
# Install WABT
sudo apt-get install wabt

# Compile WAT to WASM
wat2wasm test_simple.wat -o test_simple.wasm
```

### Using WASI SDK

```bash
# Download WASI SDK from https://github.com/WebAssembly/wasi-sdk

# Compile C to WASM
clang --target=wasm32-wasi test_simple.c -o test_simple.wasm
```

### Using Emscripten

```bash
# Install Emscripten
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh

# Compile C to WASM
emcc test_simple.c -o test_simple.wasm
```

## Usage in WAMR

Once compiled, the WASM module can be loaded and executed in WAMR:

```c
// Load module
uint8_t *wasm_bytes = ...; // Read from file
uint32_t wasm_size = ...;
wasm_module_t module = wasm_runtime_load(wasm_bytes, wasm_size, ...);

// Instantiate
wasm_module_inst_t instance = wasm_runtime_instantiate(module, ...);

// Call function
wasm_function_inst_t func = wasm_runtime_lookup_function(instance, "add");
uint32_t args[2] = {5, 3};
wasm_runtime_call_wasm(exec_env, func, 2, args);
```

## Test Functions

- `add(a, b)` - Returns a + b
- `multiply(a, b)` - Returns a * b
- `fibonacci(n)` - Returns nth Fibonacci number
EOF

echo -e "${GREEN}✓${NC} Created README: $README"
echo ""

echo -e "${GREEN}=== WASM Test Module Created ===${NC}"
echo ""
echo "Files created in: $WASM_DIR"
echo ""
echo "To compile:"
echo "  1. Install WABT: sudo apt-get install wabt"
echo "  2. Compile: wat2wasm $TEST_WAT -o $WASM_DIR/test_simple.wasm"
echo ""
echo "Or use WASI SDK or Emscripten (see README.md)"

