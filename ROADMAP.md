<h1 align="center">🧭 Roadmap</h1> 

**Big-picture phases, roughly in order. See `TODO.md` for concrete, immediately-actionable items.**

### **Project-wide sequencing, phases:**

1. **Local-only, fully working** (current phase) — every command, including the bootstrap installer (§5), works end-to-end against local input only (`--file`/`--source` paths, no network). This is the bar for calling upac itself *"done"* as a local-first package manager.
2. **Network** (§6) — name-based package resolution against a remote repository (index/metadata fetching, download + checksum verification, mirror/retry policy), plus wiring a network payload source into the bootstrap installer.
3. **Dependency-resolution plugins** (§7) — pluggable dependency-graph/constraint resolution, built on top of whatever repository/index format phase 2 settles on.
4. **AUR Integration** — integration of isolated AUR package builds, similar to `paru`, only in an isolated environment.
5. **Flatpak Integration** — possible further expansion of flatpak functionality, allowing this package distribution format to work directly in the `up` CLI.

Phases 2 and 3 are distinct, roughly year-long undertakings that have not yet been designed; do not begin either of them until Phase 1 is truly complete. Phases 4 and 5, as well as the further definition of project goals, are scheduled for the following year, once the core functionality comprising Phases 2 and 3 has been completed.

## 1. Composefs redesign — **done**.

Upac was originally OSTree-based. The project is migrated to composefs-backed atomic deploys
(new repo layout, `composefs`/`composefs-oci` crates, `redb`-backed package/file database). Both
the read-only surface (`unmutated/`) and the composefs/database layers are done and tested.

**Every mutating command's entire pipeline is real end-to-end** — install/update/uninstall/rollback/commit/files/gc all have genuine `transaction`/`merge`/`checkout`/`swap` bodies (no
`todo!()` anywhere in `lib/lib/src`), including the boot-plugin subsystem (pluggable UKI/systemd-boot/grub/rEFInd one-shot reboot selection), the real §5.1 3-way `/etc` merge with `.upac-new` conflict handling, and deploy retention/pinning. Package unpacking itself (`upac-lib`'s own decoder-plugin invocation, checksum, output-dir bookkeeping) is done (`PackageUnpacker` in `lib/lib/src/plugin/decoder/unpack.rs`), wired into both install's and update's `PreparationStage`.

What's still explicitly out of scope for this phase, tracked separately below: network-based
package fetching (§6) and the first-boot bootstrap installer (§5).

## 2. Upac-cli ABI resync — **done**.

`upac-cli` was rewritten from the old OSTree-era hand-rolled FFI layer to a thin dlopen frontend
driving `upac-abi`'s C-ABI types directly, with zero business logic of its own (decoding, path
resolution, etc. all live in `upac-lib`).

**Status:**
- Foundation (`libcore.rs`'s `Lib`/`RoSymbols`/`RwSymbols` split, `require_write()` root gate, `types/abi.rs` request-building helpers) is done.
- Full command surface is implemented against the current ABI: all read-only commands (`pkg list/search/diff`, `commit list`) and all mutating commands (`pkg install/update/remove`, `commit new/rollback`, `file add/remove/diff/search`, `gc`, `mime sync`) have real bodies, no `todo!()` stubs left in `upac-cli`.
- `#[derive(CNew)]` (in `upac-macro`) replaced manual `struct_size: size_of::<...>()` boilerplate across every C-ABI struct construction site in both `upac-lib` and `upac-cli`.

## 3. Config-file / declarative test coverage — **done**.

`upac-lib`'s own paths are still baked in at build time via `lib.toml`. Separately, a genuine *runtime* config now exists: `upac_types::settings::RuntimeSettings` reads a shared `/etc/upac.d/upac.toml` (currently `[gc] retention_depth` and `[progress]` indicatif templates), parsed independently by both `upac-lib` and `upac-cli` (both link `upac_types` directly) — best-effort, missing/malformed file falls back to defaults.

## 4. Decoders (Zig, `decoders/`) — **in progress**.

Per-format decoder shared libraries (deb/rpm/alpm/xbps) exist with real backend logic; see `decoders/*/` and the README's Decoders section for the current contract (`package_path` + `output_dir` + SHA-256 checksum in, `PackageMeta` + dependencies out). Remaining work, not near-term:

- **Packaging pipeline.** Decoder manifest TOML files exist for all 4 decoders 
(`decoders/{alpm,deb,rpm,xbps}/upac-*.toml`), but there's no PKGBUILD/spec/etc packaging, Mime types for alpm/xbps (`application/x-alpm-package`/`application/x-xbps-package`) are unofficial and vendor-prefixed (no shared-mime-info registration exists for either format, unlike deb/rpm). `up mime sync` is the mechanism meant to populate desktop integration once decoders are actually installed if any problems arise with automatic updating—not written yet either.
- **Full rewrite to Rust.** The 4 decoders are considered legacy: static linking was never started (separate mechanism from the Rust boot plugins' Cargo-feature approach — no Zig-side equivalent designed), and `parseVersion`/`CVersion` still populate the pre-`Ver`-brick 4-field shape, out of sync with the simplified Rust/C-ABI `{epoch, raw}`. Rather than reconciling that drift in Zig, the plan is a full rewrite of the decoders in Rust later — do not invest in fixing/extending the Zig side in the meantime.

## 5. Bootstrap / installer concerns (scaffolded, design settled, not yet implemented) — **in progress**.

- **Separate library for initial disk layout** — partitioning/formatting a blank disk for a first-time install is a distinct concern from `upac-lib` (which manages an already-installed system's deploys/packages). Lives in its own crate, `lib/install` (`upac-install`), a workspace member since `21aae2a`. Same lib+CLI split as `upac-lib`/`upac-cli`, but the install lib defaults to static linking (it runs once, outside any already-deployed system, so there's no shared `.so` to dlopen against yet) — depends on `upac-lib`/`upac-types`/`composefs` directly as ordinary Rust dependencies, not through the C ABI. The CLI driving it (`user/install-cli`, not created yet) is otherwise a regular thin frontend, same shape as `upac-cli`.
- **No separate "empty seed" deploy.** The very first deploy the installer creates already contains upac itself, tracked as a normal installed package — not an empty tree followed by a second `up pkg install` pass. This works because the package-database write (`MetaStoreMut::insert_package_meta` + `FileStoreMut::insert_package_file`, `lib/lib/src/database/{meta,files}.rs`) is fully decoupled from the decoder-plugin/`PackageUnpacker` machinery — the installer calls `FileHandle::import_directory` to lay a fixed set of pre-built upac binaries (`up`, `libupac.so`, boot plugins, decoders, default configs) into a fresh tree, registers that same tree's files as an "upac" package directly against those two DB traits, then `commit_tree`s it, writes the first `DeployRecord` (`allocate_seq`), and writes the boot entry + one-shot select (`write_boot_entry`, `plugin::boot::resolve_boot_plugin`) — all bricks that already exist, reused as-is, no decoder/`PackageUnpacker` involvement at all for this bootstrap case.
- **Payload source: local-only for phase 1** (see the project-wide sequencing note above) — the
installer CLI's source flag (`--source <path>`, exact name TBD) is **required**, same convention
as `up pkg install -f` (`README.md`'s own "`-f` is local-only; a future network form will resolve by name instead"). No network payload source is implemented this phase; the intended extension point is a `PayloadSource` enum/trait (`LocalPath` now, a `Network` variant added once §6 lands), not a half-built network path today.
- Self-uninstall (`up pkg remove upac`) needs no special protection — composefs's atomicity already makes it safe (the running deploy is untouched until next boot; rollback covers a change of mind before the old deploy is GC'd). Not building any guard against it.
- Partitioning/formatting a blank disk (GPT: ESP + deployment partition + `/var` + `/home`, per chapter 3's map) and mounting them is still fully unwritten — no crate chosen yet for partitioning. The crates that can be enabled for use are already included in `Cargo.toml`; unfortunately, the rest will have to be implemented via an external process.

## 6. Network package fetching — **not yet started**.

`FetchingStage` (`install`/`update`) is a real no-op placeholder, not `todo!()` — resolving a name-based package request (as opposed to a local `--file` path) against a remote repository needs its own design pass (repository format, index/metadata fetching, download + checksum verification, mirror/retry policy) before any code lands here. Also gates the installer's `PayloadSource::Network` variant (§5).

## 7. Dependency-resolution plugins — **not yet started**.

Pluggable dependency-graph/constraint resolution — no design started, blocked on §6 settling a repository/index format to resolve against first. `Dependency.constraint: u8` (`lib/types/src/lib.rs`) is an already-defined but currently-unused bitflag presumably meant for this.
