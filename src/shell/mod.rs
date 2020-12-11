use crate::{drive::cpio::CPIOEntry, prelude::*};
struct Z {
    pub x: usize,
}
const ADDR: u16 = 0x1f0;
ezy_static! { CPIO, alloc::vec::Vec<CPIOEntry>, crate::drive::cpio::parse(
    &mut unsafe { crate::drive::Drive::new(true, ADDR, 0x3f6) }.get_offreader(),
).unwrap() }
fn ls(cmd: String) {
    {
        let mut file = "/".to_string() + &cmd + "/";
        if file == "" {
            file = "/".to_string();
        }
        file = file.replace("/./", "/");
        file = file.replace("//", "/");
        file = file.replace("//", "/");
        file = file.replace("//", "/");

        println!("Listing of {}", file);

        let entries = CPIO.get();
        for e in &*entries {
            if ("/".to_string() + e.name.clone().as_str()).starts_with(&(file.clone())) {
                println!("  {} | sz={} | perms={}", e.name, e.filesize, e.mode);
            }
        }
    }
}
pub fn prompt() {
    println!("+-------------------+");
    println!("| OhEs Shell v0.2.0 |");
    println!("+-------------------+");
}
fn cat(file: String) {
    let entries = CPIO.get();
    let mut is_ok = false;
    for e in &*entries {
        if e.name == file || (e.name.clone() + "/") == file {
            println!("{}", String::from_utf8(e.data.clone()).unwrap());
            is_ok = true;
            break;
        }
    }
    if !is_ok {
        println!("Error: ENOENT");
    }
}
fn loadksymmap(file: String) {
    let entries = CPIO.get();
    let mut is_ok = false;
    for e in &*entries {
        if e.name == file || (e.name.clone() + "/") == file {
            ksymmap::load_ksymmap(e.name.clone(), e.data.clone().as_slice());
            is_ok = true;
            break;
        }
    }
    if !is_ok {
        println!("Error: ENOENT");
    }
}
fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}
pub async fn shell() {
    println!("Shell!");
    prompt();
    let mut ch: Vec<String> = vec!["prompt".to_string()];

    let cmds_m = Mutex::new(vec!["ls", "cat", "exit", "prompt", "cls", "pci"]);
    let mut cmd_h = BTreeMap::<String, Box<dyn Fn<(), Output = ()>>>::new();
    let input: Mutex<String> = Mutex::new("<unk>".to_string());
    macro_rules! cmd {
        ( $code:expr ) => {
            cmds_m.get().push(stringify!($code));
            cmd_h.insert(
                stringify!($code).to_string(),
                Box::new(|| ($code)(input.get().clone())),
            );
        };
    }
    macro_rules! ecmd {
        ( $name:ident, $code:expr ) => {
            cmds_m.get().push(stringify!($name));
            cmd_h.insert(
                stringify!($name).to_string(),
                Box::new(|| {
                    $code;
                }),
            );
        };
    }
    cmd!(ls);
    cmd!(cat);
    cmd!(loadksymmap);
    ecmd!(cls, Printer.clear_screen());
    ecmd!(sup, Printer.scroll_up());
    ecmd!(prompt, prompt());
    ecmd!(panic, panic!("You asked for it..."));
    ecmd!(user, crate::userland::loaduser());
    ecmd!(gptt, drive::gpt::test0());
    ecmd!(pci, crate::pci::testing());
    
    loop {
        print!("\x1b[44m\x1b[30m ~ \x1b[0m\x1b[34m\u{e0b0}\x1b[0m ");
        let im = Mutex::new(Z { x: ch.len() });
        let result: String = input!(
            || {
                let mut i = im.get().x;
                if i == 0 {
                    return None;
                }
                i -= 1;
                im.get().x = i;
                Some(ch[i].clone())
            },
            || {
                let mut i = im.get().x;
                if i == ch.len() {
                    return None;
                }
                i += 1;
                im.get().x = i;
                if i == ch.len() {
                    return Some("".to_string());
                }
                Some(ch[i].clone())
            },
            |s: String| {
                let mut can_suggest: Option<String> = None;
                let cmds = cmds_m.get();
                for opt in cmds.clone().into_iter() {
                    if can_suggest.is_none() && opt.starts_with(&s) {
                        can_suggest = Some(opt.to_string().clone());
                    } else if can_suggest.is_some() && opt.starts_with(&s) {
                        let sug = can_suggest.clone().unwrap();
                        let lene = min(sug.len(), opt.len());
                        let sc = sug.clone();
                        let mut s_iter = sc.chars();
                        let mut o_iter = opt.clone().chars();
                        let mut mxi = 0;
                        for _i in 0..lene {
                            if s_iter.next() == o_iter.next() {
                                mxi += 1;
                            } else {
                                break;
                            }
                        }
                        can_suggest = Some(sc.split_at(mxi).0.to_string());
                    }
                }
                can_suggest
            }
        )
        .await;
        ch.push(result.clone());
        let cmd = result.split(' ').next().unwrap();
        *input.get() = result
            .clone()
            .split_at(cmd.len())
            .1
            .to_string()
            .trim()
            .to_string();

        match cmd {
            "exit" => {
                return;
            }
            "cat" => {}
            _ => match cmd_h.get(cmd) {
                Some(handler) => {
                    handler();
                }
                None => {
                    println!("Unknown command: {}", cmd);
                }
            },
        }
    }
}
