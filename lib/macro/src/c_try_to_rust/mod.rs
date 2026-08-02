//! `#[derive(CTryToRust)]` — generates `impl TryFrom<&CRust> for Rust`,
//! validating the C-ABI struct first and then converting it into an owned
//! Rust domain type (fallible inbound direction).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

use crate::common::{PRIMITIVES, SHARED_TYPES, generic_arg, segment_name};

fn field_from_c_fallible(ident: &syn::Ident, ty: &Type) -> TokenStream2 {
    if let Type::Array(_) = ty {
        return quote! { value.#ident };
    }

    let Type::Path(type_path) = ty else {
        return quote! { compile_error!("CTryToRust: unsupported field type") };
    };

    let Some(segment) = type_path.path.segments.last() else {
        return quote! { compile_error!("CTryToRust: unsupported field type") };
    };

    match segment.ident.to_string().as_str() {
        "String" => quote! {
            {
                let s: &str = (&value.#ident).try_into()?;
                s.to_owned()
            }
        },
        "Option" => {
            let Some(inner_name) = generic_arg(segment).and_then(segment_name) else {
                return quote! { compile_error!("CTryToRust: unsupported Option inner type") };
            };

            if inner_name == "String" {
                quote! { Option::<&str>::try_from(&value.#ident)?.map(str::to_owned) }
            } else {
                quote! { compile_error!("CTryToRust: unsupported Option inner type") }
            }
        }
        "Vec" => {
            let Some(inner_name) = generic_arg(segment).and_then(segment_name) else {
                return quote! { compile_error!("CTryToRust: unsupported Vec element type") };
            };

            if PRIMITIVES.contains(&inner_name.as_str()) {
                quote! {
                    {
                        unsafe { value.#ident.validate()? };
                        unsafe { value.#ident.as_borrowed() }.to_vec()
                    }
                }
            } else {
                quote! { Vec::try_from(&value.#ident)? }
            }
        }
        name if PRIMITIVES.contains(&name) || SHARED_TYPES.contains(&name) => quote! { value.#ident },
        name => {
            let rust_ty = format_ident!("{name}");
            quote! { #rust_ty::try_from(&value.#ident)? }
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
                return syn::Error::new_spanned(name, "CTryToRust only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "CTryToRust only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut field_values = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return syn::Error::new_spanned(field, "CTryToRust only supports named fields")
                .to_compile_error()
                .into();
        };

        let value = field_from_c_fallible(ident, &field.ty);
        field_values.push(quote! { #ident: #value, });
    }

    let expanded = quote! {
        impl TryFrom<&#c_name> for #name {
            type Error = ErrorKind;

            fn try_from(value: &#c_name) -> Result<Self, ErrorKind> {
                unsafe { value.validate()? };

                Ok(#name {
                    #(#field_values)*
                })
            }
        }
    };

    expanded.into()
}
