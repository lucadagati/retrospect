/* Linker script for ARM Cortex-M (nRF52840) */

MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
_stack_size = 0x2000;  /* 8KB stack */

ENTRY(reset_handler)

SECTIONS
{
  .vectors :
  {
    . = ALIGN(4);
    LONG(ORIGIN(RAM) + LENGTH(RAM));  /* Initial stack pointer (first word) */
    KEEP(*(.vectors))  /* Interrupt vectors (reset, NMI, etc.) */
  } > FLASH

  .text :
  {
    . = ALIGN(4);
    *(.text*)
    *(.rodata*)
    . = ALIGN(4);
  } > FLASH

  .data :
  {
    . = ALIGN(4);
    _sdata = .;
    *(.data*)
    . = ALIGN(4);
    _edata = .;
  } > RAM AT > FLASH

  .bss :
  {
    . = ALIGN(4);
    _sbss = .;
    *(.bss*)
    . = ALIGN(4);
    _ebss = .;
  } > RAM

  _sidata = LOADADDR(.data);
}

