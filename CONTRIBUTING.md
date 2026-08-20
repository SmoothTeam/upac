# Contributing to Upac

## Before you start

Upac is under active rewrite. New work happens on `lib-rs`, not `main` — please branch off and
target PRs against `lib-rs` unless you're told otherwise. `main` only receives merges once a slice
of `lib-rs` is ready to be the new baseline.

The design and architecture live in the per-chapter design notes under
[`doc/eng/`](doc/eng/) (canonical) / [`doc/rus/`](doc/rus/) — read them before touching the
orchestrator/stage pipeline, the composefs layer, or the FFI boundary; they explain the *why* behind
a lot of decisions that aren't obvious from the code alone.

For current work status, check [`ROADMAP.md`](ROADMAP.md) (bigger phases) and [`TODO.md`](TODO.md)
(near-term, concrete items) before picking something up — they're kept up to date, unlike design
docs which describe intent rather than progress.

## Building

See the [README](README.md#-building) for prerequisites and build commands.

## Git hooks

Tracked hooks live in [`.githooks/`](.githooks/):

- `pre-commit` runs `cargo fmt --all` across the whole workspace before each commit and re-stages
  whatever it reformats, so formatting never drifts and `cargo fmt --all -- --check` in CI never
  fails on something that should've been caught locally.
- `commit-msg` lowercases the first letter after a `fix:`/`new:` prefix, matching this repo's
  commit message convention.

Git doesn't run tracked hooks automatically (by design — cloning a repo shouldn't execute
arbitrary code), so opt in once per clone:

```sh
git config core.hooksPath .githooks
```

## Repo tree in the docs

The directory tree embedded in each design chapter (between `<!-- tree:start -->`/`<!-- tree:end
-->` markers) is generated, not hand-edited. Run `cargo xtask gen-tree` after a structural change
(new crate, moved directory) to refresh it, or `cargo xtask gen-tree --check` to verify it's still
current before opening a PR.

## License and REUSE

This repo is [REUSE](https://reuse.software/)-compliant and multi-licensed by directory:

- `lib/*` (`upac-abi`, `upac-macro`, `upac-lib`, `upac-pki`) — LGPL-3.0-or-later
- `user/*` (`upac-cli`, `upac-sign-cli`) — GPL-3.0-only
- `doc/*` — CC-BY-SA-4.0

Every new source file needs an SPDX header matching its directory's license, e.g.:

```rust
// SPDX-FileCopyrightText: 2026 <your name>
//
// SPDX-License-Identifier: LGPL-3.0-or-later
```

For files where a comment header doesn't make sense (e.g. Markdown, TOML), add an annotation to
`REUSE.toml` instead. Run `reuse lint` before submitting — it must be clean.

## Code style

- No `unwrap()`/`expect()` in library/binary code — everything fallible returns `Result` and
  propagates. Default to `impl From<Source> for TargetError` + bare `?`. Reach for `map_err()`
  instead only when a single unconditional `From` can't express the conversion: the same source
  error type needs to become a different local variant depending on the call site (no context to
  pick from in a `From` impl), the target `Result`'s `Err` isn't a plain error type (e.g. a
  `(StateId, Error)` tuple), you're chaining through two `From` hops with no direct one, or you need
  to attach context (a message, an original input string) that `From` can't carry.
- Imports (`use`) always at the top of the file — no inline or fully-qualified paths. Within a
  command file that defines its own `pub struct Args`, `clap::Args` (the derive macro) collides
  with it by name — resolve that with `use clap::Args as ClapArgs;` + `#[derive(ClapArgs)]`, not
  an inline `#[derive(clap::Args)]`. The one real exception left: `main.rs`'s `Command` enum
  references multiple different modules' `Args` structs by the same name (`generate_root::Args`,
  `sign_hook::Args`, ...) — importing all of them under one local name isn't possible, so those
  call sites use the full path (`commands::sign_hook::Args`) instead. Don't reach for a
  fully-qualified path to avoid an otherwise ordinary import outside that one case.
- Same rule for error types used in `impl From<...>`: import the source error type at the top via
  `use some_crate::Error as SomeCrateError;` (or the type's real name if it isn't already called
  `Error`) and reference the alias in the impl — never write `impl From<some_crate::Error>` with
  the path inline. This applies even when nothing else in the file collides with the bare name;
  the alias makes it obvious which crate's error is being converted without hunting through the
  file. The one exception: a `macro_rules!` macro whose body references another crate's type via a
  crate-qualified path (e.g. `regex::Error` inside `regex_error_from!` in `errors.rs`) — such a
  path resolves through the extern prelude at every call site with zero extra imports needed;
  replacing it with a locally-aliased bare name would instead force every file that invokes the
  macro to add its own redundant `use` just to satisfy it. Don't do that swap inside a
  multi-call-site macro body.
- No comments unless they explain a non-obvious *why* (a hidden constraint, a workaround, something
  that would surprise a reader). Don't restate what the code already says.
- Long, descriptive names over abbreviations, in both variables and functions.
- When a file grows past one logical unit, split it into a folder with a `mod.rs`, not a single
  giant file.
- Command families under `mutated`/`unmutated` (e.g. `diff`/`diff_packages`/`diff_prefix`/
  `diff_config`, or `search_meta`/`search_files`) stay flat, one folder per command — don't
  nest them under a shared group folder. The common name prefix already groups them in any
  directory listing; nesting would only add import/path churn without a real benefit at the
  current command count.

### Cargo.toml conventions

- `[package]`: `name` is always the very first field. `description` (when the crate has one)
  goes right after `name`, not down with the rest of the block — then every `.workspace = true`
  field, then any crate-specific field (license override, etc.).
- `[dependencies]`: bracketed entries with multiple keys first, ordered by descending key count;
  then, after a blank line, single-line/bare-version entries. Internal `upac-*` crates get their
  own subgroup at the very top, ahead of everything else — even other `{ workspace = true }`
  entries.

### TOML configs and constants

- Flat `key = value` TOML configs (`rustfmt.toml` and similar — not `Cargo.toml`): bool fields,
  then string fields, then number fields, each group separated by a blank line.
- No `const X: &Type = value;` for a fixed value (a path, a protocol string, a GUID, ...) directly
  in a `.rs` file — ever. It goes in a `.toml` file (`lib.toml`, or a crate-local equivalent like
  `booter.toml`) with a comment explaining what the value is and why it's fixed, read in via a
  `build.rs` codegen step. The only thing allowed to live as a literal in Rust source is a genuine
  cross-language C-ABI contract expressed as a real type (an enum discriminant, a `#[repr(C)]`
  field) — never a bare string/scalar constant standing in for one.

### File and item ordering

- Top-level ordering inside a Rust file: `use` → `macro_rules!` blocks → type aliases (simple ones
  first, then function-pointer types, with a blank line between each fn-pointer type) → enums →
  (`trait` → `struct` → `impl`), repeating the trait/struct/impl group per logical unit in the
  file.
- In a file that exports `extern "C"` symbols: the exported `extern "C" fn`s go at the top of the
  file (right after imports and any `include!`-generated constants), other `pub fn`s next, private
  `fn`s last — the C-ABI surface is what a reader of that file needs to find first.
- Within the `use` block: `pub use` (if any) first, then plain unconditional `use` (grouped
  std/external/`crate::` as usual), then every `#[cfg(...)]`-gated `use` last, each as its own
  block separated by blank lines — a reader should see what's always compiled before what's
  conditional.
- For every other item in the file (not `use`), the split is the other way round: right after the
  `mod` declarations, every `#[cfg(...)]`-gated item (including `#[cfg(not(...))]` variants) comes
  first, grouped together; the unconditional functions/structs/impls that follow the normal
  top-level ordering above come after that.

### Macros

- `macro_rules!` only for a concrete, present need — never a macro whose only job is calling other
  macros, and never one written for uniformity that only holds while the code it abstracts over is
  still unfinished (`todo!()`).
- Each `macro_rules!` block is immediately followed by its own visibility line (e.g.
  `pub(crate) use macro_name;`) — don't write the macro, move on, and group all the visibility
  lines together separately later.

### Naming

- Dispatcher-style functions (e.g. `field_path_validate`): the verb always goes last, with
  modifiers (`path`, `ptr`, ...) before it.

### Types over free functions

- Prefer methods/associated consts on a type over free functions/module-level consts whenever
  there's a natural owning type for them — e.g. `Uki::probes()`/`Self::BOOT_NEXT_VAR`, not a bare
  `probes()`/`BOOT_NEXT_VAR` floating in the module. Free functions are still fine when there's
  genuinely no owning type (a generic FFI-symbol-loading helper shared across unrelated callers).
- Prefer a newtype + `impl Display` over a free `format_x(&T) -> String` function or a raw unsafe
  accessor; reuse an existing safe `TryFrom` impl instead of calling the unsafe conversion
  directly.
- When moving a type into its own crate breaks a foreign-trait impl (orphan rule), wrap it in a
  `#[repr(transparent)]` newtype and cast via `from_ref()` — don't clone to route around it.

### Extracting helpers

- Ask before extracting a one-off call into its own function — don't do it silently. A helper
  earns its existence only once it does real work for 2+ call sites; a thin pass-through gets
  duplicated instead of extracted.

### Re-exports

- Never `pub use self::module::Item` at a crate or module root — always reach a type through its
  full module path.

### Cross-crate constants

- When two or more otherwise-independent crates need the exact same constant, share the *data* —
  one `.toml` file each crate's own `build.rs` reads independently — rather than introducing a
  compiled shared crate as a dependency between them. Keeps the crates independent while still
  giving the value a single source of truth.
- A `lib.toml`-style `build.rs` + TOML pair only earns a dedicated `layout` module wrapper when
  there's more than one section to namespace; for a single-section config, plain
  `include!(concat!(env!("OUT_DIR"), "/layout.rs"))` at the crate root is simpler.

## Tests

- Anything with a real `pub` surface gets a `tests/` integration test file in that crate.
- Inline `#[cfg(test)]` is only for genuinely private modules with no public surface to test through.
- Plain `#[test]`, no test frameworks (no `rstest`, etc).

## Pull requests

Keep PRs scoped to one change. Make sure `cargo build --workspace`, `cargo test --workspace --lib
--tests`, `cargo clippy`, and `reuse lint` all pass before opening — see the PR template's checklist.
