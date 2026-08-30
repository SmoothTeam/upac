// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! `#[derive(CNew)]` — generates a `new(...)` constructor for a C-ABI struct. Every field except
//! `struct_size` becomes a parameter, in declaration order; `struct_size` itself is computed via
//! `size_of::<Self>()` rather than taken as a parameter, so callers never need to import
//! `std::mem::size_of` just to build a request/response struct.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Error::new_spanned(name, "CNew only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "CNew only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut params: Vec<TokenStream2> = Vec::new();
    let mut field_idents = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return Error::new_spanned(field, "CNew only supports named fields")
                .to_compile_error()
                .into();
        };

        if ident == "struct_size" {
            continue;
        }

        let ty = &field.ty;
        params.push(quote! { #ident: #ty });
        field_idents.push(ident);
    }

    quote! {
        impl #name {
            // Every C-ABI request/response struct field (minus struct_size) becomes a
            // constructor parameter by design — this isn't the "should refactor into a builder"
            // code smell clippy's default threshold is meant to catch.
            #[allow(clippy::too_many_arguments)]
            pub fn new(#(#params),*) -> Self {
                Self {
                    struct_size: ::std::mem::size_of::<#name>(),
                    #(#field_idents),*
                }
            }
        }
    }
    .into()
}
