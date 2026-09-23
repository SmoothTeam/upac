// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! `#[derive(CFree)]` — generates an unsafe `free()` that releases every
//! owned buffer a C-ABI struct holds. This is the reflection-over-fields
//! that Zig got from `inline for (std.meta.fields)`.
//!
//! Dispatch is by field TYPE, decided at compile time:
//!   CSlice           -> self.field.free()
//!   CVec<CSlice>     -> self.field.free_owning(|entry| entry.free())
//!   CVec<composite>  -> self.field.free_owning(|entry| entry.free())
//!   CVec<primitive>  -> self.field.free()
//!   primitive (u32, [u8;32], bool, ...) -> owns nothing, skipped
//!   other named type (composite)        -> self.field.free()
//! Every case dispatches to an inherent free()/free_owning() method on the field's own type
//! (CSlice, CVec<T>, or the composite's own #[derive(CFree)] impl) — nothing here needs a `use`
//! at the derive site, since method-call syntax resolves by receiver type, not by scope.
//! Add a new owned field and it's handled automatically — no list to maintain.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, PathSegment, Type, parse_macro_input};

use crate::common::{generic_arg, is_validatable_composite, segment_name};

fn cslice_free(ident: &Ident) -> TokenStream2 {
    quote! { self.#ident.free(); }
}

fn composite_free(ident: &Ident) -> TokenStream2 {
    quote! { self.#ident.free(); }
}

fn cvec_free(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    match generic_arg(segment).and_then(segment_name) {
        Some(name) if name == "CSlice" || is_validatable_composite(&name) => quote! {
            self.#ident.free_owning(|entry| entry.free());
        },
        _ => quote! { self.#ident.free(); },
    }
}

fn field_path_free(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    match segment.ident.to_string().as_str() {
        "CSlice" => cslice_free(ident),
        "CVec" => cvec_free(ident, segment),
        name if is_validatable_composite(name) => composite_free(ident),
        _ => quote! {},
    }
}

fn field_free(ident: &Ident, ty: &Type) -> TokenStream2 {
    let Type::Path(type_path) = ty else {
        return quote! {};
    };

    match type_path.path.segments.last() {
        Some(segment) => field_path_free(ident, segment),
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

        frees.push(field_free(ident, &field.ty));
    }

    free_impl(name, &frees).into()
}
