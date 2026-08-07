// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

//! Proc-macro crate for UPAC. Each derive reflects over a struct's (or
//! enum's) fields at compile time to generate boilerplate that would
//! otherwise need `inline for (std.meta.fields)`-style manual maintenance:
//!   CFree          - unsafe free() releasing every owned C-ABI buffer
//!   RustToC        - Rust domain type -> its C-ABI mirror (outbound)
//!   CTryToRust     - C-ABI struct -> Rust domain type, fallible (inbound)
//!   CToRust        - C-ABI struct -> Rust domain type, infallible (inbound)
//!   CValidate      - unsafe validate() checking struct_size + every field
//!   FromStageIndex - orchestrator stage index -> enum variant (by position)
//!   RedbCodec      - encode_into()/decode_from() for the redb key-value store
//!   JsonCodec      - to_json()/from_json() for on-disk records outside the redb DB
//!
//! `#[proc_macro_derive]` functions must live at the crate root, so each of
//! the derives below is a thin wrapper delegating into its own module, which
//! holds the actual field-dispatch logic.

use proc_macro::TokenStream;

mod c_free;
mod c_to_rust;
mod c_try_to_rust;
mod c_validate;
mod common;
mod from_stage_index;
mod json_codec;
mod redb_codec;
mod rust_to_c;

#[proc_macro_derive(CFree)]
pub fn derive_cfree(input: TokenStream) -> TokenStream {
    c_free::expand(input)
}

#[proc_macro_derive(RustToC)]
pub fn derive_rust_to_c(input: TokenStream) -> TokenStream {
    rust_to_c::expand(input)
}

#[proc_macro_derive(CTryToRust)]
pub fn derive_c_try_to_rust(input: TokenStream) -> TokenStream {
    c_try_to_rust::expand(input)
}

#[proc_macro_derive(CToRust)]
pub fn derive_c_to_rust(input: TokenStream) -> TokenStream {
    c_to_rust::expand(input)
}

#[proc_macro_derive(CValidate, attributes(optional, non_empty))]
pub fn derive_cvalidate(input: TokenStream) -> TokenStream {
    c_validate::expand(input)
}

#[proc_macro_derive(FromStageIndex)]
pub fn derive_from_stage_index(input: TokenStream) -> TokenStream {
    from_stage_index::expand(input)
}

#[proc_macro_derive(RedbCodec)]
pub fn derive_redb_codec(input: TokenStream) -> TokenStream {
    redb_codec::expand(input)
}

#[proc_macro_derive(JsonCodec)]
pub fn derive_json_codec(input: TokenStream) -> TokenStream {
    json_codec::expand(input)
}
