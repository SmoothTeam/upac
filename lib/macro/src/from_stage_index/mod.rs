// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

//! `#[derive(FromStageIndex)]` — generates `from_stage_index(usize) -> Self`
//! for a fieldless enum, mapping an orchestrator stage index to the variant
//! at that DECLARATION position (not by explicit discriminant).

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new_spanned(name, "FromStageIndex only supports enums")
                .to_compile_error()
                .into();
        }
    };

    let mut arms = Vec::new();

    for (index, variant) in variants.iter().enumerate() {
        if !matches!(variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(variant, "FromStageIndex only supports fieldless variants")
                .to_compile_error()
                .into();
        }

        let variant_ident = &variant.ident;
        arms.push(quote! {
            #index => Self::#variant_ident,
        });
    }

    let expanded = quote! {
        impl #name {
            pub fn from_stage_index(index: usize) -> Self {
                match index {
                    #(#arms)*
                    _ => unreachable!(),
                }
            }
        }
    };

    expanded.into()
}
