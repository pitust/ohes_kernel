#![feature(type_name_of_val)]
extern crate proc_macro2;
extern crate syn;
extern crate quote;
use syn::{parse_macro_input};
use syn::ItemFn;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn register(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let inp= parse_macro_input!(item as ItemFn);
    let block = inp.block.as_ref();
    let attrs = inp.attrs.clone();
    let sig = inp.sig.clone();
    let vis = inp.vis.clone();
    let ident = sig.ident.clone();
    let expanded = quote! {
        #vis #sig {
            crate::backtrace::mark(#ident as *const (), &(alloc::string::String::from(stringify!(#ident)) + " @ " + file!() + ":" + &crate::i2s(line!())));
            #block
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