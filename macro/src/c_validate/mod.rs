//! `#[derive(CValidate)]` — generates an unsafe `validate()` that checks
//! `struct_size` and every field, driven by `#[optional]`/`#[non_empty]`
//! field attributes.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, Type, parse_macro_input};

use crate::common::VALIDATABLE_COMPOSITES;

fn has_attr(field: &Field, name: &str) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn field_validate(field: &Field) -> TokenStream2 {
    let Some(ident) = field.ident.as_ref() else {
        return quote! { compile_error!("CValidate only supports named fields") };
    };
    let optional = has_attr(field, "optional");
    let non_empty = has_attr(field, "non_empty");

    match &field.ty {
        Type::Path(tp) => {
            let Some(seg) = tp.path.segments.last() else {
                return quote! {};
            };

            match seg.ident.to_string().as_str() {
                "CSlice" if optional => quote! {
                    if !self.#ident.ptr.is_null() {
                        unsafe { self.#ident.validate()? };
                    }
                },
                "CSlice" => quote! {
                    unsafe { self.#ident.validate()?; }
                },
                "CVec" if non_empty => quote! {
                    unsafe { self.#ident.validate()? };
                    if self.#ident.len == 0 {
                        return Err(ErrorKind::InvalidEntry);
                    }
                },
                "CVec" => quote! {
                    unsafe { self.#ident.validate()?; }
                },
                name if VALIDATABLE_COMPOSITES.contains(&name) => quote! {
                    unsafe { self.#ident.validate()?; }
                },
                _ => quote! {},
            }
        }
        Type::Ptr(ptr) => {
            let Type::Path(tp) = ptr.elem.as_ref() else {
                return quote! {};
            };
            let Some(seg) = tp.path.segments.last() else {
                return quote! {};
            };

            if VALIDATABLE_COMPOSITES.contains(&seg.ident.to_string().as_str()) {
                quote! {
                    unsafe {
                        if self.#ident.is_null() {
                            return Err(ErrorKind::InvalidEntry);
                        }
                        (*self.#ident).validate()?;
                    }
                }
            } else {
                quote! {}
            }
        }
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
                return syn::Error::new_spanned(name, "CValidate only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "CValidate only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let validations: Vec<TokenStream2> = fields.iter().map(field_validate).collect();

    let expanded = quote! {
        impl #name {
            pub unsafe fn validate(&self) -> Result<(), ErrorKind> {
                check_size::<#name>(self.struct_size)?;
                #(#validations)*
                Ok(())
            }
        }
    };

    expanded.into()
}
