# TODO

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `file remove`, `file diff`, `file search` (`user/upac-cli/src/commands/file/`) are still
  `todo!()` stubs. ABI-side they map to `files` (`FileDiffKind::Removed`), `diff_prefix`, and
  `search_files` respectively — the `Args` structs exist, only `run()` bodies are missing.
- ABI capabilities with no CLI surface at all yet (not a rename — genuinely new subcommands, so
  deliberately not added without a design decision first): `diff` (combined packages+prefix-files),
  `diff_config`, `search_in_meta`, `search_in_package_files`.

## upac-lib

- The entire mutated-command pipeline body (composefs mount/merge/checkout/swap) is still
  `todo!()` across every mutating command — see `ROADMAP.md`.
