<h1 align="center">✅ TODO</h1> 

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

## upac-setup

- `lib/setup/src/format.rs`: `mkfs.ext4`/`mkfs.xfs` formatting (shell-out) — only `format_btrfs`
  (native, via `btrfs-mkfs`) exists so far.
- Mode 1 (`CSetupWholeDiskRequest`) dispatcher: no function yet ties `DiskLayout::create` (GPT) +
  per-partition formatting (by `FsKind`) + `TargetSysroot` together — `SetupWholeDiskData` only
  parses the request, nothing consumes it yet.
- Genesis `run()` (mode 2, `CSetupExistingRequest`) still unwritten: import `/usr`+`/etc` trees →
  register "upac" package in DB → embed DB → `commit_tree` both → write `DeployRecord` +
  `state/.next-seq` → resolve boot plugin → `write_boot_entry` → `set_one_shot`.
- `up-sp` CLI (the `upac-setup` frontend) not started at all.
