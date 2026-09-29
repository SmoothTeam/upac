// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! `#[stage]` — on an `impl Stage<E> for …` block, reads the `run` body and generates
//! `requires()` from every `ctx_get!`/`ctx_take!` and `provides()` from every `context.put(…)`,
//! so the orchestrator can validate a pipeline's context slots before running it.

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::parse::ParseStream;
use syn::visit::{Visit, visit_expr_method_call, visit_local, visit_macro};
use syn::{
    Error, Expr, ExprMethodCall, GenericArgument, Ident, ImplItem, ItemImpl, Local, Macro, Pat, Path,
    Result as SynResult, Token, Type, parse_macro_input, parse_quote,
};

const CONTEXT_RECEIVER: &str = "context";
const READ_MACROS: &[&str] = &["ctx_get", "ctx_take"];

#[derive(Default)]
struct ContextAccess {
    requires: Vec<Type>,
    provides: Vec<Type>,
    bindings: HashMap<Ident, Type>,
    errors: Vec<Error>,
}

impl ContextAccess {
    fn read_macro_type(mac: &Macro) -> Option<SynResult<Type>> {
        let name = mac.path.segments.last()?.ident.to_string();
        if !READ_MACROS.contains(&name.as_str()) {
            return None;
        }

        Some(mac.parse_body_with(|input: ParseStream| {
            input.parse::<Expr>()?;
            input.parse::<Token![,]>()?;
            input.parse::<Type>()
        }))
    }

    fn put_type(&self, call: &ExprMethodCall) -> SynResult<Type> {
        if let Some(turbofish) = &call.turbofish {
            if let Some(GenericArgument::Type(ty)) = turbofish.args.first() {
                return Ok(ty.clone());
            }
        }

        let Some(argument) = call.args.first() else {
            return Err(Error::new_spanned(call, "#[stage]: context.put(…) without an argument"));
        };

        let slot_type = match argument {
            Expr::Struct(expr_struct) => Some(Self::path_type(&expr_struct.path)),
            Expr::Call(expr_call) => match expr_call.func.as_ref() {
                Expr::Path(expr_path) if Self::names_a_type(&expr_path.path) => Some(Self::path_type(&expr_path.path)),
                _ => None,
            },
            Expr::Path(expr_path) => expr_path
                .path
                .get_ident()
                .and_then(|ident| self.bindings.get(ident))
                .cloned(),
            _ => None,
        };

        slot_type.ok_or_else(|| {
            Error::new_spanned(
                argument,
                "#[stage]: can't tell the slot type of this context.put(…); write context.put::<Type>(…)",
            )
        })
    }

    fn path_type(path: &Path) -> Type {
        parse_quote!(#path)
    }

    fn names_a_type(path: &Path) -> bool {
        path.segments.last().is_some_and(|segment| {
            segment
                .ident
                .to_string()
                .starts_with(|first: char| first.is_ascii_uppercase())
        })
    }
}

impl<'ast> Visit<'ast> for ContextAccess {
    fn visit_local(&mut self, local: &'ast Local) {
        if let (Pat::Ident(pat_ident), Some(init)) = (&local.pat, &local.init) {
            if let Expr::Macro(expr_macro) = init.expr.as_ref() {
                if let Some(Ok(ty)) = Self::read_macro_type(&expr_macro.mac) {
                    self.bindings.insert(pat_ident.ident.clone(), ty);
                }
            }
        }

        visit_local(self, local);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        match Self::read_macro_type(mac) {
            Some(Ok(ty)) => self.requires.push(ty),
            Some(Err(error)) => self.errors.push(error),
            None => {}
        }

        visit_macro(self, mac);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let is_context_put = call.method == "put"
            && matches!(call.receiver.as_ref(), Expr::Path(receiver) if receiver.path.is_ident(CONTEXT_RECEIVER));

        if is_context_put {
            match self.put_type(call) {
                Ok(ty) => self.provides.push(ty),
                Err(error) => self.errors.push(error),
            }
        }

        visit_expr_method_call(self, call);
    }
}

fn deduplicated(types: Vec<Type>) -> Vec<Type> {
    let mut seen = Vec::new();
    let mut unique = Vec::new();

    for ty in types {
        let key = ty.to_token_stream().to_string();
        if !seen.contains(&key) {
            seen.push(key);
            unique.push(ty);
        }
    }

    unique
}

fn type_ids_fn(name: &str, types: &[Type]) -> TokenStream2 {
    let name = Ident::new(name, proc_macro2::Span::call_site());

    quote! {
        fn #name(&self) -> ::std::vec::Vec<::std::any::TypeId> {
            ::std::vec![#(::std::any::TypeId::of::<#types>()),*]
        }
    }
}

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let mut item_impl = parse_macro_input!(input as ItemImpl);

    let run = item_impl.items.iter().find_map(|item| match item {
        ImplItem::Fn(function) if function.sig.ident == "run" => Some(function),
        _ => None,
    });

    let Some(run) = run else {
        return Error::new_spanned(&item_impl.self_ty, "#[stage] needs an impl block with a `run` method")
            .to_compile_error()
            .into();
    };

    let mut access = ContextAccess::default();
    access.visit_block(&run.block);

    if let Some(first) = access.errors.into_iter().reduce(|mut combined, error| {
        combined.combine(error);
        combined
    }) {
        return first.to_compile_error().into();
    }

    let requires = type_ids_fn("requires", &deduplicated(access.requires));
    let provides = type_ids_fn("provides", &deduplicated(access.provides));

    item_impl.items.push(parse_quote!(#requires));
    item_impl.items.push(parse_quote!(#provides));

    item_impl.into_token_stream().into()
}
