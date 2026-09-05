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

Genesis (`up-sp`) now installs the actual bootloader binary onto a fresh ESP for systemd-boot and
rEFInd (`StageBootStage::run`, `lib/setup/lib.toml`'s `[genesis]` source paths) — confirmed working
via a live VM test for systemd-boot; rEFInd wired the same way but not yet VM-verified. grub is NOT
handled — a real `grub-install`-equivalent (target-specific generated `grubx64.efi`, not a plain
file copy) is out of scope for now; either shell out to `grub-install` against the mounted ESP, or
explicitly document grub as unsupported for genesis whole-disk mode.

`StageBootStage` picks which ESP loader binary to copy via a `match input.boot_plugin.as_deref()`
against literal `"systemd-boot"`/`"refind"` strings — this bypasses the actual dynamic boot-plugin
system (`resolve_boot_plugin`/`BootPluginManifest`/`static_plugins`), which is supposed to be the
one place plugin names are known. Adding a 5th booter plugin would require editing this match by
hand instead of just dropping in a new plugin. The correct fix is extending the `Booter` ABI itself
with a 4th function (e.g. `esp_loader_source() -> CSlice`, empty for uki/grub) so genesis asks the
already-resolved plugin for its own install-time source path instead of hardcoding names — but that
means bumping `BOOT_ABI_VERSION` and touching all 4 `booters/*` crates, so deliberately deferred;
the hardcoded match stays as a known, scoped limitation until then.

**Genesis-produced disks don't actually boot into the installed system yet** — found via a live
QEMU/OVMF test (systemd-boot now starts, finds the BLS entry, loads kernel+initramfs — that part
works after the bootloader-binary fix above). Two separate gaps, both required:
1. `partition.rs`'s `LINUX_PARTITION_TYPE_GUID` (`0fc63daf-8483-4772-8e79-3d69d8477de4`, generic
   "Linux filesystem data") should be the discoverable-root GUID
   (`4f68bce3-e8cd-4db1-96e7-fbcaf984b709`, "Linux root x86-64") so `systemd-gpt-auto-generator`
   can find the deploy partition at all instead of hanging on `/dev/gpt-auto-root`.
2. Even with (1) fixed, a plain partition mount isn't how composefs systems boot — nothing in this
   project resolves `composefs.digest=<hash>` (the kernel cmdline param `write_boot_entry` already
   writes) against the on-disk repository, mounts the erofs image with fs-verity, and overlays
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

**Genesis tracks the entire bootstrapped system as a single synthetic "rootfs" package**, not
per-package (`ReadMetaStage` reads one `meta.toml`, `ImportTreesStage` imports all of source's
`usr`/`etc` wholesale). Found while reasoning about the `composefs-setup-root` hook: if it needs to
already be installed on the source system (via pacman) for genesis to pick it up, its files still
end up attributed to the one fake "rootfs" package in our database — no real per-package
provenance for anything baked into the source image, unlike a `pacstrap`-then-`up install` flow
would give. Decision made: genesis should eventually be rewritten to install real, individually
decoded packages through the same pipeline `up install` uses, instead of importing a pre-built
directory wholesale — no special-casing even for the kernel package. This is a genesis rewrite, not
a patch; deliberately deferred until after a dedicated code-cleanup/macro-consolidation pass
(reduce duplicated lines, extract shared macros) elsewhere in the codebase first.

