/*
 * SPDX-License-Identifier: Apache-2.0
 * Copyright © 2025 Wasmbed contributors
 *
 * Wasmbed Protocol Handler
 * Handles communication with Wasmbed gateway
 */

#ifndef WASMBED_PROTOCOL_H
#define WASMBED_PROTOCOL_H

#include <stdint.h>

/* Initialize Wasmbed protocol handler */
int wasmbed_protocol_init(void);

/* Handle incoming message from gateway */
int wasmbed_protocol_handle_message(const uint8_t *data, uint32_t data_len);

/* Send message to gateway */
int wasmbed_protocol_send_message(const uint8_t *data, uint32_t data_len);

#endif /* WASMBED_PROTOCOL_H */

