// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

//! `#[derive(JsonCodec)]` — generates `to_json()`/`from_json()` for
//! storing a struct as a `serde_json::Value` (used for on-disk records
//! outside the redb package DB, e.g. deploy `meta.json`).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, PathSegment, Type, parse_macro_input};

use crate::common::{generic_arg, segment_name};

fn string_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! { object.insert(stringify!(#ident).to_string(), serde_json::Value::String(self.#ident.clone())); },
        quote! {
            let #ident = object
                .get(stringify!(#ident))
                .and_then(serde_json::Value::as_str)
                .ok_or(crate::database::error::DeployRecordError::InvalidField)?
                .to_string();
        },
    )
}

fn u64_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! { object.insert(stringify!(#ident).to_string(), serde_json::Value::from(self.#ident)); },
        quote! {
            let #ident = object
                .get(stringify!(#ident))
                .and_then(serde_json::Value::as_u64)
                .ok_or(crate::database::error::DeployRecordError::InvalidField)?;
        },
    )
}

fn option_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! {
            object.insert(
                stringify!(#ident).to_string(),
                match &self.#ident {
                    Some(inner) => serde_json::Value::String(inner.clone()),
                    None => serde_json::Value::Null,
                },
            );
        },
        quote! {
            let #ident = object.get(stringify!(#ident)).and_then(serde_json::Value::as_str).map(str::to_string);
        },
    )
}

fn composite_codec(ident: &Ident, ty: &Type) -> (TokenStream2, TokenStream2) {
    (
        quote! { object.insert(stringify!(#ident).to_string(), self.#ident.to_json()); },
        quote! {
            let #ident = #ty::from_json(
                object.get(stringify!(#ident)).ok_or(crate::database::error::DeployRecordError::InvalidField)?,
            )?;
        },
    )
}

fn vec_string_codec(ident: &Ident) -> (TokenStream2, TokenStream2) {
    (
        quote! {
            object.insert(
                stringify!(#ident).to_string(),
                serde_json::Value::Array(self.#ident.iter().cloned().map(serde_json::Value::String).collect()),
            );
        },
        quote! {
            let #ident = object
                .get(stringify!(#ident))
                .and_then(serde_json::Value::as_array)
                .ok_or(crate::database::error::DeployRecordError::InvalidField)?
                .iter()
                .map(|element| {
                    element
                        .as_str()
                        .map(str::to_string)
                        .ok_or(crate::database::error::DeployRecordError::InvalidField)
                })
                .collect::<Result<Vec<_>, _>>()?;
        },
    )
}

fn vec_composite_codec(ident: &Ident, element: &Type) -> (TokenStream2, TokenStream2) {
    (
        quote! {
            object.insert(
                stringify!(#ident).to_string(),
                serde_json::Value::Array(self.#ident.iter().map(#element::to_json).collect()),
            );
        },
        quote! {
            let #ident = object
                .get(stringify!(#ident))
                .and_then(serde_json::Value::as_array)
                .ok_or(crate::database::error::DeployRecordError::InvalidField)?
                .iter()
                .map(#element::from_json)
                .collect::<Result<Vec<_>, _>>()?;
        },
    )
}

fn vec_codec(ident: &Ident, segment: &PathSegment) -> (TokenStream2, TokenStream2) {
    let Some(element) = generic_arg(segment) else {
        let error = quote! { compile_error!("JsonCodec: Vec must have a type argument"); };
        return (error.clone(), error);
    };

    match segment_name(element).as_deref() {
        Some("String") => vec_string_codec(ident),
        _ => vec_composite_codec(ident, element),
    }
}

fn field_path_codec(ident: &Ident, segment: &PathSegment, ty: &Type) -> (TokenStream2, TokenStream2) {
    match segment.ident.to_string().as_str() {
        "String" => string_codec(ident),
        "u64" => u64_codec(ident),
        "Option" => option_codec(ident),
        "Vec" => vec_codec(ident, segment),
        _ => composite_codec(ident, ty),
    }
}

fn field_codec(ident: &Ident, ty: &Type) -> (TokenStream2, TokenStream2) {
    let Type::Path(type_path) = ty else {
        let error = quote! { compile_error!("JsonCodec: unsupported field type"); };
        return (error.clone(), error);
    };

    let Some(segment) = type_path.path.segments.last() else {
        let error = quote! { compile_error!("JsonCodec: unsupported field type"); };
        return (error.clone(), error);
    };

    field_path_codec(ident, segment, ty)
}

fn codec_impl(name: &Ident, encodes: &[TokenStream2], decodes: &[TokenStream2], names: &[Ident]) -> TokenStream2 {
    quote! {
        impl #name {
            pub fn to_json(&self) -> serde_json::Value {
                let mut object = serde_json::Map::new();
                #(#encodes)*
                serde_json::Value::Object(object)
            }

            pub fn from_json(value: &serde_json::Value) -> Result<#name, crate::database::error::DeployRecordError> {
                let object = value.as_object().ok_or(crate::database::error::DeployRecordError::InvalidField)?;
                #(#decodes)*

                Ok(#name {
                    #(#names),*
                })
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
                return Error::new_spanned(name, "JsonCodec only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "JsonCodec only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut encodes = Vec::new();
    let mut decodes = Vec::new();
    let mut names = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return Error::new_spanned(field, "JsonCodec only supports named fields")
                .to_compile_error()
                .into();
        };
        let (encode, decode) = field_codec(ident, &field.ty);

        names.push(ident.clone());
        encodes.push(encode);
        decodes.push(decode);
    }

    codec_impl(name, &encodes, &decodes, &names).into()
}
