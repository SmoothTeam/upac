// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! `#[derive(RedbCodec)]` — generates an `upac_types::codec::RedbCodable` impl for storing a
//! struct as a `redb` value via the crate's own byte layout. Field dispatch is uniform: every
//! field just calls its own type's `RedbCodable` impl, resolved by the real Rust compiler (not
//! this macro) from the field's declared type — so `String`/`u32`/`u64`/`bool`/`Option<T>`/
//! `Vec<T>`/any other `RedbCodable` composite all just work without this macro ever needing to
//! special-case a field's type by name.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, Type, TypeArray, parse_macro_input};

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

fn codec_impl(name: &Ident, encodes: &[TokenStream2], decodes: &[TokenStream2], names: &[Ident]) -> TokenStream2 {
    quote! {
        impl crate::codec::RedbCodable for #name {
            fn redb_encode(&self, buf: &mut Vec<u8>) {
                #(#encodes)*
            }

            fn redb_decode(data: &[u8], offset: &mut usize) -> #name {
                #(#decodes)*

                #name {
                    #(#names),*
                }
            }
        }
    }
}

fn array_codec(ident: &Ident, ty: &Type, array: &TypeArray) -> (TokenStream2, TokenStream2) {
    let len = &array.len;

    let encode = quote! {
        buf.extend_from_slice(&self.#ident);
    };
    let decode = quote! {
        let #ident: #ty = data[*offset..*offset + (#len)].try_into().unwrap();
        *offset += #len;
    };

    (encode, decode)
}

fn field_codec(ident: &Ident, ty: &Type) -> (TokenStream2, TokenStream2) {
    if let Type::Array(array) = ty {
        return array_codec(ident, ty, array);
    }

    let encode = quote! { crate::codec::RedbCodable::redb_encode(&self.#ident, buf); };
    let decode = quote! { let #ident: #ty = crate::codec::RedbCodable::redb_decode(data, offset); };

    (encode, decode)
}
