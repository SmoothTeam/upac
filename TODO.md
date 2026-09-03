<h1 align="center">✅ TODO</h1> 

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

## upac-lib

Test-coverage pass in progress, going file by file through the non-command core first
(`errors.rs`/`lock.rs`/`search.rs`/`fs.rs`/`orchestrator/*`/`database/*` done), commands
(`mutated`/`unmutated`) last. Remaining core files not yet visited: `deploy/{error,retention,mod}.rs`
(`esp.rs` skipped — real mount), `scripts/{error,file,load,pipeline,primitive}.rs`,
`plugin/decoder/{error,unpack,mod}.rs`, `plugin/boot/{error,manifest,mod}.rs`,
`composefs/{diff,error,mod}.rs`, `config/mod.rs`, `boot/{error,mod}.rs`.

