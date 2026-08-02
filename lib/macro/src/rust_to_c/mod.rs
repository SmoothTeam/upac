//! `#[derive(RustToC)]` — generates `impl From<Rust> for CRust`, converting
//! an owned Rust domain type into its C-ABI mirror (outbound direction).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

use crate::common::{PRIMITIVES, SHARED_TYPES, generic_arg, segment_name};

fn field_to_c(ident: &syn::Ident, ty: &Type) -> TokenStream2 {
    if let Type::Array(_) = ty {
        return quote! { value.#ident };
    }

    let Type::Path(type_path) = ty else {
        return quote! { compile_error!("RustToC: unsupported field type") };
    };

    let Some(segment) = type_path.path.segments.last() else {
        return quote! { compile_error!("RustToC: unsupported field type") };
    };

    match segment.ident.to_string().as_str() {
        "String" => quote! { CSlice::from_owned(value.#ident.into_bytes()) },
        "Option" => quote! { value.#ident.into() },
        "Vec" => {
            let Some(inner_name) = generic_arg(segment).and_then(segment_name) else {
                return quote! { compile_error!("RustToC: unsupported Vec element type") };
            };

            if PRIMITIVES.contains(&inner_name.as_str()) {
                quote! { CVec::from_owned(value.#ident) }
            } else {
                let c_inner = format_ident!("C{inner_name}");
                quote! { CVec::from_owned(value.#ident.into_iter().map(#c_inner::from).collect()) }
            }
        }
        name if PRIMITIVES.contains(&name) || SHARED_TYPES.contains(&name) => quote! { value.#ident },
        name => {
            let c_ty = format_ident!("C{name}");
            quote! { #c_ty::from(value.#ident) }
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
                return syn::Error::new_spanned(name, "RustToC only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "RustToC only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut field_values = Vec::new();

    for field in fields {
        let Some(ident) = field.ident.as_ref() else {
            return syn::Error::new_spanned(field, "RustToC only supports named fields")
                .to_compile_error()
                .into();
        };

        let value = field_to_c(ident, &field.ty);
        field_values.push(quote! { #ident: #value, });
    }

    let expanded = quote! {
        impl From<#name> for #c_name {
            fn from(value: #name) -> Self {
                #c_name {
                    struct_size: size_of::<#c_name>(),
                    #(#field_values)*
                }
            }
        }
    };

    expanded.into()
}
