use crate::prelude::*;
use serde::{Deserialize, Serialize};
// use serde_derive::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct Symbol {
    addr: u64,
    line: i32,
}
enum SymFmt {
    JSON,
    Postcard,
    EfficentPostcard,
}
ezy_static! { SYMTAB, Option<BTreeMap<u64, String>>, None }
pub fn load_ksymmap(name: String, symmap: &[u8]) {
    let mut sfmt: Option<SymFmt> = None;
    if name.ends_with(".pcrd") {
        sfmt = Some(SymFmt::Postcard);
    }
    if name.ends_with(".cpost") {
        sfmt = Some(SymFmt::EfficentPostcard);
    }
    if name.ends_with(".json") {
        sfmt = Some(SymFmt::JSON);
    }
    if sfmt.is_none() {
        println!("Unknown ksymmap format");
        return;
    }
    let deser: BTreeMap<String, Vec<Symbol>> = match sfmt.unwrap() {
        SymFmt::JSON => serde_json::from_slice(&symmap).unwrap(),
        SymFmt::Postcard => postcard::from_bytes(&symmap).unwrap(),
        SymFmt::EfficentPostcard => {
            *SYMTAB.get() = postcard::from_bytes(&symmap).unwrap();
            return;
        }
    };
    let mut symtab = BTreeMap::<u64, String>::new();
    for (k, v) in deser {
        for sym in v {
            symtab.insert(sym.addr, k.clone() + &":" + &sym.line.to_string());
        }
    }
    *SYMTAB.get() = Some(symtab);
}
pub fn addr2sym(addr: u64) -> Option<String> {
    let p = &*SYMTAB.get();
    match p {
        Some(stab) => {
            let mut addr = addr;
            if addr > 0xf00000000 {
                loop {
                    if addr < 0xf00000000 {
                        break;
                    }
                    match stab.get(&addr) {
                        Some(r) => {
                            return Some(r.clone());
                        }
                        None => {
                            addr -= 1;
                        }
                    }
                }
            }
        }
        None => {}
    }
    None
}
pub fn addr_fmt(addr: u64, fcnname: Option<String>) {
    let loc = addr2sym(addr);
    let part = if loc.is_some() {
        loc.unwrap()
    } else {
        "???".to_string()
    };
    let part2 = if fcnname.is_some() {
        fcnname.unwrap()
    } else {
        "???".to_string()
    };
    println!(" at {:p} {} in {}", addr as *const u8, part, part2);
}
