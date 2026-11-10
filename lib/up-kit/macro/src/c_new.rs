// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

use crate::common::is_out_field;

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
    let mut field_values: Vec<TokenStream2> = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return Error::new_spanned(field, "CNew only supports named fields")
                .to_compile_error()
                .into();
        };

        if ident == "struct_size" {
            continue;
        }

        if is_out_field(field) {
            field_values.push(quote! { #ident: Default::default() });
            continue;
        }

        let ty = &field.ty;
        params.push(quote! { #ident: #ty });
        field_values.push(quote! { #ident });
    }

    quote! {
        impl #name {
            #[allow(clippy::too_many_arguments)]
            pub fn new(#(#params),*) -> Self {
                Self {
                    struct_size: ::std::mem::size_of::<#name>(),
                    #(#field_values),*
                }
            }
        }
    }
    .into()
}
