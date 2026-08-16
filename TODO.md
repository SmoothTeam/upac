# TODO

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- **Bug**: `commit list` (`user/upac-cli/src/commands/commit/list.rs`) calls `list_history` and
  prints each entry's `prefix_digest`. But `commit rollback` expects a **`config_digest`**
  (`CRollbackRequest.config_digest`) — a different identifier space (deploy/prefix vs. config
  commit). Copying the digest `commit list` prints into `commit rollback` won't resolve. Fix:
  switch `commit list` to the `list_config` export (`CListConfigRequest` → `Vec<CConfigCommitEntry>`,
  which already carries `config_digest` directly).
- `file remove`, `file diff`, `file search` (`user/upac-cli/src/commands/file/`) are still
  `todo!()` stubs. ABI-side they map to `files` (`FileDiffKind::Removed`), `diff_prefix`, and
  `search_files` respectively — the `Args` structs exist, only `run()` bodies are missing.
- `display/package.rs`: `License`/`Url` field formatting still chains
  `.unwrap_or_default().unwrap_or_default()`. A cleanup (small `optional_str(&CSlice) -> &str`
  helper) was started but never applied — sitting in a local `git stash` entry
  (`wip: optional_str cleanup + stray upac-macro dep`), not on any branch.
- `gc` — `RwSymbols.gc`/`CGcRequest` are loaded and gated behind `Lib::require_write()`, but no
  CLI subcommand calls it yet.
- ABI capabilities with no CLI surface at all yet (not a rename — genuinely new subcommands, so
  deliberately not added without a design decision first): `list_prefix`, `diff` (combined
  packages+prefix-files), `diff_config`, `search_in_meta`, `search_in_package_files`.

## upac-lib

- The entire mutated-command pipeline body (composefs mount/merge/checkout/swap) is still
  `todo!()` across every mutating command — see `ROADMAP.md`.
