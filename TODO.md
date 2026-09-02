<h1 align="center">✅ TODO</h1> 

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.
- `src/types/tests/progress.rs` looks broken: it asserts `state.bar.message() == "stage_pre_hooks"`
  (underscore) without calling `locale::init()`; `StageKey` generates kebab-case
  (`"stage-pre-hooks"`), and with no bundle loaded `LOADER.get()` returns
  `"No localization for id: \"...\""` — neither matches. Needs a real fix pass.
- No tests for `types/abi.rs` (`empty_slice`/`slice_from_cstr`/`optional_slice`) or
  `types/errors.rs` (`AbiMismatch`/`StageName`/`LibError` `Display`) — both pure, untested.
- Check `commands/package/search.rs`, `commands/file/search.rs`, `commands/package/remove.rs` for
  extractable/testable validation logic (each has `bail!` branches similar to `up-sp`'s
  `whole_disk.rs`, already covered there).

## upac-lib

- No tests for `errors.rs` (`CommonError` → `ErrorKind`), `lock.rs` (unix-socket-bind exclusive
  lock), `search.rs` (`Search::new`), `database/attribution.rs` (`FileAttribute::attribute_file`),
  `database/{files,meta,triggers}.rs` (check whether `database_record.rs`'s existing fixture
  already exercises them first), `plugin/decoder/error.rs`, `plugin/decoder/unpack.rs`
  (`unpack_one` against a small real archive). None need real hardware/mount.

## sign-cli

- Zero test coverage despite `tempfile` already sitting unused in `[dev-dependencies]`. Unlike
  `up-sp`, `generate_root`/`generate_cert`/`sign_hook`/`verify_hook` are pure crypto + local file
  I/O (no hardware), so a real round-trip test (root → leaf cert → sign a hook → verify) is
  feasible, plus a `Display` test for `errors.rs`'s `LocalizedPkiError` (needs the same
  `init_for_test()` locale helper `up-sp`/`up` use).
