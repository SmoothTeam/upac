// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! `#[derive(CEnum)]` — for a fieldless `#[repr(uN)]` enum carried over the
//! C ABI as its raw integer, generates `TryFrom<uN>` (unknown values become
//! `ErrorKind::InvalidEntry`) and `From<Enum> for uN`.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Fields, Ident, parse_macro_input};

const REPRS: &[&str] = &["u8", "u16", "u32"];

fn repr_type(attrs: &[Attribute]) -> Option<Ident> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("repr"))
        .find_map(|attr| attr.parse_args::<Ident>().ok())
        .filter(|repr| REPRS.contains(&repr.to_string().as_str()))
}

fn c_enum_impl(name: &Ident, repr: &Ident, arms: &[TokenStream2]) -> TokenStream2 {
    quote! {
        impl TryFrom<#repr> for #name {
            type Error = crate::error::ErrorKind;

            fn try_from(value: #repr) -> Result<Self, crate::error::ErrorKind> {
                match value {
                    #(#arms)*
                    _ => Err(crate::error::ErrorKind::InvalidEntry),
                }
            }
        }

        impl From<#name> for #repr {
            fn from(value: #name) -> Self {
                value as #repr
            }
        }
    }
}

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return Error::new_spanned(name, "CEnum only supports enums")
                .to_compile_error()
                .into();
        }
    };

    let Some(repr) = repr_type(&input.attrs) else {
        return Error::new_spanned(name, "CEnum requires #[repr(u8)], #[repr(u16)] or #[repr(u32)]")
            .to_compile_error()
            .into();
    };

    let mut arms = Vec::new();

    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Error::new_spanned(variant, "CEnum only supports fieldless variants")
                .to_compile_error()
                .into();
        }

        let variant_ident = &variant.ident;
        arms.push(quote! {
            value if value == Self::#variant_ident as #repr => Ok(Self::#variant_ident),
        });
    }

    c_enum_impl(name, &repr, &arms).into()
}
