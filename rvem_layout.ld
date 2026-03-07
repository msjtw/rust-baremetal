OUTPUT_ARCH("riscv")

ENTRY(_entry)

MEMORY {
  ram (rwx) : ORIGIN = 0x80000000, LENGTH = 4M
}

SECTIONS {
  .text.boot : {
    *(.text.boot)
  } > ram

  .text : {
    *(.text)
  } > ram

  .data : {
    *(.data)
  } > ram

  .rodata : {
    *(.rodata)
  } > ram

  .bss : {
    *(.bss)
  } > ram

  . = ALIGN(8);
  . = . + 4096;
  _STACK_PTR = .;
}
