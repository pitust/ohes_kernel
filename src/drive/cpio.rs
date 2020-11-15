use crate::{dbg, print, println};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
#[derive(Debug, Clone)]
pub struct CPIOEntry {
    pub magic: u64,
    pub dev: u64,
    pub ino: u64,
    pub mode: u64,
    pub uid: u64,
    pub gid: u64,
    pub nlink: u64,
    pub rdev: u64,
    pub mtime: u64,
    pub namesize: u64,
    pub filesize: u64,
    pub name: String,
    pub data: Vec<u8>,
}

pub fn chars2int(c: &[u8]) -> u64 {
    let mut res = 0u64;
    for i in c {
        if *i < 48 {
            res *= 8;
            continue;
        }
        res *= 8;
        res += (*i as u64) - 48;
    }
    res
}

pub fn parse_one(drv: &mut crate::drive::Offreader) -> Result<CPIOEntry, String> {
    let magic = chars2int(drv.read_consume(6)?.as_slice());
    let dev = chars2int(drv.read_consume(6)?.as_slice());
    let ino = chars2int(drv.read_consume(6)?.as_slice());
    let mode = chars2int(drv.read_consume(6)?.as_slice());
    let uid = chars2int(drv.read_consume(6)?.as_slice());
    let gid = chars2int(drv.read_consume(6)?.as_slice());
    let nlink = chars2int(drv.read_consume(6)?.as_slice());
    let rdev = chars2int(drv.read_consume(6)?.as_slice());
    let mtime = chars2int(drv.read_consume(11)?.as_slice());
    let nd = drv.read_consume(6)?;
    let nds = nd.as_slice();
    let namesize = chars2int(nds);
    let filesize = chars2int(drv.read_consume(11)?.as_slice());
    let name = String::from_utf8(
        drv.read_consume(namesize)?
            .split_at((namesize - 1) as usize)
            .0
            .to_vec(),
    )
    .unwrap();
    let data = drv.read_consume(filesize)?;
    if name == "TRAILER!!!" {
        return Err("EOF".to_string());
    }
    Ok(CPIOEntry {
        magic,
        dev,
        ino,
        mode,
        uid,
        gid,
        nlink,
        rdev,
        mtime,
        namesize,
        filesize,
        name,
        data,
    })
}
pub fn parse(drv: &mut crate::drive::Offreader) -> Result<Vec<CPIOEntry>, String> {
    let mut rv = vec![];
    loop {
        let x = parse_one(drv);
        match x {
            Ok(o) => {
                rv.push(o);
            }
            Err(e) => {
                if rv.len() == 0 {
                    return Err(e);
                }
                return Ok(rv);
            }
        }
    }
}
