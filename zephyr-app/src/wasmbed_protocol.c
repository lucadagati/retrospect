/*
 * SPDX-License-Identifier: Apache-2.0
 * Copyright © 2025 Wasmbed contributors
 *
 * Wasmbed Protocol Handler Implementation
 */

#include "wasmbed_protocol.h"
#include "wamr_integration.h"
#include "network_handler.h"
#include <zephyr/logging/log.h>
#include <string.h>
#include <zephyr/kernel.h>
#include <stdlib.h>

LOG_MODULE_REGISTER(wasmbed_protocol, LOG_LEVEL_INF);

/* Memory address where Renode writes the gateway endpoint */
#define GATEWAY_ENDPOINT_ADDR 0x20001000

static bool protocol_initialized = false;
static char gateway_endpoint[64] = {0};
static bool gateway_connected = false;

/* Read gateway endpoint from memory (written by Renode) */
static int read_gateway_endpoint(void)
{
    /* Read length from first 4 bytes */
    uint32_t *length_ptr = (uint32_t *)GATEWAY_ENDPOINT_ADDR;
    uint32_t length = *length_ptr;
    
    if (length == 0 || length >= sizeof(gateway_endpoint)) {
        LOG_ERR("Invalid endpoint length: %u", length);
        return -1;
    }
    
    /* Read endpoint string from memory */
    char *endpoint_ptr = (char *)(GATEWAY_ENDPOINT_ADDR + 4);
    memcpy(gateway_endpoint, endpoint_ptr, length);
    gateway_endpoint[length] = '\0';
    
    LOG_INF("Read gateway endpoint from memory: %s (length: %u)", gateway_endpoint, length);
    return 0;
}

/* Parse endpoint string (format: "host:port") */
static int parse_endpoint(const char *endpoint, char *host, size_t host_len, uint16_t *port)
{
    if (endpoint == NULL || host == NULL || port == NULL) {
        return -1;
    }
    
    /* Find colon separator */
    const char *colon = strchr(endpoint, ':');
    if (colon == NULL) {
        LOG_ERR("Invalid endpoint format (missing port): %s", endpoint);
        return -1;
    }
    
    /* Extract host */
    size_t host_len_actual = colon - endpoint;
    if (host_len_actual >= host_len) {
        LOG_ERR("Host name too long");
        return -1;
    }
    memcpy(host, endpoint, host_len_actual);
    host[host_len_actual] = '\0';
    
    /* Extract port */
    *port = (uint16_t)atoi(colon + 1);
    if (*port == 0) {
        LOG_ERR("Invalid port number");
        return -1;
    }
    
    return 0;
}

/* Initialize Wasmbed protocol handler */
int wasmbed_protocol_init(void)
{
    if (protocol_initialized) {
        LOG_WRN("Protocol already initialized");
        return 0;
    }

    LOG_INF("Initializing Wasmbed protocol handler...");
    
    /* Read gateway endpoint from memory (written by Renode) */
    if (read_gateway_endpoint() != 0) {
        LOG_ERR("Failed to read gateway endpoint from memory");
        /* Use default endpoint as fallback */
        strncpy(gateway_endpoint, "127.0.0.1:40029", sizeof(gateway_endpoint) - 1);
        LOG_WRN("Using default endpoint: %s", gateway_endpoint);
    }
    
    /* Parse endpoint and connect to gateway with TLS */
    char host[64];
    uint16_t port;
    if (parse_endpoint(gateway_endpoint, host, sizeof(host), &port) == 0) {
        LOG_INF("Connecting to gateway with TLS: %s:%u", host, port);
        /* Add delay to ensure network is ready */
        k_sleep(K_SECONDS(1));
        if (network_connect_tls(host, port) == 0) {
            gateway_connected = true;
            LOG_INF("Connected to gateway via TLS");
        } else {
            LOG_ERR("Failed to connect to gateway with TLS - will retry later");
            /* Don't fail initialization - connection can be retried */
        }
    } else {
        LOG_ERR("Failed to parse gateway endpoint");
    }

    protocol_initialized = true;
    LOG_INF("Wasmbed protocol handler initialized");

    return 0;
}

/* Handle incoming message from gateway */
int wasmbed_protocol_handle_message(const uint8_t *data, uint32_t data_len)
{
    if (!protocol_initialized) {
        LOG_ERR("Protocol not initialized");
        return -1;
    }

    if (data == NULL || data_len == 0) {
        LOG_ERR("Invalid message data");
        return -1;
    }

    LOG_INF("Handling message from gateway (size: %u bytes)", data_len);

    /* TODO: Parse CBOR message
     * - Extract message type (deploy, heartbeat, etc.)
     * - Handle WASM deployment: call wamr_load_module()
     * - Handle other message types
     * 
     * For now, just log the message
     */
    LOG_INF("Received message (first 32 bytes):");
    for (uint32_t i = 0; i < data_len && i < 32; i++) {
        LOG_INF("  [%u] = 0x%02x", i, data[i]);
    }

    return 0;
}

/* Send message to gateway */
int wasmbed_protocol_send_message(const uint8_t *data, uint32_t data_len)
{
    if (!protocol_initialized) {
        LOG_ERR("Protocol not initialized");
        return -1;
    }

    if (!gateway_connected) {
        LOG_ERR("Not connected to gateway");
        return -1;
    }

    if (data == NULL || data_len == 0) {
        LOG_ERR("Invalid message data");
        return -1;
    }

    LOG_INF("Sending message to gateway (size: %u bytes)", data_len);

    /* Send via network connection (TCP bridge handles TLS) */
    if (network_send(data, data_len) != 0) {
        LOG_ERR("Failed to send message to gateway");
        return -1;
    }

    return 0;
}

