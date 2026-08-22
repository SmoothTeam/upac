# TODO

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

- Decoder manifest TOML files now exist for all 4 Zig decoders
  (`decoders/{alpm,deb,rpm,xbps}/upac-*.toml`), but there's still no packaging pipeline
  (PKGBUILD/spec/etc) or `build.zig` install step to actually copy them to
  `/etc/upac.d/decoders/` — they're canonical source only, unexercised end-to-end until packaging
  exists. mime types for alpm/xbps (`application/x-alpm-package`/`application/x-xbps-package`) are
  unofficial vendor-prefixed — shared-mime-info has no registered type for either format, only for
  deb/rpm. `user/upac-cli/data/{upac-mime.xml,*.desktop}` were deliberately emptied of the mime
  types they used to hardcode: upac itself ships no decoders, so the shipped bootstrap files claim
  no format support out of the box — `up mime sync` is what populates them, meant to run once
  decoders are actually installed (e.g. from a decoder package's postinstall hook, not written
  yet).

## upac-lib

- `rollback` is the first mutating command with a fully real pipeline body:
  `MergeStage`/`CheckoutStage`/`SwapStage` all done (resolves the target prefix digest, writes the
  ESP boot entry via `boot::write_boot_entry` — now self-naming the entry from the image's own
  boot resource type instead of taking a caller-supplied name — and selects it for one-shot boot
  through the boot-plugin subsystem's `plugin::boot::resolve_boot_plugin`). New bricks along the
  way: `deploy::esp::find_esp_mount` (ESP mount-point discovery) and
  `composefs::repository::object_id_from_hex` (digest string → `ObjectID`). The closed
  `upac_abi::BootKind` enum (`Auto`/`Uki`/`Bls`) is gone — every mutating C-ABI request now takes
  an open `boot_plugin: CSlice` (plugin name, empty = autodetect) instead, and `--boot` on the CLI
  is a plain string.
- `install`'s `TransactionStage` is now real: opens the currently-booted `/usr` tree, imports each
  unpacked package's `usr/`-prefixed content into it (package archives are unpacked as-is —
  `etc/`, `var/`, etc. included — so this stage filters to the `/usr`-relevant part only), inserts
  `PackageMeta`/`FileEntry` rows into the embedded `redb` DB via the already-existing
  `MetaStoreMut`/`FileStoreMut`, and `commit_tree`s the result. It also separately imports each
  package's `etc/`-prefixed content into its own in-memory tree (never committed) — this is
  `config::merge::merge_config`'s future `new` input, since the package-shipped `/etc` defaults
  live in the very same unpacked directory, just a different top-level prefix. Both the new
  prefix digest and that `/etc`-defaults tree are stashed in `Context`
  (`installer::{NewPrefixDigest, NewConfigDefaults}`).
- `install`'s `MergeStage` is now real too: builds `base` (the current record's sealed
  `working_config` tree) and `live` (`base` + the current deploy's on-disk `etc-upper/upper`
  applied via `composefs::overlay::apply_overlay_upper`), runs `config::merge::merge_config(base,
  new, live)` against `TransactionStage`'s `NewConfigDefaults`, commits the merged tree as a new
  config-digest, then either updates or **creates** the `DeployRecord` for the freshly-committed
  `/usr` digest — a real gap this surfaced: unlike `rollback`'s `MergeStage` (which only ever
  *selects* an already-existing record), install's target `/usr` digest is brand new, so there was
  no existing `state/deploy/<digest>/meta.json` to read. New brick: `DeployRecord::allocate_seq`
  (simple read/increment/write of a new `state/next-seq` counter file, per doc §5.7 — had no
  implementation anywhere before this). Conflict `.upac-new` notification via the message-hook
  mechanism (per §5.1) is **not** wired yet — deferred, same class of gap as `up mime sync`'s
  best-effort cache refresh.
- `install`'s own `CheckoutStage`/`SwapStage` are now real too, same shape as `rollback`'s: writes
  the ESP boot entry for the newly-committed `NewPrefixDigest` via `boot::write_boot_entry`,
  resolves the requested (or autodetected) boot plugin via `plugin::boot::resolve_boot_plugin`, and
  selects it for one-shot boot. `install` is now the first command with its **entire** pipeline
  real end to end (Transaction+Merge+Checkout+Swap), no `todo!()` left in it.
- `uninstaller` is now fully real too, end to end (`Preparation` → `Transaction` → `ConfigMerge` →
  `PrepareBoot` → `BootOption`, no `todo!()` left). `UninstallStateId` originally had a `Build`/
  `Commit` split inherited from an old, undocumented rename (git-archaeology found it was a
  by-product of an unrelated repo-restructuring commit, not a deliberate design) — collapsed back
  into a single `Transaction` stage to match install/update/files' shape.
  Building this surfaced a real, pre-existing gap: `install`'s `TransactionStage` never recorded
  which `/etc` files belong to which package in the DB (only `/usr` paths got `insert_package_file`
  calls), even though `database::attribution::FileAttribute` (used by `diff_config`) already
  assumed that tracking existed. Fixed via a new `upac_types::FileEntryScope { Prefix, Config }`
  field on `FileEntry` (the `#[derive(RedbCodec)]` macro needed zero changes — it already falls
  back to calling a field type's own `encode_into`/`decode_from` generically), plus updating
  `TransactionStage` to insert `FileEntryScope::Config` rows for the `/etc`-imported paths it was
  previously discarding.
  This in turn required generalizing `config::merge::merge_config` itself: the existing algorithm
  assumed `new` always still has whatever the user edited (fine for install, where packages only
  ever add/modify defaults), but uninstall's `new` (`base` minus the removed package's own config
  paths) can lack a path entirely. Fixed by keying the package-side diff by `FileDiffKind` (not
  just path presence) so "package no longer provides this path at all" is treated as *not* a
  conflict — the user's edit (or deletion) is simply carried forward, with no `.upac-new` sidecar
  synthesized against nothing. Two new tests cover this (`user_edit_survives_when_the_package_stops
  _providing_the_file`, `user_deletion_is_not_a_conflict_when_the_package_also_removed_the_file`);
  all pre-existing `merge_config` tests still pass unchanged.
  `update`/`files` still have `todo!()` `TransactionStage`/`MergeStage`/`CheckoutStage`/`SwapStage`
  bodies, though they should look very close to install's now that it's a full template. One test
  (`opaque_directory_drops_base_subtree_entirely`) is `#[ignore]`d — setting the
  `trusted.overlay.opaque` xattr needs `CAP_SYS_ADMIN`/root, unavailable in normal test runs.
- `up gc`'s `CleaningStage` is now real: composefs 0.9.0's own `Repository::gc(additional_roots)`
  already walks `objects/`/`streams/` and sweeps everything unreachable — no hand-rolled
  `ObjectCollector` needed, just a thin `composefs::repository::gc` wrapper. Since none of our
  images are registered under `images/refs/` (every commit is anonymous, addressed by digest), the
  stage itself enumerates every currently-existing `state/deploy/<digest>/` (via `Deploy::deploys`)
  and passes each one's `prefix_digest` + `working_config` + every `config_history` entry's
  `config_digest` as `additional_roots` — anything not reachable from that set gets swept. Deploy
  *retention* (deciding which `state/deploy/<digest>/` directories should exist in the first place,
  §5.5 point 1, "light cleanup after every operation") is a separate, still-unbuilt mechanism —
  `gc` only sweeps objects given whatever deploys currently happen to be on disk.
- Decoder static linking (Zig, separate mechanism from the Rust boot plugins' Cargo-feature
  approach) — not started, see `ROADMAP.md` §1.
