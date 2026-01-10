# SPDX-License-Identifier: Apache-2.0

cmake_minimum_required(VERSION 3.20.0)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(wasmbed)

# WAMR integration
set(WAMR_ROOT ${CMAKE_CURRENT_SOURCE_DIR}/../wamr)
set(WAMR_BUILD_PLATFORM "zephyr")

# Configure WAMR build for ARM Cortex-M
if(CMAKE_SYSTEM_PROCESSOR MATCHES "arm")
    set(WAMR_BUILD_TARGET "THUMB")
else()
    set(WAMR_BUILD_TARGET "X86_32")
endif()

# WAMR build options
set(WAMR_BUILD_INTERP 1)
set(WAMR_BUILD_AOT 0)
set(WAMR_BUILD_JIT 0)
set(WAMR_BUILD_FAST_JIT 0)
set(WAMR_BUILD_LIBC_BUILTIN 1)
set(WAMR_BUILD_LIBC_WASI 0)
set(WAMR_BUILD_MULTI_MODULE 1)
set(WAMR_BUILD_FAST_INTERP 1)

# Global heap configuration
set(WAMR_BUILD_GLOBAL_HEAP_POOL 1)
set(WAMR_BUILD_GLOBAL_HEAP_SIZE 131072)  # 128 KB

# Include WAMR build script
set(WAMR_ROOT_DIR ${WAMR_ROOT})
include(${WAMR_ROOT}/build-scripts/runtime_lib.cmake)

# Add WAMR sources to app
target_sources(app PRIVATE ${WAMR_RUNTIME_LIB_SOURCE})

# Include directories
target_include_directories(app PRIVATE ${WAMR_ROOT}/core/iwasm/include)

target_sources(app PRIVATE
    src/main.c
    src/wamr_integration.c
    src/wasmbed_protocol.c
)

# Network stack
target_sources(app PRIVATE
    src/network_handler.c
)

# WAMR sources are included directly in the app target

