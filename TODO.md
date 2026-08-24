# TODO

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

## upac-lib

- The `MessageHook`/`on_hook` mechanism (progress events) is real and wired for the 5 commands
  with genuine per-item progress (`install`/`update`/`remove`(uninstall)/`file add`/`file remove`
  — `types/progress.rs`'s `ProgressState` + indicatif bar). It is still purely one-directional
  (lib → caller, ack is only "delivered"/"retry") — there is still no way for the caller to answer
  a question and have lib block on the response. The `.upac-new` conflict notification from doc
  §5.1 needs exactly that (a real yes/no decision, not a fire-and-forget event), so it needs a
  genuinely new, separate two-way mechanism, not just another event on the existing hook.
