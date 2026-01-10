/*
 * SPDX-License-Identifier: Apache-2.0
 * Copyright © 2025 Wasmbed contributors
 *
 * Network Handler
 * TCP/IP and TLS communication
 */

#ifndef NETWORK_HANDLER_H
#define NETWORK_HANDLER_H

#include <stdint.h>

/* Initialize network stack */
int network_init(void);

/* Process network events */
void network_process(void);

/* Connect to gateway */
int network_connect(const char *host, uint16_t port);

/* Connect to gateway with TLS */
int network_connect_tls(const char *host, uint16_t port);

/* Send data via network */
int network_send(const uint8_t *data, uint32_t data_len);

/* Receive data from network */
int network_receive(uint8_t *buffer, uint32_t buffer_len, uint32_t *received_len);

#endif /* NETWORK_HANDLER_H */

