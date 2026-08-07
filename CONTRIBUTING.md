# Contributing to Upac

## Before you start

Upac is under active rewrite. New work happens on `lib-rs`, not `main` — please branch off and
target PRs against `lib-rs` unless you're told otherwise. `main` only receives merges once a slice
of `lib-rs` is ready to be the new baseline.

The design and architecture live in
[`doc/UPAC_project_note.en.md`](<doc/UPAC project note.en.md>) — read it before touching the
orchestrator/stage pipeline, the composefs layer, or the FFI boundary; it explains the *why* behind
a lot of decisions that aren't obvious from the code alone.

## Building

See the [README](README.md#-building) for prerequisites and build commands.

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

- No `unwrap()`/`expect()`/`map_err()` in library code — implement `From` for the relevant error
  type and propagate with `?`.
- Imports (`use`) always at the top of the file — no inline or fully-qualified paths.
- No comments unless they explain a non-obvious *why* (a hidden constraint, a workaround, something
  that would surprise a reader). Don't restate what the code already says.
- Long, descriptive names over abbreviations, in both variables and functions.
- When a file grows past one logical unit, split it into a folder with a `mod.rs`, not a single
  giant file.

## Tests

- Anything with a real `pub` surface gets a `tests/` integration test file in that crate.
- Inline `#[cfg(test)]` is only for genuinely private modules with no public surface to test through.
- Plain `#[test]`, no test frameworks (no `rstest`, etc).

## Pull requests

Keep PRs scoped to one change. Make sure `cargo build --workspace`, `cargo test --workspace --lib
--tests`, `cargo clippy`, and `reuse lint` all pass before opening — see the PR template's checklist.
