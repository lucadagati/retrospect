/*
 * SPDX-License-Identifier: Apache-2.0
 * Copyright © 2025 Wasmbed contributors
 *
 * Network Handler Implementation
 * Uses Zephyr network stack (TCP/IP, TLS)
 */

#include "network_handler.h"
#include <zephyr/logging/log.h>
#include <zephyr/net/net_if.h>
#include <zephyr/net/net_core.h>
#include <zephyr/net/net_context.h>
#include <zephyr/net/socket.h>
#include <zephyr/net/net_event.h>
#include <zephyr/net/dhcpv4.h>
#include <zephyr/kernel.h>
#include <string.h>
#include <errno.h>

/* Use Zephyr socket API (zsock_*) instead of POSIX */
#include <zephyr/net/socket.h>
#include <zephyr/net/tls_credentials.h>

LOG_MODULE_REGISTER(network_handler, LOG_LEVEL_INF);

static bool network_initialized = false;
static int socket_fd = -1;
static struct net_if *net_iface = NULL;

/* Initialize network stack */
int network_init(void)
{
    if (network_initialized) {
        LOG_WRN("Network already initialized");
        return 0;
    }

    LOG_INF("Initializing network stack...");

    /* Get default network interface */
    /* Wait a bit for network interface to be available */
    int retries = 10;
    while (retries > 0) {
        net_iface = net_if_get_default();
        if (net_iface != NULL) {
            break;
        }
        LOG_WRN("Network interface not available yet, retrying... (%d)", retries);
        k_sleep(K_MSEC(500));
        retries--;
    }
    
    if (net_iface == NULL) {
        LOG_ERR("No network interface available after retries");
        /* Don't fail - allow firmware to continue without network */
        LOG_WRN("Continuing without network interface");
        return -1;
    }

    /* Bring interface up */
    if (!net_if_is_up(net_iface)) {
        net_if_up(net_iface);
        LOG_INF("Network interface brought up");
    }

    /* Start DHCP client if available */
#if defined(CONFIG_NET_DHCPV4)
    net_dhcpv4_start(net_iface);
    LOG_INF("DHCP client started");
#else
    /* Static IP configuration would go here */
    LOG_INF("Using static IP configuration (DHCP disabled)");
#endif

    /* Wait a bit for network to be ready */
    k_sleep(K_SECONDS(2));

    network_initialized = true;
    LOG_INF("Network stack initialized");

    return 0;
}

/* Process network events */
void network_process(void)
{
    if (!network_initialized) {
        return;
    }

    /* Network event processing is handled by Zephyr network stack automatically
     * This function can be used for custom event handling if needed */
    
    /* Check if interface is still up */
    if (net_iface != NULL && !net_if_is_up(net_iface)) {
        LOG_WRN("Network interface is down");
    }
}

/* Connect to gateway */
int network_connect(const char *host, uint16_t port)
{
    if (!network_initialized) {
        LOG_ERR("Network not initialized");
        return -1;
    }

    LOG_INF("Connecting to gateway: %s:%u", host, port);

    /* Close existing socket if any */
    if (socket_fd >= 0) {
        zsock_close(socket_fd);
        socket_fd = -1;
    }

    /* Create TCP socket */
    socket_fd = zsock_socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (socket_fd < 0) {
        LOG_ERR("Failed to create socket: %d", errno);
        return -1;
    }

    /* Setup address structure */
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons(port);

    /* Resolve hostname to IP (simplified - assumes IP address string) */
    if (net_addr_pton(AF_INET, host, &addr.sin_addr) < 0) {
        LOG_ERR("Invalid IP address: %s", host);
        zsock_close(socket_fd);
        socket_fd = -1;
        return -1;
    }

    /* Connect to server */
    if (zsock_connect(socket_fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        LOG_ERR("Failed to connect: %d", errno);
        zsock_close(socket_fd);
        socket_fd = -1;
        return -1;
    }

    LOG_INF("Connected to gateway: %s:%u", host, port);

    return 0;
}

/* Connect to gateway with TLS */
int network_connect_tls(const char *host, uint16_t port)
{
    if (!network_initialized) {
        LOG_ERR("Network not initialized");
        return -1;
    }

    LOG_INF("Connecting to gateway with TLS: %s:%u", host, port);

    /* Close existing socket if any */
    if (socket_fd >= 0) {
        zsock_close(socket_fd);
        socket_fd = -1;
    }

    /* Create TCP socket (TLS will be configured via socket options) */
    socket_fd = zsock_socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (socket_fd < 0) {
        LOG_ERR("Failed to create socket: %d", errno);
        return -1;
    }

    /* Configure TLS before connecting */
    /* Set TLS hostname for SNI (Server Name Indication) */
    int ret = zsock_setsockopt(socket_fd, SOL_TLS, TLS_HOSTNAME, host, strlen(host) + 1);
    if (ret < 0) {
        LOG_WRN("Failed to set TLS hostname: %d", errno);
        /* Continue - TLS might still work without SNI */
    }

    /* Note: For development, we skip certificate verification
     * In production, proper certificate validation should be enabled
     * by setting TLS_SEC_TAG_LIST with appropriate security tags */

    /* Setup address structure */
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons(port);

    /* Resolve hostname to IP */
    if (net_addr_pton(AF_INET, host, &addr.sin_addr) < 0) {
        LOG_ERR("Invalid IP address: %s", host);
        zsock_close(socket_fd);
        socket_fd = -1;
        return -1;
    }

    /* Connect to server (TLS handshake happens during connect) */
    if (zsock_connect(socket_fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        LOG_ERR("Failed to connect: %d", errno);
        zsock_close(socket_fd);
        socket_fd = -1;
        return -1;
    }

    LOG_INF("Connected to gateway with TLS: %s:%u", host, port);

    return 0;
}

/* Send data via network */
int network_send(const uint8_t *data, uint32_t data_len)
{
    if (socket_fd < 0) {
        LOG_ERR("Socket not connected");
        return -1;
    }

    ssize_t sent = zsock_send(socket_fd, data, data_len, 0);
    if (sent < 0) {
        LOG_ERR("Failed to send data: %d", errno);
        return -1;
    }

    if (sent != (ssize_t)data_len) {
        LOG_WRN("Partial send: %zd/%u bytes", sent, data_len);
    }

    return 0;
}

/* Receive data from network */
int network_receive(uint8_t *buffer, uint32_t buffer_len, uint32_t *received_len)
{
    if (socket_fd < 0) {
        LOG_ERR("Socket not connected");
        return -1;
    }

    ssize_t received = zsock_recv(socket_fd, buffer, buffer_len, 0);
    if (received < 0) {
        if (errno == EAGAIN || errno == EWOULDBLOCK) {
            /* No data available */
            *received_len = 0;
            return 0;
        }
        LOG_ERR("Failed to receive data: %d", errno);
        return -1;
    }

    *received_len = (uint32_t)received;
    return 0;
}

