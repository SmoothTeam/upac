// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

//! `#[derive(RedbCodec)]` — generates `encode_into()`/`decode_from()` for
//! storing a struct as a `redb` value via the crate's own byte layout.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, PathSegment, Type, TypeArray, parse_macro_input};

fn array_codec(ident: &Ident, ty: &Type, array: &TypeArray) -> (TokenStream2, TokenStream2) {
    let len = &array.len;

    let encode = quote! {
        buf.extend_from_slice(&value.#ident);
    };
    let decode = quote! {
        let #ident: #ty = data[*offset..*offset + (#len)].try_into().unwrap();
        *offset += #len;
    };

    (encode, decode)
}

fn string_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! { crate::codec::write_len_prefixed(buf, value.#ident.as_bytes()); },
        quote! { let #ident = crate::codec::read_str(data, offset); },
    )
}

fn u32_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! { crate::codec::write_u32(buf, value.#ident); },
        quote! { let #ident = crate::codec::read_u32(data, offset); },
    )
}

fn u64_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! { crate::codec::write_u64(buf, value.#ident); },
        quote! { let #ident = crate::codec::read_u64(data, offset); },
    )
}

fn bool_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! { crate::codec::write_bool(buf, value.#ident); },
        quote! { let #ident = crate::codec::read_bool(data, offset); },
    )
}

fn option_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! { crate::codec::write_opt_str(buf, value.#ident.as_deref()); },
        quote! { let #ident = crate::codec::read_opt_str(data, offset); },
    )
}

fn vec_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! { crate::codec::write_vec_u32(buf, &value.#ident); },
        quote! { let #ident = crate::codec::read_vec_u32(data, offset); },
    )
}

fn composite_codec(ident: &Ident, ty: &Type) -> (TokenStream2, TokenStream2) {
    (
        quote! { #ty::encode_into(buf, &value.#ident); },
        quote! { let #ident = #ty::decode_from(data, offset); },
    )
}

fn field_path_codec(ident: &Ident, segment: &PathSegment, ty: &Type) -> (TokenStream2, TokenStream2) {
    match segment.ident.to_string().as_str() {
        "String" => string_codec(ident),
        "u32" => u32_codec(ident),
        "u64" => u64_codec(ident),
        "bool" => bool_codec(ident),
        "Option" => option_codec(ident),
        "Vec" => vec_codec(ident),
        _ => composite_codec(ident, ty),
    }
}

fn field_codec(ident: &Ident, ty: &Type) -> (TokenStream2, TokenStream2) {
    if let Type::Array(array) = ty {
        return array_codec(ident, ty, array);
    }

    let Type::Path(type_path) = ty else {
        let error = quote! { compile_error!("RedbCodec: unsupported field type"); };
        return (error.clone(), error);
    };

    let Some(segment) = type_path.path.segments.last() else {
        let error = quote! { compile_error!("RedbCodec: unsupported field type"); };
        return (error.clone(), error);
    };

    field_path_codec(ident, segment, ty)
}

fn codec_impl(name: &Ident, encodes: &[TokenStream2], decodes: &[TokenStream2], names: &[Ident]) -> TokenStream2 {
    quote! {
        impl #name {
            pub fn encode_into(buf: &mut Vec<u8>, value: &#name) {
                #(#encodes)*
            }

            pub fn decode_from(data: &[u8], offset: &mut usize) -> #name {
                #(#decodes)*

                #name {
                    #(#names),*
                }
            }
        }
    }
}

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Error::new_spanned(name, "RedbCodec only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "RedbCodec only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut encodes = Vec::new();
    let mut decodes = Vec::new();
    let mut names = Vec::new();

    for filed in fields {
        let Some(ident) = filed.ident.as_ref() else {
            return Error::new_spanned(filed, "RedbCodec only supports named fields")
                .to_compile_error()
                .into();
        };
        let (encode, decode) = field_codec(ident, &filed.ty);

        names.push(ident.clone());
        encodes.push(encode);
        decodes.push(decode);
    }

    codec_impl(name, &encodes, &decodes, &names).into()
}
