set disassembly-flavor intel
add-symbol-file build/debugkernel.elf
add-symbol-file rootfs/bin/init
target remote :1234
alias break=hbreak
c