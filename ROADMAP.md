# Roadmap

Big-picture phases, roughly in order. See `TODO.md` for concrete, immediately-actionable items.

## 1. composefs redesign (in progress)

Upac was originally OSTree-based; the project is migrating to composefs-backed atomic deploys
(new repo layout, `composefs`/`composefs-oci` crates, `redb`-backed package/file database). Most
of the read-only surface (`unmutated/`) and the composefs/database layers themselves are done and
tested. What remains:

- **Mutating command bodies** (`lib/lib/src/mutated/*/{transaction,merge,checkout,swap}.rs` and
  similar) are still `todo!()` across every mutating command (install/update/uninstall/rollback/
  commit/files/gc). This is the actual composefs mount/merge/checkout/swap logic — the largest
  remaining piece of work in `upac-lib`.
- Package unpacking itself (`upac-lib`'s own decoder-plugin invocation, checksum, output-dir
  bookkeeping) is done (`PackageUnpacker` in `lib/lib/src/plugin/decoder/unpack.rs`), wired into
  both install's and update's `PreparationStage`.

## 2. upac-cli ABI resync (done)

`upac-cli` was rewritten from the old OSTree-era hand-rolled FFI layer to a thin dlopen frontend
driving `upac-abi`'s C-ABI types directly, with zero business logic of its own (decoding, path
resolution, etc. all live in `upac-lib`). Status:

- Foundation (`libcore.rs`'s `Lib`/`RoSymbols`/`RwSymbols` split, `require_write()` root gate,
  `types/abi.rs` request-building helpers) is done.
- Full command surface is implemented against the current ABI: all read-only commands (`pkg
  list/search/diff`, `commit list`) and all mutating commands (`pkg install/update/remove`,
  `commit new/rollback`, `file add/remove/diff/search`, `gc`, `mime sync`) have real bodies, no
  `todo!()` stubs left in `upac-cli`.
- `#[derive(CNew)]` (in `upac-macro`) replaced manual `struct_size: size_of::<...>()` boilerplate
  across every C-ABI struct construction site in both `upac-lib` and `upac-cli`.

## 3. Config-file / declarative test coverage (done)

`upac-lib`'s own paths are still baked in at build time via `lib.toml`. Separately, a genuine
*runtime* config now exists: `upac_types::settings::RuntimeSettings` reads a shared
`/etc/upac.d/upac.toml` (currently `[gc] retention_depth` and `[progress]` indicatif templates),
parsed independently by both `upac-lib` and `upac-cli` (both link `upac_types` directly) —
best-effort, missing/malformed file falls back to defaults. No dedicated `cli.toml`/`build.rs`
codegen mechanism was needed once this existed.

## 4. Decoders (Zig, `decoders/`)

Per-format decoder shared libraries (deb/rpm/alpm/xbps) exist with real backend logic; see
`decoders/*/` and the README's Decoders section for the current contract (`package_path` +
`output_dir` + SHA-256 checksum in, `PackageMeta` + dependencies out). Remaining work, not
near-term:

- **Packaging pipeline.** Decoder manifest TOML files exist for all 4 decoders
  (`decoders/{alpm,deb,rpm,xbps}/upac-*.toml`), but there's no PKGBUILD/spec/etc packaging and no
  `build.zig` install step that actually copies them (or the built `.so`) to
  `/etc/upac.d/decoders/` — canonical source only, unexercised end-to-end. mime types for
  alpm/xbps (`application/x-alpm-package`/`application/x-xbps-package`) are unofficial
  vendor-prefixed (no shared-mime-info registration exists for either format, unlike deb/rpm).
  `up mime sync` is the mechanism meant to populate desktop integration once decoders are
  actually installed (e.g. from a decoder package's postinstall hook) — not written yet either.
- **Full rewrite to Rust.** The 4 decoders are considered legacy: static linking was never
  started (separate mechanism from the Rust boot plugins' Cargo-feature approach — no Zig-side
  equivalent designed), and `parseVersion`/`CVersion` still populate the pre-`Ver`-brick 4-field
  shape, out of sync with the simplified Rust/C-ABI `{epoch, raw}`. Rather than reconciling that
  drift in Zig, the plan is a full rewrite of the decoders in Rust later — do not invest in
  fixing/extending the Zig side in the meantime.

## 5. Bootstrap / installer concerns (not yet started)

- **Separate library for initial disk layout** — partitioning/formatting a blank disk for a
  first-time install is a distinct concern from `upac-lib` (which manages an already-installed
  system's deploys/packages) and needs its own crate, not bolted onto `upac-lib`. Same lib+CLI
  split as `upac-lib`/`upac-cli`, but the install lib defaults to static linking (it runs once,
  outside any already-deployed system, so there's no shared `.so` to dlopen against yet) — the CLI
  driving it is otherwise a regular thin frontend, same shape as `upac-cli`. Deliberately not
  started before the main mutated pipeline's `TransactionStage`/`CheckoutStage`/`SwapStage` bodies
  are real — the installer needs the same "build tree → commit image → write boot entry →
  one-shot select" logic those stages will implement, and building it first risks duplicating
  work that'll need reworking once `Deploy`'s real write-side semantics land.
- Scope beyond these points not yet defined — expand this section as decisions get made.

## 6. Network package fetching (not yet started)

`FetchingStage` (`install`/`update`) is a real no-op placeholder, not `todo!()` — resolving a
name-based package request (as opposed to a local `--file` path) against a remote repository
needs its own design pass (repository format, index/metadata fetching, download + checksum
verification, mirror/retry policy) before any code lands here.
