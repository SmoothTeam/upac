<h1 align="center">✅ TODO</h1> 

Near-term, concrete items. See `ROADMAP.md` for the bigger picture.

## upac-cli

- `user/upac-cli/data/` (`.desktop`, `upac-mime.xml`, `.policy`) reference `Icon=upac`/`icon_name=upac`,
  but there's no actual icon asset (SVG/PNG) yet, and no install step wiring it into
  `/usr/share/icons/hicolor/...`. Needs real artwork before packaging.

## upac-lib

Test-coverage pass in progress. The entire non-command core is covered (`errors.rs`/`lock.rs`/
`search.rs`/`fs.rs`/`orchestrator/*`/`database/*`/`deploy/*`/`scripts/*`/`composefs/*`/`config/*`/
`boot/*`/`plugin/decoder/{error,manifest,triggers}.rs`/`plugin/boot/{error,manifest}.rs`), except
`plugin/decoder/unpack.rs`/`plugin/decoder/mod.rs`/`plugin/boot/mod.rs` (need a real dlopen'd/
`builtin-*` plugin) and `deploy/esp.rs` (real mount table) — both explicit, justified skips. Every
`mutated`/`unmutated` command's own `<Command>Error` enum is also now covered (inline tests next to
each `error.rs`, since `mutated`/`unmutated` aren't `pub`) — only each variant's own logic, not the
macro-generated `Common(...)` delegation shared with `errors.rs`'s already-tested `CommonError`.
Remaining: the `Stage::run()` bodies themselves — each needs a real composefs `Repository`/`Deploy`/
database in context, likely out of scope for unit tests unless a pure-logic helper turns out to be
extractable.

**`genesis`'s `system/` mechanism is done**: `ImportSystemStage` requires `<source>/system/` (a
literal 1:1 mirror of the target's real `/usr`, sibling to the package archives —
`EnumeratePackagesStage` already skips it, it only looks at files) to contain
`lib/systemd/system/composefs-setup-root.service` (hard error, `SetupError::
ComposefsSetupRootUnitNotFound`, if missing) and imports the whole tree into `PrefixTree`. This is
also how a built `up`/`upac-lib`/booters gets onto a genesis'd disk at all — genesis never installs
itself automatically, whoever assembles `--source` has to place it under `system/` too, same
assumption already made for the systemd-boot/rEFInd binaries. Confirmed `composefs-setup-root`'s own
hardcoded expectations already match upac's on-disk layout exactly (repo at `composefs/`, per-deploy
state at `state/deploy/<hex>/`, `composefs=<hex>` cmdline karg) — no restructuring was needed, only
the unit + the `system/` plumbing. The unit's `*.target.wants/` enablement is deliberately NOT
created by this stage (a symlink to `initrd-root-fs.target.wants/` in the real root tree is a no-op
— that target only exists inside the initrd's own systemd instance) — it's created instead by the
dracut module at `hooks/dracut/37composefs/` at initrd-build time.
**Decided: upac packages/vendors `composefs-setup-root` itself** (same call for the systemd-boot/
rEFInd binaries) rather than assuming the source distro already provides it — genesis-time import
should also check whether one already exists under `system/` rather than blindly trusting our own
copy is the only source. Not yet implemented.

## upac-setup

`KernelStage`'s mkinitcpio path (`lib/setup/src/stages/kernel.rs`) needs rechecking — it only
redirects `/lib/modules` via `-r <scratch>/lib/modules`, and there's no confirmed mkinitcpio
equivalent of dracut's full `--sysroot` (which redirects everything: hooks, config, binaries).
Unlike dracut, mkinitcpio may still fall through to the *real* host's `/etc/mkinitcpio.conf`/`/usr`
instead of the scratch tree genesis built. Needs verifying against a real mkinitcpio run before
trusting the generated initramfs for the `mkinitcpio` generator choice.

`KernelStage`'s `run_dracut`/`run_mkinitcpio` (`lib/setup/src/stages/kernel.rs`) currently take a
plain `is_uki: bool` and branch internally (`--uefi`/`-U` vs the plain-initramfs flags). Once UKI
signing or a separate UKI-specific generation path is added, this needs splitting into distinct
`run_<tool>`/`run_<tool>_with_uki` functions instead of a bool flag, so the two concerns (plain
initramfs vs UKI build+sign) don't stay tangled inside one function.
