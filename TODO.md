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

- The entire mutated-command pipeline body (composefs mount/merge/checkout/swap) is still
  `todo!()` across every mutating command — see `ROADMAP.md`. Some write-side building blocks now
  exist ahead of the stage bodies themselves: `FileHandle::import_directory`/
  `composefs::repository::commit_tree` (composefs image write side), `boot::write_boot_entry`
  (ESP entry write, boot-kind-agnostic), and a full boot-plugin subsystem for one-shot NVRAM/
  bootloader-config selection (`upac_abi::boot::Booter` trait + C-ABI contract,
  `plugin::boot::resolve_boot_plugin` loader/resolver, four working plugin crates under
  `booters/`: `uki`, `systemd-boot`, `grub`, `refind`). None of this is consumed by a stage
  body yet, since `CheckoutStage`/`SwapStage`/`PrepareBoot`/`BootOption` are still `todo!()`.
  `config::merge::merge_config` (§5.1, 3-way `/etc` merge) now exists as a pure algorithm brick
  (base/new/live classification + conflict `.upac-new` sidecars, built on the existing
  `composefs::diff::TreeDiff` + a new `FileHandle::copy_from_tree`), but nothing acquires its three
  input trees yet (`live` needs importing the on-disk `etc-upper/upper` overlay with whiteout
  handling — not built; `new` needs `TransactionStage`, itself still `todo!()`) and no `MergeStage`
  body calls it yet.
- Decoder static linking (Zig, separate mechanism from the Rust boot plugins' Cargo-feature
  approach) — not started, see `ROADMAP.md` §1.
