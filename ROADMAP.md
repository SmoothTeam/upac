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
`todo!()` anywhere in `lib/lib/src`), including the boot-plugin subsystem (pluggable UKI/systemd-boot/grub/rEFInd one-shot reboot selection), the real §5.1 3-way `/etc` merge with `.upac-new` conflict handling, and deploy retention/pinning. Package unpacking itself (`upac-lib`'s own decoder-plugin invocation, checksum, output-dir bookkeeping) is done (`PackageUnpacker` in `lib/lib/src/plugin/decoder/unpack.rs`), wired into both install's and update's `PreparationStage`. **Declarative (package-format-native) triggers are also fully wired**: `PackageUnpacker` carries each decoded package's `DeclarativeTrigger{format, triggers}` through `Context` as its own key (separate from `PackageTemp`, so `TransactionStage`'s `context.take::<Vec<PackageTemp>>()` doesn't remove it), install/update persist it into the new `packages_triggers` database table (`database::triggers`, keyed by the package's `Uuid`) alongside `PackageMeta`, uninstall reads it back from there (no re-`decode()`), and `HookStage` — via a new `Timing::Declarative` position on `PipelineTrigger`, firing right before each command's Post hooks — matches the stored trigger names against `build_trigger_table()`'s per-format table and runs whatever hooks matched.

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

## 4. Decoders (`decoders/`) — **done**.

Per-format decoder shared libraries (alpm/deb/rpm/xbps) exist with real backend logic; see `decoders/*/` and the README's Decoders section for the current contract (`package_path` + `output_dir` + SHA-256 checksum in, `PackageMeta` + dependencies + declarative triggers out).

**All four decoders — `alpm`, `deb`, `rpm` and `xbps` — are done and written in Rust** (`decoders/{alpm,deb,rpm,xbps}`, `[lib] name = "upac_decoder_{alpm,deb,rpm,xbps}"`), `xbps` being the last, having replaced the legacy Zig implementation. Each decoder's pipeline is `verify` (SHA-256) → `extract` (format-specific archive + compression handling) → a `DecodeMeta::decode` implementation producing `PackageMeta` + `Vec<Dependency>` → `triggers::scan` (declarative trigger names, matched against the shared `upac_types::DecoderTrigger` 6-point enum). Total test count: 12 (`alpm`) + 11 (`deb`) + 13 (`rpm`) + 10 (`xbps`).

All four share `upac_types::decoder::{DecodedMeta, DecodeMeta}` for the metadata-decoding step (`DecodedMeta` the common output struct, `DecodeMeta` a trait implemented on each crate's own source type — a trait rather than a shared inherent impl because of Rust's orphan rule) and the shared `upac_types::decoder::read_to_string<R: Read>` helper, plus a strict "no free-floating helper functions" convention (every helper is either inlined at its call site or a private associated function on the relevant type). Format-specific constants live in a shared `decoders/decoder.toml`, each crate's `build.rs` reading only its own section. The dependency-version-constraint parsing (`<=`/`>=`/etc. → `CONSTRAINT_*` bitflags) is shared as `upac_abi::decoder::parse_constraint_prefix`; `rpm` doesn't need it since its `REQUIREFLAGS` tag is already a clean bitflag.

All four decoders can also be **statically linked** into `upac-lib`, mirroring the boot-plugin `builtin-*` Cargo-feature mechanism (`builtin-alpm`/`builtin-deb`/`builtin-rpm`/`builtin-xbps` + shared `builtin-decoders` gate) — see `lib/lib/src/plugin/decoder/mod.rs`'s `static_decoders()`.

Remaining work, not near-term:

- **Packaging pipeline.** Decoder manifest TOML files exist for all 4 decoders
(`decoders/{alpm,deb,rpm,xbps}/upac-*.toml`), but there's no PKGBUILD/spec/etc packaging, Mime types for alpm/xbps (`application/x-alpm-package`/`application/x-xbps-package`) are unofficial and vendor-prefixed (no shared-mime-info registration exists for either format, unlike deb/rpm). `up mime sync` (populates desktop/mime-type integration from installed decoder manifests) is already fully implemented — see §1 — this item is just "no decoder is actually packaged/installed yet for it to sync against."

## 5. Bootstrap / installer concerns — **pipeline done, boot-to-a-working-system not yet proven**.

**Correction (found via a live QEMU/VM boot test, not just reading the code):** everything
genesis writes to disk (composefs repo objects, deploy record, BLS boot entry, and — after this
session's fix — the bootloader binary itself on the ESP) is correct and verified present on disk.
But the resulting disk does not actually boot into the installed system yet: `systemd-gpt-auto-generator`
hangs forever on `/dev/gpt-auto-root`, because `partition.rs`'s `LINUX_PARTITION_TYPE_GUID`
(`0fc63daf-...`, generic "Linux filesystem data") is not the discoverable-root GUID
(`4f68bce3-...`, "Linux root x86-64") systemd's root-autodetection looks for. Even fixing that
GUID only gets partition auto-mount working — composefs systems don't boot by mounting a partition
as `/` directly; the kernel cmdline's `composefs.digest=` needs a dedicated initramfs hook (dracut
module or mkinitcpio hook, matching how real composefs distros do it) to actually resolve the
digest against the on-disk repository, mount the erofs image with fs-verity, and overlay
`state/deploy/<digest>/etc/` on top. **This hook does not exist anywhere in the project** —
tracked as a new, unstarted, non-trivial subsystem in `TODO.md`.

- **Own crate, `lib/setup` (`upac-setup`)** — partitioning/formatting a blank disk and running a system's first deploy, statically linked (no dlopen). Has a full C-ABI request surface (`CSetupExistingRequest`/`CSetupWholeDiskRequest`) for structural consistency, but `up-sp` bypasses it — builds `SetupExistingData`/`SetupWholeDiskData` as plain struct literals and calls `.run()` directly.
- **No separate "empty seed" deploy.** The first deploy already contains upac itself as a normal installed package, since the package-database write is decoupled from the decoder-plugin machinery — genesis imports a pre-built `source_dir` straight into the DB and composefs trees.
- **Two request modes, both real.** `SetupExistingData` mounts caller-provided already-partitioned devices; `SetupWholeDiskData` partitions a blank whole-disk device itself (GPT via `gptman`, btrfs/ESP natively, ext4/xfs by shelling out to `mkfs.*`) before mounting.
- **Genesis pipeline** is a real `Context`/`Stage`/`SequentialOrchestrator` pipeline (`lib/setup/src/genesis/`, four stages: `ReadMetaStage`/`ImportTreesStage`/`WriteDeployRecordStage`/`StageBootStage`) — deliberately has no `StateId`/`ErrorDomain` since genesis never crosses the C-ABI; progress still reaches the caller via `hook_message`.
- **`up-sp` CLI** — whole-disk mode is the top-level default, existing-partitions mode is `up-sp manual`. Own `indicatif` progress bar and own gettext catalogs, embedded into the binary and extracted at startup (no `/usr/share/locale` assumption in a rescue environment).
- **Payload source is local-only for phase 1** — genesis always reads a local `source_dir`, same as `up pkg install -f`.
- Self-uninstall (`up pkg remove upac`) needs no special guard — composefs's atomicity already makes it safe.

## 6. Network package fetching — **not yet started**.

`FetchingStage` (`install`/`update`) is a real no-op placeholder, not `todo!()` — resolving a name-based package request (as opposed to a local `--file` path) against a remote repository needs its own design pass (repository format, index/metadata fetching, download + checksum verification, mirror/retry policy) before any code lands here. Also gates the installer's `PayloadSource::Network` variant (§5).

## 7. Dependency-resolution plugins — **not yet started**.

Pluggable dependency-graph/constraint resolution — no design started, blocked on §6 settling a repository/index format to resolve against first. `Dependency.constraint: u8` (`lib/types/src/lib.rs`) is an already-defined but currently-unused bitflag presumably meant for this.
