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
  (`installer::{NewPrefixDigest, NewEtcDefaults}`).
- `install`'s `MergeStage` is now real too: builds `base` (the current record's sealed
  `working_config` tree) and `live` (`base` + the current deploy's on-disk `etc-upper/upper`
  applied via `composefs::overlay::apply_overlay_upper`), runs `config::merge::merge_config(base,
  new, live)` against `TransactionStage`'s `NewEtcDefaults`, commits the merged tree as a new
  etc-digest, then either updates or **creates** the `DeployRecord` for the freshly-committed
  `/usr` digest — a real gap this surfaced: unlike `rollback`'s `MergeStage` (which only ever
  *selects* an already-existing record), install's target `/usr` digest is brand new, so there was
  no existing `state/deploy/<digest>/meta.json` to read. New brick: `DeployRecord::allocate_seq`
  (simple read/increment/write of a new `state/next-seq` counter file, per doc §5.7 — had no
  implementation anywhere before this). Conflict `.upac-new` notification via the message-hook
  mechanism (per §5.1) is **not** wired yet — deferred, same class of gap as `up mime sync`'s
  best-effort cache refresh.
  `update`/`files` still have `todo!()` `TransactionStage`/`MergeStage`/`CheckoutStage`/`SwapStage`
  bodies (though `update`'s should look very close to install's, now that both exist as a
  template), install's own `CheckoutStage`/`SwapStage` are still `todo!()` (should look close to
  rollback's, now that both boot bricks are proven working), and so are `uninstaller`'s
  `PrepareBootStage`/`BootOptionStage`. One test
  (`opaque_directory_drops_base_subtree_entirely`) is `#[ignore]`d — setting the
  `trusted.overlay.opaque` xattr needs `CAP_SYS_ADMIN`/root, unavailable in normal test runs.
- Decoder static linking (Zig, separate mechanism from the Rust boot plugins' Cargo-feature
  approach) — not started, see `ROADMAP.md` §1.
