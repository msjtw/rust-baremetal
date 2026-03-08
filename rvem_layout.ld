OUTPUT_ARCH("riscv")

ENTRY(_entry)

MEMORY {
  ram (rwx) : ORIGIN = 0x80200000, LENGTH = 62M
}

SECTIONS {
  .text.boot : {
    *(.text.boot)
  } > ram

  .text : {
    *(.text)
  } > ram
  PROVIDE(etext = .);

  .rodata : {
    *(.rodata)
  } > ram

  .data : {
    *(.data)
  } > ram

  .bss : {
    *(.bss)
  } > ram

  . = ALIGN(8);
  . = . + 4096;
  PROVIDE(_STACK_PTR = .);
  . = . + 32;
  PROVIDE(ekernel = .);
}
