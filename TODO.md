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

Genesis (`up-sp`) never installs the actual bootloader binary (`systemd-bootx64.efi`) onto a fresh
ESP or registers an EFI NVRAM Boot#### entry — `StageBootStage`/`write_boot_entry`/
`resolve_boot_plugin().set_one_shot()` only ever manage an *existing* boot chain (works for
install/update on an already-installed system, since the bootloader is already on the ESP from a
prior run). On a brand-new disk this leaves the VM/machine with nothing for firmware to execute at
boot, even though every other genesis artifact (composefs repo, deploy record, BLS entry) is
written correctly — confirmed via a live VM test. Needs a new genesis step that copies the
bootloader binary from the source tree onto the ESP (`/EFI/BOOT/BOOTX64.EFI` and/or
`/EFI/systemd/systemd-bootx64.efi`) and/or registers the NVRAM entry.

