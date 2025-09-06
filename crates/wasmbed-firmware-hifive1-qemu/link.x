/* Linker script for RISC-V HiFive1 */

/* Include memory layout */
INCLUDE memory.x

/* Entry point */
ENTRY(_start);

/* Sections */
SECTIONS
{
    /* Vector table at start of flash */
    .text : {
        KEEP(*(.text._start));
        *(.text);
        *(.text.*);
    } > flash

    /* Read-only data */
    .rodata : {
        *(.rodata);
        *(.rodata.*);
    } > flash

    /* Data section: copied from flash to RAM */
    .data : {
        . = ALIGN(4);
        _sdata = .;
        *(.data);
        *(.data.*);
        . = ALIGN(4);
        _edata = .;
    } > ram AT> flash

    /* BSS section: zeroed in RAM */
    .bss : {
        . = ALIGN(4);
        _sbss = .;
        *(.bss);
        *(.bss.*);
        *(COMMON);
        . = ALIGN(4);
        _ebss = .;
    } > ram

    /* Stack */
    .stack : {
        . = ALIGN(16);
        _stack_start = .;
        . = . + _stack_size;
        _stack_end = .;
    } > ram

    /* Discard unused sections */
    /DISCARD/ : {
        *(.comment);
        *(.gnu.*);
        *(.note.*);
        *(.eh_frame);
    }
}
