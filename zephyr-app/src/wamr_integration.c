/*
 * SPDX-License-Identifier: Apache-2.0
 * Copyright © 2025 Wasmbed contributors
 *
 * WAMR Integration Layer Implementation
 */

#include "wamr_integration.h"
#include <zephyr/logging/log.h>
#include "wasm_export.h"
#include <string.h>

LOG_MODULE_REGISTER(wamr_integration, LOG_LEVEL_INF);

/* WAMR runtime state */
static bool wamr_initialized = false;

/* WAMR heap buffer (64KB) */
#define WAMR_HEAP_SIZE (64 * 1024)
static uint8_t wamr_heap_buffer[WAMR_HEAP_SIZE] __aligned(8);

/* Module registry */
#define MAX_MODULES 16
#define MAX_INSTANCES 16

typedef struct {
    uint32_t id;
    wasm_module_t module;
    bool in_use;
} module_entry_t;

typedef struct {
    uint32_t id;
    wasm_module_inst_t instance;
    wasm_exec_env_t exec_env;
    bool in_use;
} instance_entry_t;

static module_entry_t modules[MAX_MODULES];
static instance_entry_t instances[MAX_INSTANCES];
static uint32_t next_module_id = 1;
static uint32_t next_instance_id = 1;

/* Initialize WAMR runtime */
int wamr_init(void)
{
    if (wamr_initialized) {
        LOG_WRN("WAMR already initialized");
        return 0;
    }

    LOG_INF("Initializing WAMR runtime...");
    
    RuntimeInitArgs init_args;
    memset(&init_args, 0, sizeof(RuntimeInitArgs));
    
    /* Use pool memory allocator with static buffer */
    init_args.mem_alloc_type = Alloc_With_Pool;
    init_args.mem_alloc_option.pool.heap_buf = wamr_heap_buffer;
    init_args.mem_alloc_option.pool.heap_size = WAMR_HEAP_SIZE;
    
    /* No native symbols for now */
    init_args.native_symbols = NULL;
    init_args.n_native_symbols = 0;
    
    if (!wasm_runtime_full_init(&init_args)) {
        LOG_ERR("Failed to initialize WAMR runtime");
        return -1;
    }

    /* Initialize module and instance registries */
    memset(modules, 0, sizeof(modules));
    memset(instances, 0, sizeof(instances));

    wamr_initialized = true;
    LOG_INF("WAMR runtime initialized");
    
    return 0;
}

/* Load WASM module from bytes */
int wamr_load_module(const uint8_t *wasm_bytes, uint32_t wasm_size, uint32_t *module_id)
{
    if (!wamr_initialized) {
        LOG_ERR("WAMR not initialized");
        return -1;
    }

    if (wasm_bytes == NULL || wasm_size == 0 || module_id == NULL) {
        LOG_ERR("Invalid parameters");
        return -1;
    }

    LOG_INF("Loading WASM module (size: %u bytes)...", wasm_size);

    /* Find free module slot */
    int slot = -1;
    for (int i = 0; i < MAX_MODULES; i++) {
        if (!modules[i].in_use) {
            slot = i;
            break;
        }
    }
    
    if (slot < 0) {
        LOG_ERR("No free module slots");
        return -1;
    }

    /* Error buffer for WAMR */
    static char error_buf[128];
    error_buf[0] = '\0';

    /* Load WASM module */
    wasm_module_t module = wasm_runtime_load((uint8_t *)wasm_bytes, wasm_size, 
                                            error_buf, sizeof(error_buf));
    
    if (module == NULL) {
        LOG_ERR("Failed to load WASM module: %s", error_buf);
        return -1;
    }

    /* Store module in registry */
    modules[slot].id = next_module_id++;
    modules[slot].module = module;
    modules[slot].in_use = true;

    *module_id = modules[slot].id;
    LOG_INF("WASM module loaded (module_id: %u)", *module_id);

    return 0;
}

/* Instantiate WASM module */
int wamr_instantiate(uint32_t module_id, uint32_t *instance_id)
{
    if (!wamr_initialized) {
        LOG_ERR("WAMR not initialized");
        return -1;
    }

    LOG_INF("Instantiating WASM module (module_id: %u)...", module_id);

    /* Find module */
    wasm_module_t module = NULL;
    int module_slot = -1;
    for (int i = 0; i < MAX_MODULES; i++) {
        if (modules[i].in_use && modules[i].id == module_id) {
            module = modules[i].module;
            module_slot = i;
            break;
        }
    }
    
    if (module == NULL) {
        LOG_ERR("Module not found: %u", module_id);
        return -1;
    }

    /* Find free instance slot */
    int slot = -1;
    for (int i = 0; i < MAX_INSTANCES; i++) {
        if (!instances[i].in_use) {
            slot = i;
            break;
        }
    }
    
    if (slot < 0) {
        LOG_ERR("No free instance slots");
        return -1;
    }

    /* Default stack size: 64KB */
    uint32_t stack_size = 64 * 1024;
    uint32_t heap_size = 0; /* Use default */

    /* Error buffer for WAMR */
    static char error_buf[128];
    error_buf[0] = '\0';

    /* Instantiate WASM module */
    wasm_module_inst_t instance = wasm_runtime_instantiate(module, stack_size, 
                                                           heap_size, error_buf, 
                                                           sizeof(error_buf));
    
    if (instance == NULL) {
        LOG_ERR("Failed to instantiate WASM module: %s", error_buf);
        return -1;
    }

    /* Create execution environment */
    wasm_exec_env_t exec_env = wasm_runtime_create_exec_env(instance, stack_size);
    if (exec_env == NULL) {
        LOG_ERR("Failed to create execution environment");
        wasm_runtime_deinstantiate(instance);
        return -1;
    }

    /* Store instance in registry */
    instances[slot].id = next_instance_id++;
    instances[slot].instance = instance;
    instances[slot].exec_env = exec_env;
    instances[slot].in_use = true;

    *instance_id = instances[slot].id;
    LOG_INF("WASM module instantiated (instance_id: %u)", *instance_id);

    return 0;
}

/* Execute WASM function */
int wamr_call_function(uint32_t instance_id, const char *function_name,
                       uint32_t *args, uint32_t args_count, uint32_t *results, uint32_t results_count)
{
    if (!wamr_initialized) {
        LOG_ERR("WAMR not initialized");
        return -1;
    }

    LOG_INF("Calling WASM function: %s (instance_id: %u)", function_name, instance_id);

    /* Find instance */
    wasm_module_inst_t instance = NULL;
    wasm_exec_env_t exec_env = NULL;
    for (int i = 0; i < MAX_INSTANCES; i++) {
        if (instances[i].in_use && instances[i].id == instance_id) {
            instance = instances[i].instance;
            exec_env = instances[i].exec_env;
            break;
        }
    }
    
    if (instance == NULL || exec_env == NULL) {
        LOG_ERR("Instance not found: %u", instance_id);
        return -1;
    }

    /* Find function */
    wasm_function_inst_t function = wasm_runtime_lookup_function(instance, function_name);
    if (function == NULL) {
        LOG_ERR("Function not found: %s", function_name);
        return -1;
    }

    /* Call WASM function - wasm_runtime_call_wasm takes argc and argv array */
    /* Note: Results are stored in the WASM stack and can be read using 
     * wasm_runtime_get_function_ret_value or similar APIs if needed */
    if (!wasm_runtime_call_wasm(exec_env, function, args_count, args)) {
        const char *exception = wasm_runtime_get_exception(instance);
        if (exception) {
            LOG_ERR("WASM exception: %s", exception);
        } else {
            LOG_ERR("Failed to call WASM function: %s", function_name);
        }
        return -1;
    }

    /* TODO: Read return values from WASM stack if results_count > 0
     * For now, results parameter is ignored as WAMR stores results differently
     * This can be enhanced later if needed for specific use cases */

    LOG_INF("WASM function executed: %s", function_name);

    return 0;
}

/* Process WAMR runtime (call periodically) */
void wamr_process(void)
{
    if (!wamr_initialized) {
        return;
    }

    /* TODO: Process WAMR runtime events if needed */
}

/* Cleanup WAMR runtime */
void wamr_cleanup(void)
{
    if (!wamr_initialized) {
        return;
    }

    LOG_INF("Cleaning up WAMR runtime...");

    /* Cleanup all instances */
    for (int i = 0; i < MAX_INSTANCES; i++) {
        if (instances[i].in_use) {
            if (instances[i].exec_env) {
                wasm_runtime_destroy_exec_env(instances[i].exec_env);
            }
            if (instances[i].instance) {
                wasm_runtime_deinstantiate(instances[i].instance);
            }
            instances[i].in_use = false;
        }
    }

    /* Cleanup all modules */
    for (int i = 0; i < MAX_MODULES; i++) {
        if (modules[i].in_use) {
            if (modules[i].module) {
                wasm_runtime_unload(modules[i].module);
            }
            modules[i].in_use = false;
        }
    }

    /* Destroy WAMR runtime */
    wasm_runtime_destroy();

    wamr_initialized = false;
    LOG_INF("WAMR runtime cleaned up");
}

