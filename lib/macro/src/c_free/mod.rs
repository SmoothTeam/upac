// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

//! `#[derive(CFree)]` — generates an unsafe `free()` that releases every
//! owned buffer a C-ABI struct holds. This is the reflection-over-fields
//! that Zig got from `inline for (std.meta.fields)`.
//!
//! Dispatch is by field TYPE, decided at compile time:
//!   CSlice           -> free_cslice(&self.field)
//!   CVec<primitive>  -> free_cvec(&self.field)
//!   CVec<composite>  -> free_cvec_owning(&self.field, |entry| entry.free())
//!   primitive (u32, [u8;32], bool, ...) -> owns nothing, skipped
//!   other named type (composite)        -> self.field.free()
//! Add a new owned field and it's handled automatically — no list to maintain.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

use crate::common::{VALIDATABLE_COMPOSITES, generic_arg, segment_name};

fn field_free(ident: &syn::Ident, ty: &Type) -> TokenStream2 {
    let Type::Path(type_path) = ty else {
        return quote! {};
    };

    let Some(segment) = type_path.path.segments.last() else {
        return quote! {};
    };

    match segment.ident.to_string().as_str() {
        "CSlice" => quote! { free_cslice(&self.#ident); },
        "CVec" => {
            let inner_name = generic_arg(segment).and_then(segment_name);
            match inner_name.as_deref() {
                Some(name) if VALIDATABLE_COMPOSITES.contains(&name) => quote! {
                    free_cvec_owning(&self.#ident, |entry| entry.free());
                },
                _ => quote! { free_cvec(&self.#ident); },
            }
        }
        name if VALIDATABLE_COMPOSITES.contains(&name) => quote! { self.#ident.free(); },
        _ => quote! {},
    }
}

pub(crate) fn expand(input: TokenStream) -> TokenStream {
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
        let Some(ident) = field.ident.as_ref() else {
            return syn::Error::new_spanned(field, "CFree only supports named fields")
                .to_compile_error()
                .into();
        };

        frees.push(field_free(ident, &field.ty));
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
