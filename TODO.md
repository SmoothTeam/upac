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

`boot/mod.rs`'s UKI staging only ever writes the single fixed `upac-to` slot
(`layout::boot::UPAC_TO_SLOT`) — doc chapter 3's disk-layout map still describes a two-slot
`upac-from.efi`/`upac-to.efi` A/B scheme, but no code anywhere writes/reads an `upac-from` slot.
Real unfinished A/B swap, not just a stale doc — needs a decision (implement the second slot, or
formally drop it and fix the doc to match the single-slot design `lib.toml`'s own comment already
argues for).

**Genesis-produced disks don't actually boot into the installed system yet** — found via a live
QEMU/OVMF test (systemd-boot now starts, finds the BLS entry, loads kernel+initramfs):
1. Still open: a plain partition mount isn't how composefs systems boot — nothing in this project
   resolves `composefs.digest=<hash>` (the kernel cmdline param `write_boot_entry` already writes)
   against the on-disk repository, mounts the erofs image with fs-verity, and overlays
   `state/deploy/<digest>/etc/`. **Found a real, existing upstream tool for exactly this**:
   `composefs-setup-root` (crates.io, same `composefs-rs` project/version as our `composefs`/
   `composefs-boot` deps) — a Rust binary, not something we'd write ourselves. Our on-disk layout
   already matches its hardcoded expectations (`composefs/`, `state/deploy/<digest>/`) after
   renaming `etc-upper` → `etc` (done, `lib.toml`'s `config_dir_name`). What's still missing: the
   actual boot-time integration — the live VM's initramfs is systemd-based (mkinitcpio's `systemd`
   hook, not classic busybox-style hooks), so this needs a systemd unit ordered between
   `sysroot.mount` and `initrd-switch-root.target` (same role as ostree's
   `ostree-prepare-root.service`), not a classic mkinitcpio hook script. Also unresolved: whether
   upac needs to ship/package this integration itself, or whether it's expected to already exist
   on the source distro (same assumption as the systemd-boot/rEFInd binary copy above) — needs
   checking whether Arch/AUR already has a package for this.
