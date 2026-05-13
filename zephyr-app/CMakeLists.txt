# SPDX-License-Identifier: Apache-2.0

cmake_minimum_required(VERSION 3.20.0)

# Register OCRE as Zephyr extra module (must be before find_package)
get_filename_component(OCRE_BASE
    "${CMAKE_CURRENT_LIST_DIR}/../ocre-workspace/ocre-runtime"
    REALPATH)
list(APPEND ZEPHYR_EXTRA_MODULES ${OCRE_BASE})

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(wasmbed)

target_sources(app PRIVATE
    src/main.c
    src/ocre_integration.c
    src/wasmbed_protocol.c
    src/network_handler.c
)

target_link_libraries(app PUBLIC OcreCore)
