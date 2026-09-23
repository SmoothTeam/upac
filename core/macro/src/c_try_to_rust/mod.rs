// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! `#[derive(CTryToRust)]` — generates `impl TryFrom<&CRust> for Rust`,
//! validating the C-ABI struct first and then converting it into a Rust
//! domain type (fallible inbound direction).
//!
//! Two modes, picked automatically from the target struct's own generics:
//!   no lifetime   -> owned mode: `String`/`Vec<String>`/`Option<String>` fields
//!                    are copied out of the C-ABI buffers (`.to_owned()`), a
//!                    `*mut T` field stays a raw pointer (null-checked only
//!                    when the pointee is `CancelToken`, the one type that's
//!                    never optional; anything else, e.g. `*mut c_void`
//!                    hook contexts, passes through unchecked).
//!   `Name<'a>`    -> borrowed mode: `&'a str`/`Vec<&'a str>`/`Option<&'a str>`
//!                    fields borrow directly from the C-ABI buffers (no
//!                    allocation at all), and a `&'a T` field null-checks
//!                    then dereferences the matching `*mut T` on the C side.
//! Which mode applies is entirely driven by how each field is written on
//! the Rust struct — nothing needs to be passed in by the caller.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Error, Fields, Ident, Lifetime, PathSegment, Type, TypePtr, TypeReference, parse_macro_input,
};

use crate::common::{PRIMITIVES, SHARED_TYPES, generic_arg, is_str_type, segment_name};

fn is_str_ref(ty: &Type) -> bool {
    matches!(ty, Type::Reference(reference) if is_str_type(&reference.elem))
}

fn string_from_c(ident: &Ident) -> TokenStream2 {
    quote! {
        {
            let s: &str = (&value.#ident).try_into()?;
            s.to_owned()
        }
    }
}

fn option_from_c(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    let Some(inner) = generic_arg(segment) else {
        return quote! { compile_error!("CTryToRust: unsupported Option inner type") };
    };

    if is_str_ref(inner) {
        return quote! { Option::<&str>::try_from(&value.#ident)? };
    }

    match segment_name(inner).as_deref() {
        Some("String") => quote! { Option::<&str>::try_from(&value.#ident)?.map(str::to_owned) },
        Some("HookMessageFn") => quote! { value.#ident },
        _ => quote! { compile_error!("CTryToRust: unsupported Option inner type") },
    }
}

fn vec_from_c(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    let Some(inner) = generic_arg(segment) else {
        return quote! { compile_error!("CTryToRust: unsupported Vec element type") };
    };

    if is_str_ref(inner) {
        return quote! {
            {
                unsafe { value.#ident.validate()? };
                unsafe { value.#ident.as_slice() }
                    .iter()
                    .map(<&str>::try_from)
                    .collect::<Result<Vec<_>, ErrorKind>>()?
            }
        };
    }

    match segment_name(inner).as_deref() {
        Some("String") => quote! {
            {
                unsafe { value.#ident.validate()? };
                unsafe { value.#ident.as_slice() }
                    .iter()
                    .map(<&str>::try_from)
                    .map(|element| element.map(str::to_owned))
                    .collect::<Result<Vec<_>, ErrorKind>>()?
            }
        },
        Some(name) if PRIMITIVES.contains(&name) => quote! {
            {
                unsafe { value.#ident.validate()? };
                unsafe { value.#ident.as_borrowed() }.to_vec()
            }
        },
        _ => quote! { Vec::try_from(&value.#ident)? },
    }
}

fn primitive_from_c(ident: &Ident) -> TokenStream2 {
    quote! { value.#ident }
}

fn composite_from_c(ident: &Ident, name: &str) -> TokenStream2 {
    let rust_ty = format_ident!("{name}");
    quote! { #rust_ty::try_from(&value.#ident)? }
}

fn field_path_from_c(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    match segment.ident.to_string().as_str() {
        "String" => string_from_c(ident),
        "Option" => option_from_c(ident, segment),
        "Vec" => vec_from_c(ident, segment),
        name if PRIMITIVES.contains(&name) || SHARED_TYPES.contains(&name) => primitive_from_c(ident),
        name => composite_from_c(ident, name),
    }
}

fn pointee_name(ptr: &TypePtr) -> Option<String> {
    match ptr.elem.as_ref() {
        Type::Path(type_path) => type_path.path.segments.last().map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

fn ptr_from_c(ident: &Ident, ptr: &TypePtr) -> TokenStream2 {
    if pointee_name(ptr).as_deref() == Some("CancelToken") {
        return quote! {
            {
                if value.#ident.is_null() {
                    return Err(ErrorKind::InvalidEntry);
                }
                value.#ident
            }
        };
    }

    quote! { value.#ident }
}

fn reference_from_c(ident: &Ident, reference: &TypeReference) -> TokenStream2 {
    if is_str_type(&reference.elem) {
        return quote! { (&value.#ident).try_into()? };
    }

    quote! {
        {
            if value.#ident.is_null() {
                return Err(ErrorKind::InvalidEntry);
            }
            unsafe { &*value.#ident }
        }
    }
}

fn field_from_c_fallible(ident: &Ident, ty: &Type) -> TokenStream2 {
    if let Type::Array(_) = ty {
        return quote! { value.#ident };
    }

    if let Type::Ptr(ptr) = ty {
        return ptr_from_c(ident, ptr);
    }

    if let Type::Reference(reference) = ty {
        return reference_from_c(ident, reference);
    }

    let Type::Path(type_path) = ty else {
        return quote! { compile_error!("CTryToRust: unsupported field type") };
    };

    match type_path.path.segments.last() {
        Some(segment) => field_path_from_c(ident, segment),
        None => quote! { compile_error!("CTryToRust: unsupported field type") },
    }
}

fn try_from_impl(
    name: &Ident, c_name: &Ident, lifetime: Option<&Lifetime>, field_values: &[TokenStream2],
) -> TokenStream2 {
    match lifetime {
        Some(lifetime) => quote! {
            impl<#lifetime> TryFrom<&#lifetime #c_name> for #name<#lifetime> {
                type Error = ErrorKind;

                fn try_from(value: &#lifetime #c_name) -> Result<Self, ErrorKind> {
                    unsafe { value.validate()? };

                    Ok(#name {
                        #(#field_values)*
                    })
                }
            }
        },
        None => quote! {
            impl TryFrom<&#c_name> for #name {
                type Error = ErrorKind;

                fn try_from(value: &#c_name) -> Result<Self, ErrorKind> {
                    unsafe { value.validate()? };

                    Ok(#name {
                        #(#field_values)*
                    })
                }
            }
        },
    }
}

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let c_name = format_ident!("C{name}");
    let lifetime = input.generics.lifetimes().next().map(|param| param.lifetime.clone());

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Error::new_spanned(name, "CTryToRust only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "CTryToRust only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut field_values = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return Error::new_spanned(field, "CTryToRust only supports named fields")
                .to_compile_error()
                .into();
        };

        let value = field_from_c_fallible(ident, &field.ty);
        field_values.push(quote! { #ident: #value, });
    }

    try_from_impl(name, &c_name, lifetime.as_ref(), &field_values).into()
}
