// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! `#[derive(CValidate)]` — generates an unsafe `validate()` that checks
//! `struct_size` and every field, driven by `#[optional]`/`#[non_empty]`
//! field attributes.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

use syn::{
    Data, DeriveInput, Error, Expr, Field, Fields, Ident, PathSegment, Result as SynResult, Type, TypePtr,
    parse_macro_input,
};

use crate::common::{field_condition, generic_arg, is_validatable_composite, segment_name};

fn is_attr_present(field: &Field, name: &str) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn cslice_validate(ident: &Ident, optional: bool, non_empty: bool) -> TokenStream2 {
    let empty_check = empty_check(ident, non_empty);

    if optional {
        quote! {
            if !self.#ident.ptr.is_null() {
                unsafe { self.#ident.validate()? };
                #empty_check
            }
        }
    } else {
        quote! {
            unsafe { self.#ident.validate()?; }
            #empty_check
        }
    }
}

fn composite_validate(ident: &Ident) -> TokenStream2 {
    quote! {
        unsafe { self.#ident.validate()?; }
    }
}

fn empty_check(ident: &Ident, non_empty: bool) -> TokenStream2 {
    if non_empty {
        quote! {
            if self.#ident.len == 0 {
                return Err(::upac_abi::error::AbiError::InvalidEntry);
            }
        }
    } else {
        quote! {}
    }
}

fn cvec_element_check(ident: &Ident, seg: &PathSegment) -> TokenStream2 {
    match generic_arg(seg).and_then(segment_name) {
        Some(inner) if inner == "CSlice" || is_validatable_composite(&inner) => quote! {
            for element in unsafe { self.#ident.as_slice() } {
                unsafe { element.validate()? };
            }
        },
        _ => quote! {},
    }
}

fn cvec_validate(ident: &Ident, seg: &PathSegment, non_empty: bool) -> TokenStream2 {
    let empty_check = empty_check(ident, non_empty);
    let element_check = cvec_element_check(ident, seg);

    quote! {
        unsafe { self.#ident.validate()?; }
        #empty_check
        #element_check
    }
}

fn field_path_validate(ident: &Ident, seg: &PathSegment, optional: bool, non_empty: bool) -> TokenStream2 {
    match seg.ident.to_string().as_str() {
        "CSlice" => cslice_validate(ident, optional, non_empty),
        "CVec" => cvec_validate(ident, seg, non_empty),
        name if is_validatable_composite(name) => composite_validate(ident),
        _ => quote! {},
    }
}

fn field_ptr_validate(ident: &Ident, ptr: &TypePtr) -> TokenStream2 {
    let Type::Path(tp) = ptr.elem.as_ref() else {
        return quote! {};
    };
    let Some(seg) = tp.path.segments.last() else {
        return quote! {};
    };

    let name = seg.ident.to_string();

    if is_validatable_composite(&name) {
        quote! {
            unsafe {
                if self.#ident.is_null() {
                    return Err(::upac_abi::error::AbiError::InvalidEntry);
                }
                (*self.#ident).validate()?;
            }
        }
    } else {
        quote! {}
    }
}

fn bitflags_validate(ident: &Ident, field: &Field) -> SynResult<TokenStream2> {
    let Some(attr) = field.attrs.iter().find(|attr| attr.path().is_ident("bitflags")) else {
        return Ok(quote! {});
    };

    let mask = attr.parse_args::<Expr>()?;

    Ok(quote! {
        if self.#ident == 0 || self.#ident & !(#mask) != 0 {
            return Err(::upac_abi::error::AbiError::InvalidEntry);
        }
    })
}

fn field_validate(field: &Field) -> TokenStream2 {
    let Some(ident) = field.ident.as_ref() else {
        return quote! { compile_error!("CValidate only supports named fields") };
    };
    let optional = is_attr_present(field, "optional");
    let non_empty = is_attr_present(field, "non_empty");

    let type_validation = match &field.ty {
        Type::Path(tp) => match tp.path.segments.last() {
            Some(seg) => field_path_validate(ident, seg, optional, non_empty),
            None => quote! {},
        },
        Type::Ptr(ptr) => field_ptr_validate(ident, ptr),
        _ => quote! {},
    };

    let bitflags_validation = match bitflags_validate(ident, field) {
        Ok(tokens) => tokens,
        Err(error) => return error.to_compile_error(),
    };

    let validation = quote! {
        #type_validation
        #bitflags_validation
    };

    match field_condition(field, "skip_if") {
        None => validation,
        Some(Ok(condition)) => {
            let skipped = condition.on(quote! { self });
            quote! {
                if !(#skipped) {
                    #validation
                }
            }
        }
        Some(Err(error)) => error.to_compile_error(),
    }
}

fn validate_impl(name: &Ident, validations: &[TokenStream2]) -> TokenStream2 {
    quote! {
        impl #name {
            pub unsafe fn validate(&self) -> Result<(), ::upac_abi::error::AbiError> {
                crate::types::check_size::<#name>(self.struct_size)?;
                #(#validations)*
                Ok(())
            }
        }

        impl crate::types::CValidatable for #name {
            unsafe fn validate(&self) -> Result<(), ::upac_abi::error::AbiError> {
                unsafe { #name::validate(self) }
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
                return Error::new_spanned(name, "CValidate only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "CValidate only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let validations: Vec<TokenStream2> = fields.iter().map(field_validate).collect();

    validate_impl(name, &validations).into()
}
