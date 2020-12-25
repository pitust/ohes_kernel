import { exec } from 'child_process';
import { createServer } from 'net';
createServer(s => {
    let t;
    console.log('conn')
    let total = ''
    s.on('data', d => {
        total += d.toString();
        if (total.includes('KO')) { t = Date.now(); total = ''; }
    })
    s.on('end', () => {
        console.log(`${Date.now() - t}`);
        exec('qemu-system-x86_64 -hda build/oh_es.iso -s -debugcon telnet:localhost:3400 -global isa-debugcon.iobase=0x402 -accel kvm -cpu host -monitor none -serial stdio -m 1G');
    })
}).listen(3400);
exec('qemu-system-x86_64 -hda build/oh_es.iso -s -debugcon telnet:localhost:3400 -global isa-debugcon.iobase=0x402 -accel kvm -cpu host -monitor none -serial stdio -m 1G');