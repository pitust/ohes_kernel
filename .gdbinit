set disassembly-flavor intel
add-symbol-file build/kernel.elf
add-symbol-file build/test.elf
target remote :1234
c
