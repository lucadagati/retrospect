/* Memory layout for RISC-V HiFive1 (SiFive E) */

/* RAM: 16KB starting at 0x80000000 */
_ram_start = 0x80000000;
_ram_size = 16K;
_ram_end = _ram_start + _ram_size;

/* Flash: 32MB starting at 0x20000000 */
_flash_start = 0x20000000;
_flash_size = 32M;
_flash_end = _flash_start + _flash_size;

/* Stack: grows downward from end of RAM */
_stack_start = _ram_end;
_stack_size = 4K;
_stack_end = _stack_start - _stack_size;

/* Heap: starts after stack */
_heap_start = _stack_end;
_heap_size = 8K;
_heap_end = _heap_start - _heap_size;

/* Data section: copied from flash to RAM */
_sdata = _ram_start;
_edata = _sdata + SIZEOF(.data);
_sidata = LOADADDR(.data);

/* BSS section: zeroed in RAM */
_sbss = _edata;
_ebss = _sbss + SIZEOF(.bss);
