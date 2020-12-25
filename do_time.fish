for CHAR in (seq 20)
  sh -c 'time qemu-system-x86_64 -hda build/oh_es.iso -s -debugcon file:logz.txt -global isa-debugcon.iobase=0x402 -device isa-debug-exit,iobase=0xf4,iosize=0x04 -accel kvm -cpu host -vnc :1 -monitor none -serial stdio -m 1G' 2>>timings.txt || echo 'ABORTED' >>timings.txt
end
