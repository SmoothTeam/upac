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

- `files` is the only command left with `todo!()` stage bodies (`TransactionStage`/`CheckoutStage`/
  `SwapStage` — no `merge.rs` exists yet either). Every other command
  (`install`/`update`/`uninstaller`/`rollback`/`commit`/`gc`) is fully real now. Should closely
  follow `install`'s/`update`'s template.
- `FetchingStage` (`install`/`update`) is a real no-op placeholder, not `todo!()` — the network
  side (resolving name-based package requests, as opposed to local `--file` paths) isn't designed
  or built yet.
- Conflict `.upac-new` notification via the message-hook mechanism (doc §5.1) isn't wired for any
  command yet — same class of gap as `up mime sync`'s best-effort cache refresh.
- Deploy retention / "light cleanup after every operation" (doc §5.5 point 1 — deciding which
  `state/deploy/<digest>/` directories should be pruned) has no code anywhere yet; `up gc` only
  sweeps objects given whatever deploys currently happen to exist on disk.
- One test (`opaque_directory_drops_base_subtree_entirely`, `composefs_overlay.rs`) is `#[ignore]`d
  — setting the `trusted.overlay.opaque` xattr needs `CAP_SYS_ADMIN`/root, unavailable in normal
  test runs.
- Decoder static linking (Zig, separate mechanism from the Rust boot plugins' Cargo-feature
  approach) — not started, see `ROADMAP.md` §1.
