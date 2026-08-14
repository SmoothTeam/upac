# Contributing to Upac

## Before you start

Upac is under active rewrite. New work happens on `lib-rs`, not `main` — please branch off and
target PRs against `lib-rs` unless you're told otherwise. `main` only receives merges once a slice
of `lib-rs` is ready to be the new baseline.

The design and architecture live in the per-chapter design notes under
[`doc/eng/`](doc/eng/) (canonical) / [`doc/rus/`](doc/rus/) — read them before touching the
orchestrator/stage pipeline, the composefs layer, or the FFI boundary; they explain the *why* behind
a lot of decisions that aren't obvious from the code alone.

## Building

See the [README](README.md#-building) for prerequisites and build commands.

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
- Imports (`use`) always at the top of the file — no inline or fully-qualified paths.
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

## Tests

- Anything with a real `pub` surface gets a `tests/` integration test file in that crate.
- Inline `#[cfg(test)]` is only for genuinely private modules with no public surface to test through.
- Plain `#[test]`, no test frameworks (no `rstest`, etc).

## Pull requests

Keep PRs scoped to one change. Make sure `cargo build --workspace`, `cargo test --workspace --lib
--tests`, `cargo clippy`, and `reuse lint` all pass before opening — see the PR template's checklist.
