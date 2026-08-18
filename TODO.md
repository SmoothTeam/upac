# TODO

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

- Decoder manifests don't exist yet for the 4 Zig decoders (`decoders/{alpm,deb,rpm,xbps}/`) — no
  `*.toml` manifest file, no install step, so `DecoderManifest`'s now-required `mime` field (see
  upac-lib below) has nothing to read from a real distro decoder yet. `up mime sync` works
  correctly against zero manifests (writes an empty-but-valid `upac-mime.xml`), so this isn't
  blocking, just unexercised end-to-end.

## upac-lib

- The entire mutated-command pipeline body (composefs mount/merge/checkout/swap) is still
  `todo!()` across every mutating command — see `ROADMAP.md`. Some write-side building blocks now
  exist ahead of the stage bodies themselves: `FileHandle::import_directory`/
  `composefs::repository::commit_tree` (composefs image write side), `boot::write_boot_entry` +
  `boot::OneShotReboot`/`Uki`/`Bls` (ESP entry + one-shot NVRAM selection), and an explicit
  `boot_kind: BootKind` field threaded through the C-ABI/CLI (`--boot auto|uki|bls`) — none of it
  consumed yet, since `CheckoutStage`/`SwapStage`/`PrepareBoot`/`BootOption` are still `todo!()`.
  `config_merge` (§5.1, 3-way `/etc` merge) has no code and no bricks yet — next big piece.
