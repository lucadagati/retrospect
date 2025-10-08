/* ARM Cortex-M minimal firmware */
.syntax unified
.thumb

.section .vectors
.word 0x20001000  /* Stack pointer */
.word _start     /* Reset vector */

.section .text
.global _start
_start:
    /* Infinite loop - minimal working firmware */
    b .
