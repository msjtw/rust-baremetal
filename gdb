set architecture riscv:rv32
target remote :1234
layout asm
layout reg
until *0x80000000
break *0x80000014
