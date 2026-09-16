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
ComposefsSetupRootUnitNotFound`, if missing), imports the whole tree into `PrefixTree`, and creates
the unit's `*.target.wants/` enablement symlink itself. This is also how a built `up`/`upac-lib`/
booters gets onto a genesis'd disk at all — genesis never installs itself automatically, whoever
assembles `--source` has to place it under `system/` too, same assumption already made for the
systemd-boot/rEFInd binaries. Confirmed `composefs-setup-root`'s own hardcoded expectations already
match upac's on-disk layout exactly (repo at `composefs/`, per-deploy state at `state/deploy/<hex>/`,
`composefs=<hex>` cmdline karg) — no restructuring was needed, only the unit + the `system/` plumbing.
Still unresolved: whether upac ships/packages the `composefs-setup-root` binary itself or expects it
to already exist on the source distro (same open question as the systemd-boot/rEFInd binaries).

## upac-setup

`KernelStage` (`lib/setup/src/stages/kernel.rs`) is still a no-op stub — needs to actually run
mkinitcpio/dracut against a scratch directory and import the resulting kernel/initramfs into the
prefix tree.

**Composefs initramfs hook does not exist anywhere in the project** — a dracut module or
mkinitcpio hook that runs inside the generated initramfs on the target machine at boot: resolve
`composefs.digest=` from the kernel cmdline, mount the erofs image with fs-verity, overlay
`state/deploy/<digest>/etc/` on top. Without it, a genesis'd disk cannot boot into the installed
system even once `KernelStage` above produces a real initramfs — confirmed via a live QEMU/VM boot
test, not just reading the code (see `ROADMAP.md` §5). Tracked here as a new, unstarted,
non-trivial subsystem.
