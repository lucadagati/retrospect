/* WAMR's zephyr_platform.c calls __stdout_hook_install to redirect its printf
 * output through Zephyr's console. On native_sim, CONFIG_NATIVE_LIBC=y provides
 * the host libc which does not define this function. This stub keeps the linker
 * happy; stdout already works on native_sim via the host process stdio. */
void __stdout_hook_install(int (*hook)(int))
{
	(void)hook;
}
