# TODO

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

- Work out a scheme for explicitly binding the mime type to the backend and updating the cli mime type dynamically.

## upac-lib

- The entire mutated-command pipeline body (composefs mount/merge/checkout/swap) is still
  `todo!()` across every mutating command — see `ROADMAP.md`. Some write-side building blocks now
  exist ahead of the stage bodies themselves: `FileHandle::import_directory`/
  `composefs::repository::commit_tree` (composefs image write side), `boot::write_boot_entry` +
  `boot::OneShotReboot`/`Uki`/`Bls` (ESP entry + one-shot NVRAM selection), and an explicit
  `boot_kind: BootKind` field threaded through the C-ABI/CLI (`--boot auto|uki|bls`) — none of it
  consumed yet, since `CheckoutStage`/`SwapStage`/`PrepareBoot`/`BootOption` are still `todo!()`.
  `config_merge` (§5.1, 3-way `/etc` merge) has no code and no bricks yet — next big piece.
