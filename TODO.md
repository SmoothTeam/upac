<h1 align="center">✅ TODO</h1> 

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

## setup-cli

- `up-sp`'s localization is currently broken in bootstrap/rescue environments (archiso etc.): `gettext-rs`
  relies on glibc's `setlocale`/`LC_MESSAGES` resolution, which falls back to the "C" locale and skips
  translation lookup entirely when no real locale is installed (confirmed via live VM testing — even
  forcing `C.utf8` didn't help, since that minimal locale's `LC_MESSAGES` component is still effectively
  "C"). Needs a real decision: either the pure-Rust `gettext` crate (parses `.mo` directly, no system
  locale dependency, but reimplements its own plural-forms handling) or Mozilla's Fluent (`fluent`/
  `fluent-rs`, a different file format/model entirely, `.ftl` not `.po`/`.mo`) before re-attempting the fix.

