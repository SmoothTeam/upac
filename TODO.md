<h1 align="center">✅ TODO</h1> 

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

## upac-lib

- No tests for `errors.rs` (`CommonError` → `ErrorKind`), `lock.rs` (unix-socket-bind exclusive
  lock), `search.rs` (`Search::new`), `database/attribution.rs` (`FileAttribute::attribute_file`),
  `database/{files,meta,triggers}.rs` (check whether `database_record.rs`'s existing fixture
  already exercises them first), `plugin/decoder/error.rs`, `plugin/decoder/unpack.rs`
  (`unpack_one` against a small real archive). None need real hardware/mount.

