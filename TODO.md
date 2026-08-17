# TODO

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

- Add flags to explicitly specify boot (UKI/Manager).

- Work out a scheme for explicitly binding the mime type to the backend and updating the cli mime type dynamically.

## upac-lib

- Add explicit flag passing to boot in FFI.

- Determine how best to import errors: everything in the file header via as to understand the context and prohibit the use of :: in From constructs, or do it from exactly where:: error type.

- The entire mutated-command pipeline body (composefs mount/merge/checkout/swap) is still
  `todo!()` across every mutating command — see `ROADMAP.md`.
