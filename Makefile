run: build/oh_es.iso build/data.img
	qemu-system-x86_64 -hda build/oh_es.iso -s -debugcon file:logz.txt -global isa-debugcon.iobase=0x402 -accel kvm -cpu host -vnc :1 -monitor none -serial stdio -m 1G

build/oh_es.iso: build/debugkernel.elf build/releasekernel.elf cfg/grub.cfg
	rm -rf iso
	mkdir -p iso/boot/grub
	cp cfg/grub.cfg iso/boot/grub
	cp build/debugkernel.elf iso/boot
	cp build/releasekernel.elf iso/boot
	grub-mkrescue -o build/oh_es.iso iso

build/debugkernel.elf: target/x86_64-unknown-none/debug/liban_os.a build/boot.o
	ld.lld target/x86_64-unknown-none/debug/liban_os.a /opt/cross/lib/gcc/x86_64-elf/10.2.0/libgcc.a --allow-multiple-definition -T/home/pitust/code/an_os/link.ld build/boot.o  -o build/debugkernel.elf -n
	grub-file --is-x86-multiboot build/debugkernel.elf

build/releasekernel.elf: target/x86_64-unknown-none/debug/liban_os.a build/boot.o
	ld.lld target/x86_64-unknown-none/debug/liban_os.a /opt/cross/lib/gcc/x86_64-elf/10.2.0/libgcc.a --allow-multiple-definition -T/home/pitust/code/an_os/link.ld build/boot.o  -o build/releasekernel.elf -n
	strip build/releasekernel.elf
	# sstrip build/releasekernel.elf
	grub-file --is-x86-multiboot build/releasekernel.elf

build/test.elf: build/test.o
	ld -T user/user.ld build/test.o -o build/test.elf
target/x86_64-unknown-none/debug/liban_os.a: faux
	cargo build --features "fini_exit debug_logs address_cleaner"
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
build/data.img: build/data.note $(shell find rootfs)
	mke2fs \
		-L 'ohes-sysroot' \
		-N 0 \
		-O ^64bit \
		-d rootfs \
		-m 5 \
		-r 1 \
		-t ext2 \
		"build/ext.img" \
		128M

	dd conv=notrunc if=build/ext.img of=build/data.img bs=512 seek=4048 status=progress
build/data.note:
	dd if=/dev/zero bs=512K count=280 of=build/data.img status=progress
	dd if=/dev/zero bs=512K count=256 of=build/ext.img status=progress
	sfdisk build/data.img <cfg/disklayout.sfdsk
	touch build/data.note
faux:
ffonts:
	wget https://fonts.gstatic.com/s/robotomono/v12/L0xuDF4xlVMF-BfR8bXMIhJHg45mwgGEFl0_3vq_S-W4Ep0.woff2 -Ofonts/roboto.woff2
	woff2_decompress fonts/roboto.woff2
.PHONY: faux run ffonts