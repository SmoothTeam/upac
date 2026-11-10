// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

use syn::{
    Data, DeriveInput, Error, Expr, Field, Fields, Ident, PathSegment, Result as SynResult, Type, TypePtr,
    parse_macro_input,
};

use crate::common::{field_condition, generic_arg, is_out_field, is_validatable_composite, segment_name};

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

    let validations: Vec<TokenStream2> = fields.iter().map(validate_field).collect();

    validate_impl(name, &validations).into()
}

fn is_attr_present(field: &Field, name: &str) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn validate_cslice(ident: &Ident, optional: bool, non_empty: bool) -> TokenStream2 {
    let empty_guard = check_empty(ident, non_empty);

    if optional {
        quote! {
            if !self.#ident.ptr.is_null() {
                unsafe { self.#ident.validate()? };
                #empty_guard
            }
        }
    } else {
        quote! {
            unsafe { self.#ident.validate()?; }
            #empty_guard
        }
    }
}

fn validate_composite(ident: &Ident) -> TokenStream2 {
    quote! {
        unsafe { self.#ident.validate()?; }
    }
}

fn check_empty(ident: &Ident, non_empty: bool) -> TokenStream2 {
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

fn check_cvec_elements(ident: &Ident, segment: &PathSegment) -> TokenStream2 {
    match generic_arg(segment).and_then(segment_name) {
        Some(inner) if inner == "CSlice" || is_validatable_composite(&inner) => quote! {
            for element in unsafe { self.#ident.as_slice() } {
                unsafe { element.validate()? };
            }
        },
        _ => quote! {},
    }
}

fn validate_cvec(ident: &Ident, segment: &PathSegment, non_empty: bool) -> TokenStream2 {
    let empty_guard = check_empty(ident, non_empty);
    let element_guard = check_cvec_elements(ident, segment);

    quote! {
        unsafe { self.#ident.validate()?; }
        #empty_guard
        #element_guard
    }
}

fn validate_field_path(ident: &Ident, segment: &PathSegment, optional: bool, non_empty: bool) -> TokenStream2 {
    match segment.ident.to_string().as_str() {
        "CSlice" => validate_cslice(ident, optional, non_empty),
        "CVec" => validate_cvec(ident, segment, non_empty),
        name if is_validatable_composite(name) => validate_composite(ident),
        _ => quote! {},
    }
}

fn validate_field_ptr(ident: &Ident, ptr: &TypePtr) -> TokenStream2 {
    let Type::Path(type_path) = ptr.elem.as_ref() else {
        return quote! {};
    };
    let Some(segment) = type_path.path.segments.last() else {
        return quote! {};
    };

    let name = segment.ident.to_string();

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

fn validate_bitflags(ident: &Ident, field: &Field) -> SynResult<TokenStream2> {
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

fn validate_field(field: &Field) -> TokenStream2 {
    let Some(ident) = field.ident.as_ref() else {
        return quote! { compile_error!("CValidate only supports named fields") };
    };

    if is_out_field(field) {
        return quote! {};
    }

    let optional = is_attr_present(field, "optional");
    let non_empty = is_attr_present(field, "non_empty");

    let type_validation = match &field.ty {
        Type::Path(type_path) => match type_path.path.segments.last() {
            Some(segment) => validate_field_path(ident, segment, optional, non_empty),
            None => quote! {},
        },
        Type::Ptr(ptr) => validate_field_ptr(ident, ptr),
        _ => quote! {},
    };

    let bitflags_validation = match validate_bitflags(ident, field) {
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
