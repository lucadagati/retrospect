/*
 * SPDX-License-Identifier: Apache-2.0
 * Copyright © 2025 Wasmbed contributors
 *
 * OCRE Integration Layer — wraps ocre_container_runtime_* API
 */

#ifndef OCRE_INTEGRATION_H
#define OCRE_INTEGRATION_H

#include <stdint.h>
#include <stdbool.h>

/* Opaque handle for a deployed WASM container */
typedef uint32_t ocre_handle_t;
#define OCRE_INVALID_HANDLE 0U

/* Initialize the OCRE container runtime */
int ocre_integration_init(void);

/* Load and start a WASM module via OCRE.
 * Returns an opaque handle on success, OCRE_INVALID_HANDLE on failure. */
ocre_handle_t ocre_integration_deploy(const uint8_t *wasm_bytes, uint32_t wasm_size);

/* Stop and destroy a running container */
int ocre_integration_stop(ocre_handle_t handle);

/* Returns true if the handle refers to a running container */
bool ocre_integration_is_running(ocre_handle_t handle);

/* Cleanup all containers and the runtime */
void ocre_integration_cleanup(void);

#endif /* OCRE_INTEGRATION_H */
