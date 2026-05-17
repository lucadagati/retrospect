cmake_minimum_required(VERSION 3.20.0)

# Provide an explicit module list before find_package(Zephyr) so that
# zephyr_module.cmake's if(WEST OR ZEPHYR_MODULES) block always runs.
# Without this, west.cmake detects no workspace (when ZEPHYR_BASE points
# to propeller-ws bind mount) and sets WEST=WEST-NOTFOUND, causing
# kconfig_module_dirs.cmake to never be generated.
# The stub module is intentionally empty — it just satisfies the non-empty
# ZEPHYR_MODULES requirement without adding any Kconfig or CMake sources.
set(ZEPHYR_MODULES "${CMAKE_CURRENT_SOURCE_DIR}/zephyr-stub")

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(proplet_mqtt_client)

set(WAMR_BUILD_PLATFORM "zephyr")

# Set WAMR target based on Zephyr board architecture.
if(NOT DEFINED WAMR_BUILD_TARGET)
  if(CONFIG_ARCH_POSIX)
    # native_sim runs as a native 64-bit host process
    set(WAMR_BUILD_TARGET "X86_64")
  elseif(CONFIG_XTENSA)
    set(WAMR_BUILD_TARGET "XTENSA")
  elseif(CONFIG_ARM64)
    set(WAMR_BUILD_TARGET "AARCH64")
  elseif(CONFIG_ARM)
    set(WAMR_BUILD_TARGET "THUMBS")
  elseif(CONFIG_RISCV AND CONFIG_64BIT)
    set(WAMR_BUILD_TARGET "RISCV64")
  elseif(CONFIG_RISCV)
    set(WAMR_BUILD_TARGET "RISCV32")
  else()
    message(FATAL_ERROR "embed-proplet: unknown WAMR target for this board. Set WAMR_BUILD_TARGET manually.")
  endif()
endif()

if(NOT DEFINED WAMR_BUILD_INTERP)
  set(WAMR_BUILD_INTERP 1)
endif()

# Disable AOT on native_sim/POSIX — interpreter is sufficient and avoids LLVM dep
if(NOT DEFINED WAMR_BUILD_AOT)
  if(CONFIG_ARCH_POSIX)
    set(WAMR_BUILD_AOT 0)
  else()
    set(WAMR_BUILD_AOT 1)
  endif()
endif()

if(NOT DEFINED WAMR_BUILD_LIBC_BUILTIN)
  set(WAMR_BUILD_LIBC_BUILTIN 1)
endif()

# Disable libc wasi support by default. In the future,
# it can be enabled if libc wasi file/socket/lock support is ready on Zephyr platform
if(NOT DEFINED WAMR_BUILD_LIBC_WASI)
  set(WAMR_BUILD_LIBC_WASI 0)
endif()

set(WAMR_BUILD_GLOBAL_HEAP_POOL 1)
if(NOT DEFINED WAMR_BUILD_GLOBAL_HEAP_SIZE)
  set(WAMR_BUILD_GLOBAL_HEAP_SIZE 40960)
endif()

set(WAMR_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR}/modules/wamr/wasm-micro-runtime)
include(${WAMR_ROOT_DIR}/build-scripts/runtime_lib.cmake)

list(APPEND ZEPHYR_EXTRA_MODULES ${WAMR_ROOT_DIR}/product-mini/platforms/zephyr)

include_directories(${ZEPHYR_BASE}/include/zephyr)

target_include_directories(app PRIVATE
    ${WAMR_ROOT_DIR}/core/iwasm/include
)

# On native_sim the MQTT broker runs on the host loopback (127.0.0.1).
# Override the default broker address (10.42.0.1) at compile time.
if(CONFIG_ARCH_POSIX)
  target_compile_definitions(app PRIVATE MQTT_BROKER_HOSTNAME="127.0.0.1")
endif()

# Embed WAMR as a Zephyr library
zephyr_library_named(wamr_lib)
zephyr_library_sources(${WAMR_RUNTIME_LIB_SOURCE})
zephyr_library_app_memory(wamr_partition)

# Ensure Zephyr-generated headers (including heap_constants.h) exist before
# WAMR sources are compiled. Without this, ninja can schedule wamr_lib
# compilation before the generation step completes.
add_dependencies(wamr_lib zephyr_generated_headers)

# WiFi manager: real implementation when CONFIG_WIFI=y, stub otherwise
if(CONFIG_WIFI)
  target_sources(app PRIVATE src/wifi_manager.c)
else()
  target_sources(app PRIVATE src/wifi_manager_stub.c)
endif()

# On native_sim CONFIG_NATIVE_LIBC=y doesn't provide __stdout_hook_install
# which WAMR's zephyr_platform.c calls. Provide a no-op stub.
if(CONFIG_ARCH_POSIX)
  target_sources(app PRIVATE src/native_sim_stubs.c)
endif()

target_sources(app PRIVATE
  src/cJSON.c
  src/credentials.c
  src/main.c
  src/mqtt_client.c
  src/task_monitor.c
  src/wasm_handler.c
)

target_link_libraries(app PRIVATE wamr_lib)
