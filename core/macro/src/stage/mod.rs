// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! `#[stage]` — on an `impl Stage<E> for …` block, reads the `run` body and generates
//! `requires()` from every `context.get::<T>()`/`context.take::<T>()` and `provides()` from every
//! `context.put(…)`, `context.replace(…)` and `context.push(…)` (a pushed `T` provides `Vec<T>`), so the orchestrator can
//! validate a pipeline's context slots before running it. `context` may only be used through those
//! calls inside `run`, and one slot type may only be put once; overwriting on purpose is `replace`.

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, quote};
use syn::visit::{Visit, visit_expr, visit_expr_path, visit_local, visit_macro};
use syn::{
    Error, Expr, ExprMethodCall, ExprPath, GenericArgument, Ident, ImplItem, ItemImpl, Local, Macro, Pat, Path,
    Result as SynResult, Type, parse_macro_input, parse_quote,
};

const CONTEXT_RECEIVER: &str = "context";

enum ContextCall {
    Read,
    Put,
    Replace,
    Push,
}

#[derive(Default)]
struct ContextAccess {
    requires: Vec<Type>,
    provides: Vec<Type>,
    bindings: HashMap<Ident, Type>,
    errors: Vec<Error>,
}

impl ContextAccess {
    fn context_call(call: &ExprMethodCall) -> Option<ContextCall> {
        let is_context =
            matches!(call.receiver.as_ref(), Expr::Path(receiver) if receiver.path.is_ident(CONTEXT_RECEIVER));
        if !is_context {
            return None;
        }

        match call.method.to_string().as_str() {
            "get" | "take" => Some(ContextCall::Read),
            "put" => Some(ContextCall::Put),
            "replace" => Some(ContextCall::Replace),
            "push" => Some(ContextCall::Push),
            _ => None,
        }
    }

    fn turbofish_type(call: &ExprMethodCall) -> Option<Type> {
        match call.turbofish.as_ref()?.args.first()? {
            GenericArgument::Type(ty) => Some(ty.clone()),
            _ => None,
        }
    }

    fn read_type(call: &ExprMethodCall) -> SynResult<Type> {
        Self::turbofish_type(call).ok_or_else(|| {
            Error::new_spanned(
                call,
                format!(
                    "#[stage]: write context.{}::<Type>() so the slot type is visible",
                    call.method
                ),
            )
        })
    }

    fn written_type(&self, call: &ExprMethodCall) -> SynResult<Type> {
        if let Some(ty) = Self::turbofish_type(call) {
            return Ok(ty);
        }

        let Some(argument) = call.args.first() else {
            return Err(Error::new_spanned(
                call,
                format!("#[stage]: context.{}(…) without an argument", call.method),
            ));
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
                .cloned()
                .or_else(|| Self::names_a_type(&expr_path.path).then(|| Self::path_type(&expr_path.path))),
            _ => None,
        };

        slot_type.ok_or_else(|| {
            Error::new_spanned(
                argument,
                format!(
                    "#[stage]: can't tell the slot type of this context.{0}(…); write context.{0}::<Type>(…)",
                    call.method
                ),
            )
        })
    }

    fn bound_read_type(init: &Expr) -> Option<Type> {
        let call = match init {
            Expr::Try(expr_try) => expr_try.expr.as_ref(),
            other => other,
        };

        let Expr::MethodCall(call) = call else {
            return None;
        };

        match Self::context_call(call) {
            Some(ContextCall::Read) => Self::turbofish_type(call),
            _ => None,
        }
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

    fn mentions_context(tokens: TokenStream2) -> bool {
        tokens.into_iter().any(|token| match token {
            TokenTree::Ident(ident) => ident == CONTEXT_RECEIVER,
            TokenTree::Group(group) => Self::mentions_context(group.stream()),
            _ => false,
        })
    }

    fn record(&mut self, call: &ExprMethodCall, kind: ContextCall) {
        let result = match kind {
            ContextCall::Read => Self::read_type(call).map(|ty| self.requires.push(ty)),
            ContextCall::Put => self.written_type(call).and_then(|ty| self.provide_once(call, ty)),
            ContextCall::Replace => self.written_type(call).map(|ty| self.provides.push(ty)),
            ContextCall::Push => self
                .written_type(call)
                .map(|ty| self.provides.push(parse_quote!(Vec<#ty>))),
        };

        if let Err(error) = result {
            self.errors.push(error);
        }
    }

    fn provide_once(&mut self, call: &ExprMethodCall, ty: Type) -> SynResult<()> {
        let key = ty.to_token_stream().to_string();

        if self
            .provides
            .iter()
            .any(|provided| provided.to_token_stream().to_string() == key)
        {
            return Err(Error::new_spanned(
                call,
                format!("#[stage]: `{key}` is already put by this stage; overwrite it on purpose with context.replace"),
            ));
        }

        self.provides.push(ty);
        Ok(())
    }
}

impl<'ast> Visit<'ast> for ContextAccess {
    fn visit_local(&mut self, local: &'ast Local) {
        if let (Pat::Ident(pat_ident), Some(init)) = (&local.pat, &local.init)
            && let Some(ty) = Self::bound_read_type(&init.expr)
        {
            self.bindings.insert(pat_ident.ident.clone(), ty);
        }

        visit_local(self, local);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        let Expr::MethodCall(call) = expr else {
            visit_expr(self, expr);
            return;
        };

        let Some(kind) = Self::context_call(call) else {
            visit_expr(self, expr);
            return;
        };

        self.record(call, kind);

        for argument in &call.args {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_path(&mut self, expr_path: &'ast ExprPath) {
        if expr_path.path.is_ident(CONTEXT_RECEIVER) {
            self.errors.push(Error::new_spanned(
                expr_path,
                "#[stage]: use context only through context.get/take/put/replace/push inside run",
            ));
        }

        visit_expr_path(self, expr_path);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        if Self::mentions_context(mac.tokens.clone()) {
            self.errors.push(Error::new_spanned(
                mac,
                "#[stage]: context can't be passed into a macro; use context.get/take/put/replace/push directly",
            ));
        }

        visit_macro(self, mac);
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
    let name = Ident::new(name, Span::call_site());

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

    if let Some(combined) = access.errors.into_iter().reduce(|mut combined, error| {
        combined.combine(error);
        combined
    }) {
        return combined.to_compile_error().into();
    }

    let requires = type_ids_fn("requires", &deduplicated(access.requires));
    let provides = type_ids_fn("provides", &deduplicated(access.provides));

    item_impl.items.push(parse_quote!(#requires));
    item_impl.items.push(parse_quote!(#provides));

    item_impl.into_token_stream().into()
}
