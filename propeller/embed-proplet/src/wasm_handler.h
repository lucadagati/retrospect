#ifndef WASM_HANDLER_H
#define WASM_HANDLER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

    /**
     * Loads and instantiates a Wasm module, then invokes the named exported
     * function. The module remains loaded in memory so it can be stopped later.
     *
     * @param task_id      Unique identifier for this Wasm "task."
     * @param func_name    Exported function to invoke (e.g. "add", "main").
     * @param wasm_data    Pointer to the Wasm file bytes.
     * @param wasm_size    Size of the Wasm file in bytes.
     * @param inputs       Array of 64-bit inputs passed as function arguments.
     * @param inputs_count Number of elements in 'inputs'.
     */
    void execute_wasm_module(const char *task_id, const char *func_name,
                             const uint8_t *wasm_data, size_t wasm_size,
                             const uint64_t *inputs, size_t inputs_count);

    /**
     * Stops the Wasm module with the given task_id by deinstantiating and unloading
     * it from memory.
     *
     * @param task_id  The unique string ID assigned to the Wasm module at start
     * time.
     */
    void stop_wasm_app(const char *task_id);

#ifdef __cplusplus
}
#endif

#endif /* WASM_HANDLER_H */
