/*
 * SPDX-License-Identifier: Apache-2.0
 * Copyright © 2025 Wasmbed contributors
 *
 * Wasmbed Zephyr Application — OCRE runtime on b_u585i_iot02a
 */

#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>
#include <zephyr/net/net_if.h>
#include <zephyr/net/net_core.h>
#include <zephyr/net/net_mgmt.h>

#include "network_handler.h"
#include "ocre_integration.h"
#include "wasmbed_protocol.h"

LOG_MODULE_REGISTER(wasmbed_main, LOG_LEVEL_INF);

int main(void)
{
    LOG_INF("=== Wasmbed Zephyr Application Starting (OCRE) ===");

    if (network_init() != 0) {
        LOG_ERR("Network init failed — continuing without network");
    }

    if (ocre_integration_init() != 0) {
        LOG_ERR("OCRE runtime init failed");
        return 0;
    }

    if (wasmbed_protocol_init() != 0) {
        LOG_ERR("Wasmbed protocol init failed");
        return 0;
    }

    LOG_INF("=== Wasmbed OCRE Application Ready ===");

    while (1) {
        network_process();
        wasmbed_protocol_recv_and_handle(100);
        wasmbed_protocol_tick();
        k_sleep(K_MSEC(100));
    }
}
