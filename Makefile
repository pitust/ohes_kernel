run: build/oh_es.iso
	qemu-system-x86_64 -hda build/oh_es.iso -serial stdio -s -hdb build/initrd.cpio -debugcon file:logz.txt -global isa-debugcon.iobase=0x402
build/oh_es.iso: build/kernel.elf build/initrd.cpio cfg/grub.cfg
	rm -rf iso
	mkdir -p iso/boot/grub
	cp cfg/grub.cfg iso/boot/grub
	cp build/kernel.elf iso/boot
	grub-mkrescue -o build/oh_es.iso iso

build/kernel.elf: target/x86_64-unknown-none/debug/liban_os.a build/boot.o
	ld.lld target/x86_64-unknown-none/debug/liban_os.a /opt/cross/lib/gcc/x86_64-elf/10.2.0/libgcc.a --allow-multiple-definition -T/home/pitust/code/an_os/link.ld build/boot.o  -o build/kernel.elf -n
	grub-file --is-x86-multiboot2 build/kernel.elf
build/test.elf: build/test.o
	ld -T user/user.ld build/test.o -o build/test.elf
target/x86_64-unknown-none/debug/liban_os.a: faux build/test.elf
	cargo build --features "fini_exit debug_logs"
build/initrd.cpio: $(wildcard initrd/*) build/kernel.elf initrd/ksymmap.pcrd
	sh create-initrd.sh
initrd/ksymmap.pcrd: build/ksymmap.pcrd
	cp build/ksymmap.pcrd initrd
build/ksymmap.pcrd: build/kernel.elf
	ts-node -T sym-city.ts
build/boot.o: asm/boot.s
	nasm asm/boot.s -f elf64 -o build/boot.o
build/test.o: user/test.s
	nasm -felf64 user/test.s -o build/test.o
faux:
ffonts:
	wget https://fonts.gstatic.com/s/robotomono/v12/L0xuDF4xlVMF-BfR8bXMIhJHg45mwgGEFl0_3vq_S-W4Ep0.woff2 -Ofonts/roboto.woff2
	woff2_decompress fonts/roboto.woff2
.PHONY: faux run ffonts