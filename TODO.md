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
  `composefs::repository::commit_tree` (composefs image write side), `boot::write_boot_entry` +
  `boot::OneShotReboot`/`Uki`/`Bls` (ESP entry + one-shot NVRAM selection), and an explicit
  `boot_kind: BootKind` field threaded through the C-ABI/CLI (`--boot auto|uki|bls`) — none of it
  consumed yet, since `CheckoutStage`/`SwapStage`/`PrepareBoot`/`BootOption` are still `todo!()`.
  `config_merge` (§5.1, 3-way `/etc` merge) has no code and no bricks yet — next big piece.
