MEMORY
{
  RAM : ORIGIN = 0x00000000, LENGTH = 16M
}

SECTIONS
{
  .text : { *(.text .text.*) } > RAM
  .rodata : { *(.rodata .rodata.*) } > RAM
  .data : { *(.data .data.*) } > RAM
  .bss : { *(.bss .bss.*) } > RAM
}
