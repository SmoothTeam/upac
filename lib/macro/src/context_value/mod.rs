use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let field = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => {
                return Error::new_spanned(name, "ContextValue only supports single-field tuple structs")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(name, "ContextValue only supports tuple structs")
                .to_compile_error()
                .into();
        }
    };

    let ty = &field.ty;

    let expanded: TokenStream2 = quote! {
        impl std::ops::Deref for #name {
            type Target = #ty;

            fn deref(&self) -> &#ty {
                &self.0
            }
        }

        impl std::ops::DerefMut for #name {
            fn deref_mut(&mut self) -> &mut #ty {
                &mut self.0
            }
        }

        impl From<#ty> for #name {
            fn from(value: #ty) -> Self {
                #name(value)
            }
        }
    };

    expanded.into()
}
