<h1 align="center">✅ TODO</h1> 

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

## upac-lib

- `uninstaller` has no `HookStage::declarative(Operation::Uninstall)` pass — declarative (package-format-native) triggers only ever get produced by `decode()` during install/update's `PreparationStage`, and uninstall never decodes anything (it only knows the already-installed package's UUID/name), so there's currently no data source for them there. Making this work for real needs `declarative_triggers` persisted into the package database at install/update time (alongside `PackageMeta`) and read back from there during uninstall, instead of re-derived from a fresh `decode()` call.
