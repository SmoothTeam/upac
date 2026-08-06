// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

//! `#[derive(CToRust)]` — generates `impl From<&CRust> for Rust`, converting
//! a C-ABI struct into an owned Rust domain type without validation
//! (infallible inbound direction, for all-primitive structs).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Fields, Ident, PathSegment, Type, parse_macro_input};

use crate::common::{PRIMITIVES, SHARED_TYPES};

fn primitive_from_c(ident: &Ident) -> TokenStream2 {
    quote! { value.#ident }
}

fn composite_from_c(ident: &Ident, name: &str) -> TokenStream2 {
    let rust_ty = format_ident!("{name}");
    quote! { #rust_ty::from(&value.#ident) }
}

fn field_path_from_c(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    match segment.ident.to_string().as_str() {
        name if PRIMITIVES.contains(&name) || SHARED_TYPES.contains(&name) => primitive_from_c(ident),
        name => composite_from_c(ident, name),
    }
}

fn field_from_c_infallible(ident: &Ident, ty: &Type) -> TokenStream2 {
    if let Type::Array(_) = ty {
        return quote! { value.#ident };
    }

    let Type::Path(type_path) = ty else {
        return quote! { compile_error!("CToRust: unsupported field type") };
    };

    match type_path.path.segments.last() {
        Some(segment) => field_path_from_c(ident, segment),
        None => quote! { compile_error!("CToRust: unsupported field type") },
    }
}

fn from_impl(name: &Ident, c_name: &Ident, field_values: &[TokenStream2]) -> TokenStream2 {
    quote! {
        impl From<&#c_name> for #name {
            fn from(value: &#c_name) -> Self {
                #name {
                    #(#field_values)*
                }
            }
        }
    }
}

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let c_name = format_ident!("C{name}");

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Error::new_spanned(name, "CToRust only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "CToRust only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut field_values = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return Error::new_spanned(field, "CToRust only supports named fields")
                .to_compile_error()
                .into();
        };

        let value = field_from_c_infallible(ident, &field.ty);
        field_values.push(quote! { #ident: #value, });
    }

    from_impl(name, &c_name, &field_values).into()
}
