<h1 align="center">✅ TODO</h1> 

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

## upac-lib

- Decide whether `Stage::run()` should drop its `cancel: &CancelToken` parameter entirely
  (cancellation checked only by the orchestrator's `Cursor`, between stage/repeat invocations,
  never inside a stage body) or split into two `run` signatures — one for mutating stages that
  may still need direct I/O-level cancellation, one for unmutated stages that don't. Revisit once
  the for-loop/Repeat audit has also covered the unmutated pipelines, not just the mutating ones.

## setup-cli

- It is necessary to audit the stages, break down the cycles within them into orchestrator cycles, and increase granularity across all libraries.
- Add a domain-based error system to the setup library.
