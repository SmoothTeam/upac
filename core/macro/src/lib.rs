// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

//! Proc-macro crate for UPAC. Each derive reflects over a struct's (or
//! enum's) fields at compile time to generate boilerplate that would
//! otherwise need `inline for (std.meta.fields)`-style manual maintenance:
//!   CEnum          - TryFrom<uN>/From<Enum> for a #[repr(uN)] enum carried as its raw integer
//!   CFree          - unsafe free() releasing every owned C-ABI buffer
//!   CNew           - new(...) constructor, one param per field, struct_size computed
//!   RustToC        - Rust domain type -> its C-ABI mirror (outbound)
//!   CTryToRust     - C-ABI struct -> Rust domain type, fallible (inbound)
//!   CValidate      - unsafe validate() checking struct_size + every field
//!   ContextValue   - Deref/DerefMut/From<T> for a single-field tuple struct
//!   FromStageIndex - orchestrator stage index -> enum variant (by position)
//!   StageKey       - enum variant -> "stage_snake_case" gettext key (by name)
//!   stage          - attribute on `impl Stage`: requires()/provides() from the run body's context use
//!   JsonCodec      - to_json()/from_json() for on-disk records outside the redb DB
//!
//! `#[proc_macro_derive]` functions must live at the crate root, so each of
//! the derives below is a thin wrapper delegating into its own module, which
//! holds the actual field-dispatch logic.

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
pub fn derive_cfree(input: TokenStream) -> TokenStream {
    c_free::expand(input)
}

#[proc_macro_derive(CNew)]
pub fn derive_c_new(input: TokenStream) -> TokenStream {
    c_new::expand(input)
}

#[proc_macro_derive(RustToC, attributes(none_if))]
pub fn derive_rust_to_c(input: TokenStream) -> TokenStream {
    rust_to_c::expand(input)
}

#[proc_macro_derive(CTryToRust, attributes(none_if))]
pub fn derive_c_try_to_rust(input: TokenStream) -> TokenStream {
    c_try_to_rust::expand(input)
}

#[proc_macro_derive(CValidate, attributes(optional, non_empty, bitflags, skip_if))]
pub fn derive_cvalidate(input: TokenStream) -> TokenStream {
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
