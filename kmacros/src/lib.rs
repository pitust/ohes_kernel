#![feature(type_name_of_val)]
extern crate proc_macro2;
extern crate quote;
extern crate syn;
use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;
use syn::{parse_macro_input, parse_str, ExprMatch};

fn getfilez(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    assert!(dir.exists());
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                getfilez(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}
// fn getdirz(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
//     if dir.is_dir() {
//         for entry in fs::read_dir(dir)? {
//             let entry = entry?;
//             let path = entry.path();
//             if path.is_dir() {
//                 getfilez(&path, cb)?;
//                 cb(&entry);
//             }
//         }
//     }
//     Ok(())
// }

#[proc_macro_attribute]
pub fn handle_read(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut v = vec![];
    getfilez(Path::new("rootfs"), &mut |d| {
        let d: &DirEntry = d;
        let p = d.path();
        let p = p
            .as_os_str()
            .to_str()
            .unwrap()
            .split("rootfs/")
            .nth(1)
            .unwrap();
        v.push(p.to_string());
    })
    .unwrap();

    let inp = parse_macro_input!(item as ItemFn);
    let attrs = inp.attrs.clone();
    let sig = inp.sig.clone();
    let vis = inp.vis.clone();
    let mut o = format!("match path {{ ");
    for k in &v {
        o = format!(
            "{old} {src:?} | {src2:?} => {{ include_bytes!({file:?}) }}, ",
            old = o,
            src = k,
            src2 = format!("/{}", k),
            file = format!("../rootfs/{}", k)
        );
    }
    o = format!(
        "{} _ => panic!(\"File not found (used {{}}), have: {{}} !!!\", path, {:?}) }}",
        o,
        format!("{:?}", v)
    );

    let blk = parse_str::<ExprMatch>(&o).expect(&format!("eYYY: {}", o));
    let expanded = quote! {
        #vis #sig {
            #blk
        }
    };
    let x: proc_macro2::TokenStream = proc_macro2::TokenStream::from(expanded);
    let mut p: String = String::from("");
    for idx in 0..attrs.len() {
        let attr = attrs[idx].clone();
        let z: proc_macro2::TokenStream = quote! { #attr };
        p += &z.to_string();
    }
    p += &x.to_string();
    p.parse().expect("Should parse")
}
