/*
 * SPDX-License-Identifier: Apache-2.0
 * Copyright © 2025 Wasmbed contributors
 *
 * OCRE Integration Layer — wraps ocre_initialize / ocre_create_context /
 * ocre_context_create_container / ocre_container_start / … API.
 *
 * Flow per WASM deployment received over the network:
 *  1. Write the raw WASM bytes to LittleFS: <workdir>/images/<id>.wasm
 *  2. ocre_context_create_container(ctx, "<id>.wasm", "wamr/wasip1", …)
 *  3. ocre_container_start(container)
 */

#include "ocre_integration.h"

#include <zephyr/logging/log.h>
#include <zephyr/kernel.h>
#include <ocre/ocre.h>
#include <fcntl.h>
#include <string.h>
#include <stdio.h>
#include <sys/stat.h>
#include <errno.h>

/* High-resolution timer: µs wall-clock time.
 * On native_sim (CONFIG_ARCH_POSIX): use get_host_us_time() from Zephyr's native
 * simulator timer model, which calls clock_gettime(CLOCK_MONOTONIC_RAW) on the
 * host OS directly — bypasses Zephyr's POSIX shim that returns simulated tick time.
 * On real HW: fall back to k_uptime_get_32() with ms→µs conversion (1ms resolution). */
#if defined(CONFIG_ARCH_POSIX)
/* Declared in zephyr/scripts/native_simulator/native/src/timer_model.c */
extern uint64_t get_host_us_time(void);
static inline uint64_t _timing_us(void)
{
    return get_host_us_time();
}
#else
static inline uint64_t _timing_us(void)
{
    return (uint64_t)k_uptime_get_32() * 1000ULL;  /* ms → µs, 1ms resolution on HW */
}
#endif

LOG_MODULE_REGISTER(ocre_integration, LOG_LEVEL_INF);

#define OCRE_WORKDIR  "/lfs/ocre"
#define IMAGES_SUBDIR "/images"
#define WASM_RUNTIME  "wamr/wasip1"
#define MAX_CONTAINERS 4

typedef struct {
    ocre_handle_t handle;
    struct ocre_container *container;
    char image_name[64];
    bool in_use;
} container_slot_t;

static struct ocre_context *g_ocre_ctx;
static container_slot_t slots[MAX_CONTAINERS];
static bool runtime_initialized;
static uint32_t next_handle = 1U;

/* Write raw bytes to <workdir>/images/<name>.wasm on LittleFS. */
static int write_wasm_image(const char *name, const uint8_t *data, uint32_t size)
{
    /* Ensure images directory exists */
    char dir[sizeof(OCRE_WORKDIR) + sizeof(IMAGES_SUBDIR) + 1];
    snprintf(dir, sizeof(dir), "%s%s", OCRE_WORKDIR, IMAGES_SUBDIR);
    (void)mkdir(dir, 0755);

    char path[sizeof(dir) + 64];
    snprintf(path, sizeof(path), "%s/%s", dir, name);

    int fd = open(path, O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (fd < 0) {
        LOG_ERR("open(%s) failed: %d", path, errno);
        return -errno;
    }

    ssize_t written = write(fd, data, size);
    close(fd);

    if (written != (ssize_t)size) {
        LOG_ERR("write(%s) short: %d/%u", path, (int)written, size);
        return -EIO;
    }

    LOG_DBG("WASM image written to %s (%u bytes)", path, size);
    return 0;
}

int ocre_integration_init(void)
{
    if (runtime_initialized) {
        return 0;
    }

    LOG_INF("Initializing OCRE runtime...");

    int ret = ocre_initialize(NULL);
    if (ret != 0) {
        LOG_ERR("ocre_initialize failed: %d", ret);
        return ret;
    }

    g_ocre_ctx = ocre_create_context(OCRE_WORKDIR);
    if (!g_ocre_ctx) {
        LOG_ERR("ocre_create_context failed");
        ocre_deinitialize();
        return -ENOMEM;
    }

    memset(slots, 0, sizeof(slots));
    runtime_initialized = true;
    LOG_INF("OCRE runtime ready (workdir: %s)", OCRE_WORKDIR);
    return 0;
}

ocre_handle_t ocre_integration_deploy(const uint8_t *wasm_bytes, uint32_t wasm_size)
{
    if (!runtime_initialized || !g_ocre_ctx) {
        LOG_ERR("OCRE not initialized");
        return OCRE_INVALID_HANDLE;
    }

    if (!wasm_bytes || wasm_size == 0U) {
        LOG_ERR("Invalid WASM payload");
        return OCRE_INVALID_HANDLE;
    }

    int slot = -1;
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (!slots[i].in_use) {
            slot = i;
            break;
        }
    }
    if (slot < 0) {
        LOG_ERR("No free container slots");
        return OCRE_INVALID_HANDLE;
    }

    /* Build a unique image filename based on handle counter */
    char img_name[64];
    snprintf(img_name, sizeof(img_name), "app_%u.wasm", next_handle);

    /* --- Phase 1: LittleFS write --- */
    uint64_t t_lfs_start = _timing_us();
    int ret = write_wasm_image(img_name, wasm_bytes, wasm_size);
    uint64_t t_lfs_end = _timing_us();
    if (ret != 0) {
        LOG_ERR("Failed to write WASM image: %d", ret);
        return OCRE_INVALID_HANDLE;
    }
    LOG_INF("TIMING lfs_write_us=%llu wasm_bytes=%u",
            (unsigned long long)(t_lfs_end - t_lfs_start), (unsigned)wasm_size);

    /* --- Phase 2: WAMR load (ocre_context_create_container) --- */
    LOG_INF("Creating OCRE container from %s (%u bytes)", img_name, wasm_size);
    uint64_t t_load_start = _timing_us();
    struct ocre_container *c = ocre_context_create_container(
        g_ocre_ctx,
        img_name,
        WASM_RUNTIME,
        NULL,   /* auto-generate container ID */
        true,   /* detached — run in background */
        NULL,   /* no extra args */
        -1, -1, -1);
    uint64_t t_load_end = _timing_us();

    if (!c) {
        LOG_ERR("ocre_context_create_container failed");
        return OCRE_INVALID_HANDLE;
    }
    LOG_INF("TIMING wamr_load_us=%llu", (unsigned long long)(t_load_end - t_load_start));

    /* --- Phase 3: WAMR start (ocre_container_start) --- */
    uint64_t t_start_start = _timing_us();
    ret = ocre_container_start(c);
    uint64_t t_start_end = _timing_us();
    if (ret != 0) {
        LOG_ERR("ocre_container_start failed: %d", ret);
        (void)ocre_context_remove_container(g_ocre_ctx, c);
        return OCRE_INVALID_HANDLE;
    }
    LOG_INF("TIMING wamr_start_us=%llu", (unsigned long long)(t_start_end - t_start_start));

    slots[slot].handle    = next_handle++;
    slots[slot].container = c;
    strncpy(slots[slot].image_name, img_name, sizeof(slots[slot].image_name) - 1);
    slots[slot].in_use    = true;

    LOG_INF("OCRE container running (handle %u, image %s)", slots[slot].handle, img_name);
    return slots[slot].handle;
}

int ocre_integration_stop(ocre_handle_t handle)
{
    if (handle == OCRE_INVALID_HANDLE || !runtime_initialized) {
        return -EINVAL;
    }

    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (slots[i].in_use && slots[i].handle == handle) {
            LOG_INF("Stopping OCRE container (handle %u)", handle);
            (void)ocre_container_kill(slots[i].container);
            (void)ocre_context_remove_container(g_ocre_ctx, slots[i].container);
            slots[i].in_use = false;
            return 0;
        }
    }

    LOG_WRN("Container handle %u not found", handle);
    return -ENOENT;
}

bool ocre_integration_is_running(ocre_handle_t handle)
{
    if (handle == OCRE_INVALID_HANDLE || !runtime_initialized) {
        return false;
    }
    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (slots[i].in_use && slots[i].handle == handle) {
            ocre_container_status_t st = ocre_container_get_status(slots[i].container);
            return st == OCRE_CONTAINER_STATUS_RUNNING;
        }
    }
    return false;
}

void ocre_integration_cleanup(void)
{
    if (!runtime_initialized) {
        return;
    }

    for (int i = 0; i < MAX_CONTAINERS; i++) {
        if (slots[i].in_use) {
            (void)ocre_container_kill(slots[i].container);
            (void)ocre_context_remove_container(g_ocre_ctx, slots[i].container);
            slots[i].in_use = false;
        }
    }

    if (g_ocre_ctx) {
        ocre_destroy_context(g_ocre_ctx);
        g_ocre_ctx = NULL;
    }

    ocre_deinitialize();
    runtime_initialized = false;
    LOG_INF("OCRE runtime cleaned up");
}
