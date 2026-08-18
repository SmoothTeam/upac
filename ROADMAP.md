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

## 2. upac-cli ABI resync (in progress)

`upac-cli` was rewritten from the old OSTree-era hand-rolled FFI layer to a thin dlopen frontend
driving `upac-abi`'s C-ABI types directly, with zero business logic of its own (decoding, path
resolution, etc. all live in `upac-lib`). Status:

- Foundation (`libcore.rs`'s `Lib`/`RoSymbols`/`RwSymbols` split, `require_write()` root gate,
  `types/abi.rs` request-building helpers) is done.
- Read-only commands (`pkg list/search/diff`) and most mutating commands (`pkg install/update/
  remove`, `commit new/rollback`) are implemented against the current ABI.
- `commit list`, `file remove/diff/search` still need work — see `TODO.md`.
- `#[derive(CNew)]` (in `upac-macro`) replaced manual `struct_size: size_of::<...>()` boilerplate
  across every C-ABI struct construction site in both `upac-lib` and `upac-cli`.

## 3. Config-file / declarative test coverage

`upac-cli` no longer reads any config file at all (paths are baked into `upac-lib` via `lib.toml`
at build time) — `config.rs`/`Config` were removed entirely. No further work planned here unless
a genuine CLI-only setting (not ABI-related) comes up.

## 4. Decoders (Zig, `decoders/`)

Per-format decoder shared libraries (deb/rpm/alpm/xbps) exist as scaffolding; not tracked in
detail here — see `decoders/*/` and the README's Decoders section for the current contract
(`package_path` + `output_dir` + SHA-256 checksum in, `PackageMeta` + dependencies out).

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
- **Static linking tests** — verify the CLI/decoders/whatever else is meant to be statically
  linked actually builds and runs that way; not yet covered by any test in the workspace.
- Scope beyond these two points not yet defined — expand this section as decisions get made.
