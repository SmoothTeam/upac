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

- `FetchingStage` (`install`/`update`) is a real no-op placeholder, not `todo!()` — the network
  side (resolving name-based package requests, as opposed to local `--file` paths) isn't designed
  or built yet.
- `files add`/`files remove` only ever target `/usr`-scope (`is_user` `FileEntry` rows) — there's no
  ABI/CLI path for attaching a user file to a package's `/etc` scope, and it's architecturally
  different if it's ever added (config-digest based, not `FileStoreMut`-tracked).
- `upac-cli` never registers a message hook at all — `request_base()`
  (`user/upac-cli/src/types/abi.rs:20-22`) hardcodes `on_hook: None` for every single command.
  This means `ProgressEventBuilder` events (stage/phase/subject/current/total — already generated
  by stage bodies) and any `MessageHook` confirmation (including the `.upac-new` conflict
  notification from doc §5.1) currently go nowhere: the CLI shows no progress output and can't
  respond to any hook-driven prompt during a long-running command. Needs a real `on_hook`
  implementation in `upac-cli` (progress bar/line + confirmation prompt), not just the `None`
  placeholder.
- Cancellation (`Ctrl+C` → `CancelToken` → orchestrator) is wired and checked *between* stages
  (`lib/lib/src/orchestrator/cursor.rs:43-45`), but no individual stage body's own internal loop
  (e.g. copying many files in a `TransactionStage`) checks `cancel.is_cancelled()` mid-loop — needs
  an audit of every stage with a per-item loop to decide where a finer-grained check is actually
  worth adding, versus stage-boundary granularity being good enough.
- Deploy retention / "light cleanup after every operation" (doc §5.5 point 1 — deciding which
  `state/deploy/<digest>/` directories should be pruned) has no code anywhere yet; `up gc` only
  sweeps objects given whatever deploys currently happen to exist on disk.
- Decoder static linking (Zig, separate mechanism from the Rust boot plugins' Cargo-feature
  approach) — not started, see `ROADMAP.md` §1.
- `Version` now has a real rpmvercmp-style `Ord`/`PartialOrd` (epoch first, then alternating
  numeric/alpha token comparison), but nothing consumes it yet: `update` doesn't check whether a
  `--file` package is actually newer than what's installed, and `pkg list`/`pkg search` don't sort
  by version. The 4 Zig decoders (`decoders/{alpm,deb,rpm,xbps}`) still populate `CVersion` via
  their own independent, partially-buggy per-segment parsers (deb hard-fails on a non-numeric
  dot segment, e.g. `"1.2.3-alpha"`) — `CVersion`'s Rust/C-ABI shape was simplified to
  `{epoch, raw}` but the Zig-side parsing itself is untouched, deliberately out of scope for now.
