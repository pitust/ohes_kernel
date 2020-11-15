import { execSync } from "child_process";
import { readFileSync, unlinkSync, writeFileSync } from "fs";

// GET DA SYMBOLZ
execSync('objdump --dwarf=decodedline build/kernel.elf >data.txt', { stdio: 'inherit' })
let o = readFileSync('data.txt').toString().trim();
unlinkSync('data.txt');
let cfnm = 'NONE';
let data = {};
for (let l of o.split('\n')) {
    if (l.startsWith('CU: ')) cfnm = l.slice(4, -1).trim();
    else if (l.endsWith(':')) cfnm = l.slice(0, -1).trim();
    else if (l.trim() && cfnm != 'NONE') {
        let [_, lineno, addr] = l.split(/\s/).map(e => e.trim()).filter(e => e);
        if (+addr > 0x200000) {
            data[cfnm] = data[cfnm] || [];
            data[cfnm].push({ addr: +addr, line: +lineno });
        }
    }
}
writeFileSync('build/ksymmap.json', JSON.stringify(data));
execSync('serde_conv -i build/ksymmap.json  -F postcard -o build/ksymmap.pcrd');
execSync('serde_conv -i build/ksymmap.json  -F cpost -o build/ksymmap.epcrd');