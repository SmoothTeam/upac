// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Fields, Ident, Lifetime, PathSegment, Type, parse_macro_input};

use crate::c_try_to_rust::field_from_c_fallible;
use crate::common::{
    ABI_ENUMS, PASSTHROUGH_TYPES, PRIMITIVES, generic_arg, is_out_field, is_str_ref, is_str_type, segment_name,
};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let c_name = format_ident!("C{name}");
    let lifetime = input.generics.lifetimes().next().map(|param| param.lifetime.clone());

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Error::new_spanned(name, "RustToC only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "RustToC only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut field_values = Vec::new();
    let mut out_fields = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return Error::new_spanned(field, "RustToC only supports named fields")
                .to_compile_error()
                .into();
        };

        if is_out_field(field) {
            field_values.push(quote! { #ident: Default::default(), });
            out_fields.push((ident, &field.ty));
            continue;
        }

        let value = field_to_c(&quote! { value.#ident }, &field.ty);
        field_values.push(quote! { #ident: #value, });
    }

    let from_impl = to_c_impl(name, &c_name, lifetime.as_ref(), &field_values);
    let output_impl = output_impl(&c_name, lifetime.as_ref(), &out_fields);

    quote! {
        #from_impl
        #output_impl
    }
    .into()
}

fn string_to_c(source: &TokenStream2) -> TokenStream2 {
    quote! { CSlice::from_owned(#source.into_bytes()) }
}

fn string_ref_to_c(source: &TokenStream2) -> TokenStream2 {
    quote! { CSlice::from_owned(#source.as_bytes().to_vec()) }
}

fn option_to_c(source: &TokenStream2) -> TokenStream2 {
    quote! { #source.into() }
}

fn primitive_to_c(source: &TokenStream2) -> TokenStream2 {
    quote! { #source }
}

fn abi_enum_to_c(source: &TokenStream2) -> TokenStream2 {
    quote! { #source.into() }
}

fn composite_to_c(source: &TokenStream2, name: &str) -> TokenStream2 {
    let c_ty = format_ident!("C{name}");
    quote! { #c_ty::from(#source) }
}

fn vec_to_c(source: &TokenStream2, segment: &PathSegment) -> TokenStream2 {
    let Some(inner) = generic_arg(segment) else {
        return quote! { compile_error!("RustToC: unsupported Vec element type") };
    };

    if is_str_ref(inner) {
        return quote! {
            CVec::from_owned(#source.into_iter().map(|element| CSlice::from_owned(element.as_bytes().to_vec())).collect())
        };
    }

    let Some(inner_name) = segment_name(inner) else {
        return quote! { compile_error!("RustToC: unsupported Vec element type") };
    };

    if inner_name == "String" {
        quote! { CVec::from_owned(#source.into_iter().map(|element| CSlice::from_owned(element.into_bytes())).collect()) }
    } else if PRIMITIVES.contains(&inner_name.as_str()) {
        quote! { CVec::from_owned(#source) }
    } else {
        let c_inner = format_ident!("C{inner_name}");
        quote! { CVec::from_owned(#source.into_iter().map(#c_inner::from).collect()) }
    }
}

fn field_path_to_c(source: &TokenStream2, segment: &PathSegment) -> TokenStream2 {
    match segment.ident.to_string().as_str() {
        "String" => string_to_c(source),
        "Option" => option_to_c(source),
        "Vec" => vec_to_c(source, segment),
        name if PRIMITIVES.contains(&name) || PASSTHROUGH_TYPES.contains(&name) => primitive_to_c(source),
        name if ABI_ENUMS.contains(&name) => abi_enum_to_c(source),
        name => composite_to_c(source, name),
    }
}

fn field_to_c(source: &TokenStream2, ty: &Type) -> TokenStream2 {
    if let Type::Array(_) = ty {
        return quote! { #source };
    }

    if let Type::Ptr(_) = ty {
        return quote! { #source };
    }

    if let Type::Reference(reference) = ty {
        return if is_str_type(&reference.elem) {
            string_ref_to_c(source)
        } else {
            quote! { #source.as_ptr() }
        };
    }

    let Type::Path(type_path) = ty else {
        return quote! { compile_error!("RustToC: unsupported field type") };
    };

    match type_path.path.segments.last() {
        Some(segment) => field_path_to_c(source, segment),
        None => quote! { compile_error!("RustToC: unsupported field type") },
    }
}

fn to_c_impl(name: &Ident, c_name: &Ident, lifetime: Option<&Lifetime>, field_values: &[TokenStream2]) -> TokenStream2 {
    match lifetime {
        Some(lifetime) => quote! {
            impl<#lifetime> From<#name<#lifetime>> for #c_name {
                #[allow(unused_variables)]
                fn from(value: #name<#lifetime>) -> Self {
                    #c_name {
                        struct_size: ::std::mem::size_of::<#c_name>(),
                        #(#field_values)*
                    }
                }
            }
        },
        None => quote! {
            impl From<#name> for #c_name {
                #[allow(unused_variables)]
                fn from(value: #name) -> Self {
                    #c_name {
                        struct_size: ::std::mem::size_of::<#c_name>(),
                        #(#field_values)*
                    }
                }
            }
        },
    }
}

fn output_impl(c_name: &Ident, lifetime: Option<&Lifetime>, out_fields: &[(&Ident, &Type)]) -> TokenStream2 {
    if out_fields.is_empty() {
        return quote! {};
    }

    let lifetime = lifetime
        .cloned()
        .unwrap_or_else(|| Lifetime::new("'output", Span::call_site()));

    let idents: Vec<&Ident> = out_fields.iter().map(|(ident, _)| *ident).collect();
    let types: Vec<&Type> = out_fields.iter().map(|(_, ty)| *ty).collect();

    let read_values: Vec<TokenStream2> = out_fields
        .iter()
        .map(|(ident, ty)| field_from_c_fallible(ident, ty))
        .collect();

    let write_values: Vec<TokenStream2> = out_fields
        .iter()
        .map(|(ident, ty)| field_to_c(&quote! { #ident }, ty))
        .collect();

    let (value_type, value_pattern, read_value) = match (types.as_slice(), idents.as_slice(), read_values.as_slice()) {
        ([single_type], [single_ident], [single_read]) => (
            quote! { #single_type },
            quote! { #single_ident },
            quote! { #single_read },
        ),
        _ => (
            quote! { (#(#types),*) },
            quote! { (#(#idents),*) },
            quote! { (#(#read_values),*) },
        ),
    };

    quote! {
        impl<#lifetime> crate::request::Output<#lifetime> for #c_name {
            type Value = #value_type;

            #[allow(clippy::needless_question_mark)]
            fn read_output(&#lifetime self) -> Result<Self::Value, crate::error::ErrorKind> {
                let value = self;

                Ok(#read_value)
            }

            fn write_output(&mut self, value: Self::Value) {
                let #value_pattern = value;

                #(self.#idents = #write_values;)*
            }
        }
    }
}
