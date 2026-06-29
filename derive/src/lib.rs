//! Proc-macro crate for UPAC. Provides #[derive(CFree)], generating an unsafe
//! `free()` that releases every owned buffer a C-ABI struct holds. This is the
//! reflection-over-fields that Zig got from `inline for (std.meta.fields)`.
//!
//! Dispatch is by field TYPE, decided at compile time:
//!   CSlice      -> free_cslice(&self.field)
//!   CArray<T>   -> free_carray(&self.field)
//!   CVersion    -> self.field.free()   (composite frees itself)
//!   other (u32, [u8;32], bool, enums) -> owns nothing, skipped
//! Add a new owned field and it's handled automatically — no list to maintain.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

#[proc_macro_derive(CFree)]
pub fn derive_cfree(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(
                    name,
                    "CFree only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "CFree only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut frees = Vec::new();

    for f in fields {
        let ident = f.ident.as_ref().unwrap();
        if let Type::Path(tp) = &f.ty {
            if let Some(seg) = tp.path.segments.last() {
                match seg.ident.to_string().as_str() {
                    "CSlice" => frees.push(quote! {
                        free_cslice(&self.#ident);
                    }),
                    "CArray" => frees.push(quote! {
                        free_carray(&self.#ident);
                    }),
                    "CVersion" => frees.push(quote! {
                        self.#ident.free();
                    }),
                    _ => {}
                }
            }
        }
    }

    let expanded = quote! {
        impl #name {
            pub unsafe fn free(&self) {
                #(#frees)*
            }
        }
    };

    expanded.into()
}
