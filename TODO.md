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
- `install`/`update`/`files` still have `todo!()` `TransactionStage`/`MergeStage`/`CheckoutStage`/
  `SwapStage` bodies, and `uninstaller`'s `PrepareBootStage`/`BootOptionStage` are `todo!()` too —
  none of them yet call the write-side bricks (`FileHandle::import_directory`/
  `composefs::repository::commit_tree`, `boot::write_boot_entry`, `resolve_boot_plugin`) or
  `config::merge::merge_config` (§5.1, built as a pure algorithm brick — base/new/live
  classification + conflict `.upac-new` sidecars). Acquiring `merge_config`'s `live` input tree now
  has its own brick too — `composefs::overlay::apply_overlay_upper` imports the on-disk
  `etc-upper/upper` OverlayFS overlay onto an already-populated tree, correctly handling whiteouts
  (deletions) and opaque directories (full subtree replacement) — but nothing calls it from a real
  mutating command yet, and `new` still needs `TransactionStage`, itself still `todo!()`. One test
  (`opaque_directory_drops_base_subtree_entirely`) is `#[ignore]`d — setting the
  `trusted.overlay.opaque` xattr needs `CAP_SYS_ADMIN`/root, unavailable in normal test runs.
- Decoder static linking (Zig, separate mechanism from the Rust boot plugins' Cargo-feature
  approach) — not started, see `ROADMAP.md` §1.
