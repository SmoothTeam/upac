// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use proc_macro::TokenStream;

mod c_enum;
mod c_free;
mod c_new;
mod c_try_to_rust;
mod c_validate;
mod common;
mod context_value;
mod from_stage_index;
mod rust_to_c;
mod stage;
mod stage_key;

#[proc_macro_derive(CEnum)]
pub fn derive_c_enum(input: TokenStream) -> TokenStream {
    c_enum::expand(input)
}

#[proc_macro_derive(CFree)]
pub fn derive_c_free(input: TokenStream) -> TokenStream {
    c_free::expand(input)
}

#[proc_macro_derive(CNew)]
pub fn derive_c_new(input: TokenStream) -> TokenStream {
    c_new::expand(input)
}

#[proc_macro_derive(RustToC)]
pub fn derive_rust_to_c(input: TokenStream) -> TokenStream {
    rust_to_c::expand(input)
}

#[proc_macro_derive(CTryToRust)]
pub fn derive_c_try_to_rust(input: TokenStream) -> TokenStream {
    c_try_to_rust::expand(input)
}

#[proc_macro_derive(CValidate, attributes(optional, non_empty, bitflags, skip_if))]
pub fn derive_c_validate(input: TokenStream) -> TokenStream {
    c_validate::expand(input)
}

#[proc_macro_derive(ContextValue)]
pub fn derive_context_value(input: TokenStream) -> TokenStream {
    context_value::expand(input)
}

#[proc_macro_derive(FromStageIndex)]
pub fn derive_from_stage_index(input: TokenStream) -> TokenStream {
    from_stage_index::expand(input)
}

#[proc_macro_derive(StageKey)]
pub fn derive_stage_key(input: TokenStream) -> TokenStream {
    stage_key::expand(input)
}

#[proc_macro_attribute]
pub fn stage(_attr: TokenStream, item: TokenStream) -> TokenStream {
    stage::expand(item)
}
