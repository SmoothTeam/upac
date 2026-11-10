// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, parse_macro_input};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return Error::new_spanned(name, "StageKey only supports enums")
                .to_compile_error()
                .into();
        }
    };

    let mut arms = Vec::new();

    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Error::new_spanned(variant, "StageKey only supports fieldless variants")
                .to_compile_error()
                .into();
        }

        let variant_ident = &variant.ident;
        let key = format!("stage-{}", to_kebab_case(&variant_ident.to_string()));
        arms.push(quote! {
            Self::#variant_ident => #key,
        });
    }

    stage_key_impl(name, &arms).into()
}

fn to_kebab_case(name: &str) -> String {
    let mut result = String::new();

    for (index, character) in name.chars().enumerate() {
        if character.is_uppercase() {
            if index != 0 {
                result.push('-');
            }
            result.extend(character.to_lowercase());
        } else {
            result.push(character);
        }
    }

    result
}

fn stage_key_impl(name: &Ident, arms: &[TokenStream2]) -> TokenStream2 {
    quote! {
        impl #name {
            pub fn stage_key(&self) -> &'static str {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}
