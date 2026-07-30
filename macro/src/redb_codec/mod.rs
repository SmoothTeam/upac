//! `#[derive(RedbCodec)]` — generates `encode_into()`/`decode_from()` for
//! storing a struct as a `redb` value via the crate's own byte layout.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

fn field_codec(ident: &syn::Ident, ty: &Type) -> (TokenStream2, TokenStream2) {
    if let Type::Array(array) = ty {
        let len = &array.len;

        let encode = quote! {
            buf.extend_from_slice(&value.#ident);
        };
        let decode = quote! {
            let #ident: #ty = data[*offset..*offset + (#len)].try_into().unwrap();
            *offset += #len;
        };

        return (encode, decode);
    }

    let Type::Path(type_path) = ty else {
        let error = quote! { compile_error!("RedbCodec: unsupported field type"); };
        return (error.clone(), error);
    };

    let Some(segment) = type_path.path.segments.last() else {
        let error = quote! { compile_error!("RedbCodec: unsupported field type"); };
        return (error.clone(), error);
    };

    match segment.ident.to_string().as_str() {
        "String" => (
            quote! { crate::database::codec::write_len_prefixed(buf, value.#ident.as_bytes()); },
            quote! { let #ident = crate::database::codec::read_str(data, offset); },
        ),
        "u32" => (
            quote! { crate::database::codec::write_u32(buf, value.#ident); },
            quote! { let #ident = crate::database::codec::read_u32(data, offset); },
        ),
        "u64" => (
            quote! { crate::database::codec::write_u64(buf, value.#ident); },
            quote! { let #ident = crate::database::codec::read_u64(data, offset); },
        ),
        "bool" => (
            quote! { crate::database::codec::write_bool(buf, value.#ident); },
            quote! { let #ident = crate::database::codec::read_bool(data, offset); },
        ),
        "Option" => (
            quote! { crate::database::codec::write_opt_str(buf, value.#ident.as_deref()); },
            quote! { let #ident = crate::database::codec::read_opt_str(data, offset); },
        ),
        "Vec" => (
            quote! { crate::database::codec::write_vec_u32(buf, &value.#ident); },
            quote! { let #ident = crate::database::codec::read_vec_u32(data, offset); },
        ),
        _ => (
            quote! { #ty::encode_into(buf, &value.#ident); },
            quote! { let #ident = #ty::decode_from(data, offset); },
        ),
    }
}

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(name, "RedbCodec only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "RedbCodec only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut encodes = Vec::new();
    let mut decodes = Vec::new();
    let mut names = Vec::new();

    for filed in fields {
        let Some(ident) = filed.ident.as_ref() else {
            return syn::Error::new_spanned(filed, "RedbCodec only supports named fields")
                .to_compile_error()
                .into();
        };
        let (encode, decode) = field_codec(ident, &filed.ty);

        names.push(ident.clone());
        encodes.push(encode);
        decodes.push(decode);
    }

    let expanded = quote! {
        impl #name {
            pub(crate) fn encode_into(buf: &mut Vec<u8>, value: &#name) {
                #(#encodes)*
            }

            pub(crate) fn decode_from(data: &[u8], offset: &mut usize) -> #name {
                #(#decodes)*

                #name {
                    #(#names),*
                }
            }
        }
    };

    expanded.into()
}
