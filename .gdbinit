set disassembly-flavor intel
add-symbol-file build/kernel.elf
add-symbol-file rootfs/bin/kinfo
target remote :1234
alias break=hbreak
c