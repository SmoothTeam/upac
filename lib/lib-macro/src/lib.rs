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
use proc_macro2::TokenStream as TokenStream2;
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
                return syn::Error::new_spanned(name, "CFree only supports structs with named fields")
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

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        if let Type::Path(tp) = &field.ty {
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

fn field_codec(ident: &syn::Ident, ty: &Type) -> (TokenStream2, TokenStream2) {
    if let Type::Array(array) = ty {
        let len = &array.len;

        let encode = quote! {
            buf.extend_from_slice(&value.#ident);
        };
        let decode = quote! {
            let #ident: #ty = data[*offset..*offset + (#len)].try_into().unwrap();
            *offset += #len;
        };

        return (encode, decode);
    }

    let Type::Path(type_path) = ty else {
        let error = quote! { compile_error!("RedbCodec: unsupported field type"); };
        return (error.clone(), error);
    };

    let segment = type_path.path.segments.last().unwrap();

    match segment.ident.to_string().as_str() {
        "String" => (
            quote! { crate::database::codec::write_len_prefixed(buf, value.#ident.as_bytes()); },
            quote! { let #ident = crate::database::codec::read_str(data, offset); },
        ),
        "u32" => (
            quote! { crate::database::codec::write_u32(buf, value.#ident); },
            quote! { let #ident = crate::database::codec::read_u32(data, offset); },
        ),
        "u64" => (
            quote! { crate::database::codec::write_u64(buf, value.#ident); },
            quote! { let #ident = crate::database::codec::read_u64(data, offset); },
        ),
        "bool" => (
            quote! { crate::database::codec::write_bool(buf, value.#ident); },
            quote! { let #ident = crate::database::codec::read_bool(data, offset); },
        ),
        "Option" => (
            quote! { crate::database::codec::write_opt_str(buf, value.#ident.as_deref()); },
            quote! { let #ident = crate::database::codec::read_opt_str(data, offset); },
        ),
        "Vec" => (
            quote! { crate::database::codec::write_vec_u32(buf, &value.#ident); },
            quote! { let #ident = crate::database::codec::read_vec_u32(data, offset); },
        ),
        _ => (
            quote! { #ty::encode_into(buf, &value.#ident); },
            quote! { let #ident = #ty::decode_from(data, offset); },
        ),
    }
}

#[proc_macro_derive(RedbCodec)]
pub fn derive_redb_codec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(name, "RedbCodec only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "RedbCodec only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut encodes = Vec::new();
    let mut decodes = Vec::new();
    let mut names = Vec::new();

    for filed in fields {
        let ident = filed.ident.as_ref().unwrap();
        let (encode, decode) = field_codec(ident, &filed.ty);

        names.push(ident.clone());
        encodes.push(encode);
        decodes.push(decode);
    }

    let expanded = quote! {
        impl #name {
            pub(crate) fn encode_into(buf: &mut Vec<u8>, value: &#name) {
                #(#encodes)*
            }

            pub(crate) fn decode_from(data: &[u8], offset: &mut usize) -> #name {
                #(#decodes)*

                #name {
                    #(#names),*
                }
            }
        }
    };

    expanded.into()
}
