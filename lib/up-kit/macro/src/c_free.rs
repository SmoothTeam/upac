// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, PathSegment, Type, parse_macro_input};

use crate::common::{generic_arg, is_validatable_composite, segment_name};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Error::new_spanned(name, "CFree only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "CFree only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut frees = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return Error::new_spanned(field, "CFree only supports named fields")
                .to_compile_error()
                .into();
        };

        frees.push(free_field(ident, &field.ty));
    }

    free_impl(name, &frees).into()
}

fn free_cslice(ident: &Ident) -> TokenStream2 {
    quote! { self.#ident.free(); }
}

fn free_composite(ident: &Ident) -> TokenStream2 {
    quote! { self.#ident.free(); }
}

fn free_cvec(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    match generic_arg(segment).and_then(segment_name) {
        Some(name) if name == "CSlice" || is_validatable_composite(&name) => quote! {
            self.#ident.free_owning(|entry| entry.free());
        },
        _ => quote! { self.#ident.free(); },
    }
}

fn free_field_path(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    match segment.ident.to_string().as_str() {
        "CSlice" => free_cslice(ident),
        "CVec" => free_cvec(ident, segment),
        name if is_validatable_composite(name) => free_composite(ident),
        _ => quote! {},
    }
}

fn free_field(ident: &Ident, ty: &Type) -> TokenStream2 {
    let Type::Path(type_path) = ty else {
        return quote! {};
    };

    match type_path.path.segments.last() {
        Some(segment) => free_field_path(ident, segment),
        None => quote! {},
    }
}

fn free_impl(name: &Ident, frees: &[TokenStream2]) -> TokenStream2 {
    quote! {
        impl #name {
            pub unsafe fn free(&self) {
                #(#frees)*
            }
        }
    }
}
