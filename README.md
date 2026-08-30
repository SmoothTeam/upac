<h1 align="center">📦 Upac</h1>
<p align="center"><em>One atomic deploy. Any package format. One rollback away.</em></p>
<p align="center"><strong>A modular package management for Linux systems with composefs-based atomic deploys.</strong></p>

**Links for repositories:** [![GitHub](https://img.shields.io/badge/GitHub-SmoothTeam%2Fupac-181717?logo=github)](https://github.com/SmoothTeam/upac) [![Codeberg](https://img.shields.io/badge/Codeberg-justpav05%2Fupac-2185D0?logo=codeberg)](https://codeberg.org/justpav05/upac)

**General information:** [![Version](https://img.shields.io/badge/version-0.1.5-green)](https://github.com/SmoothTeam/upac/releases) [![REUSE](https://github.com/SmoothTeam/upac/actions/workflows/reuse.yml/badge.svg?branch=main)](https://github.com/SmoothTeam/upac/actions/workflows/reuse.yml)

**Licensing:** [![lib: LGPL-3.0-or-later](https://img.shields.io/badge/lib-LGPL--3.0--or--later-light_green.svg)](LICENSES/LGPL-3.0-or-later.txt) [![cli: GPL-3.0-only](https://img.shields.io/badge/cli-GPL--3.0--only-light_green.svg)](LICENSES/GPL-3.0-only.txt) [![documentation: CC-BY-SA-4.0](https://img.shields.io/badge/documentation-CC--BY--SA--4.0-light_green.svg)](LICENSES/CC-BY-SA-4.0.txt)

> **⚠️ Active development is on [`lib-rs`](https://github.com/SmoothTeam/upac/tree/lib-rs).**
> That branch is a from-scratch rewrite of upac's core library in Rust, built around
> [composefs](https://github.com/containers/composefs) instead of OSTree. `main` is synced
> from it periodically and lags behind.



## 🔍 Overview

**Upac** is a package manager for Linux-compatible systems. It manages the updating, removal, and installation of various package formats using per-format decoders, and keeps every change behind an atomic, [composefs](https://github.com/containers/composefs)-backed deploy so the system can always be rolled back to a previous commit.

**upac-cli** (binary: `up`) is a command-line frontend written in [Rust](https://www.rust-lang.org/) that dynamically loads `libupac.so` and drives it through its C ABI.

**upac-lib** is the core library, also written in Rust, designed to be embedded into package managers through a stable C ABI. It handles installation, database management (via [`redb`](https://github.com/cberner/redb)), and composefs-based system snapshotting — without imposing any policy on how packages are fetched or what format they come in.

The library is intentionally split into independent components: decoders handle format-specific unpacking, the core library handles installation and database operations, and everything crosses the FFI boundary through a shared `upac-abi` crate.

It covers the disk layout, the deploy/rollback model, the `/etc` merge, GC, the FFI boundary, and the planned module structure.

## 📖 Design

Full architecture and design decisions are in the project design notes:

For english (canonical):
1. [`Introduction and definitions`](<doc/eng/Upac chapter 0.md>);
2. [`Problem statement`](<doc/eng/Upac chapter 1.md>);
3. [`Defining what the project is NOT (Non-goals)`](<doc/eng/Upac chapter 2.md>);
4. [`Disk structure`](<doc/eng/Upac chapter 3.md>);
5. [`Project repository structure`](<doc/eng/Upac chapter 4.md>);
6. [`Operating mechanisms`](<doc/eng/Upac chapter 5.md>);
7. [`FFI and the boundaries of interaction between the components`](<doc/eng/Upac chapter 6.md>);
8. [`Program modules`](<doc/eng/Upac chapter 7.md>).

For russian:
1. [`Вступление и определения`](<doc/rus/Upac chapter 0.md>);
2. [`Постановка задач`](<doc/rus/Upac chapter 1.md>);
3. [`Определение того, чем проект НЕ является (Non-goals)`](<doc/rus/Upac chapter 2.md>);
4. [`Структура диска`](<doc/rus/Upac chapter 3.md>);
5. [`Структура репозитория проекта`](<doc/rus/Upac chapter 4.md>);
6. [`Механизмы работы`](<doc/rus/Upac chapter 5.md>);
7. [`FFI и границы взаимодействия составных частей`](<doc/rus/Upac chapter 6.md>);
8. [`Модули программы`](<doc/rus/Upac chapter 7.md>).

## 🚀 Usage

```sh
up pkg install -f <path>...  [-m <message>]                       # installs package(s) from local file(s), with checksum verification (-f is local-only; a future network form will resolve by name instead)
up pkg remove <name>...      [--arch <arch>] [--arch-sub <sub>] [-m <message>]  # removes installed package(s) by name, optionally disambiguated by arch (alias: uninstall)
up pkg update -f <path>...   [-m <message>]                       # updates installed package(s) from local file(s) (same -f convention as install, for the same reason)
up pkg list                  [--version --arch --author --license --url --packager --size --description --checksum]  # lists installed packages, with optional extra columns
up pkg diff                  [<from>] [<to>]                      # diffs installed packages between two prefixes (commit digests); defaults if omitted
up pkg search <query>        [--version ... --checksum] [--regex] # searches package metadata (same field flags as pkg list)
up pkg search <query> --package <name> --package-arch <arch> [--package-arch-sub <sub>] [--regex]  # same, scoped to one package's own metadata

up file add <path>...    --package <name> --arch <arch> [--arch-sub <sub>] [-m <message>]  # tracks standalone file(s) against a package
up file remove <path>... --package <name> --arch <arch> [--arch-sub <sub>] [-m <message>]  # untracks standalone file(s) from a package
up file diff              [<from>] [<to>]                          # diffs tracked files between two prefixes
up file search <query>   [--regex]                                  # searches tracked files by path
up file search <query> --package <name> --package-arch <arch> [--package-arch-sub <sub>] [--regex]  # same, scoped to one package's files

up commit new <message>      # creates a new commit of the current deploy state
up commit pin <digest>       # pins a deploy so gc's automatic retention never removes it
up commit unpin <digest>     # unpins a previously pinned deploy
up commit list               # lists config-commits for the current deploy (rollback targets)
up commit prefixes           # lists deploy-level (prefix) commits
up commit history            # lists deploy-level commits with their nested config-commits, marking the active one
up commit diff [<from>] [<to>]  # diffs tracked files between two config-commits

up diff [--from-prefix <d>] [--to-prefix <d>] [--from-config <d>] [--to-config <d>]  # combined package + untracked-file diff across two commits

up rollback <commit>         # reverts the system state to a specified commit — not just commit state, can also target an earlier /usr prefix

up mime sync                 # regenerates desktop/mime-type integration (upac-mime.xml + .desktop MimeType=) from installed decoder manifests

up gc                        # removes unreachable commits/deploys and reclaims storage
```

## 🧩 Components

### Workspace layout

| Crate | Path | License | Role |
|---|---|---|---|
| `upac-abi` | `lib/abi` | LGPL-3.0-or-later | C-ABI types, error codes, and conversions shared between `upac-lib` and its consumers |
| `upac-macro` | `lib/macro` | LGPL-3.0-or-later | Derive macros for C-ABI struct plumbing, used internally by `upac-lib`/`upac-abi` |
| `upac-lib` | `lib/lib` | LGPL-3.0-or-later | Core library: composefs-based atomic deploys, `redb` package database, exposed via a C ABI (`libupac.so`) |
| `upac-cli` | `user/upac-cli` | GPL-3.0-only | CLI frontend (binary `up`) |
| `upac-sign-cli` | `user/sign-cli` | GPL-3.0-only | CLI for signing upac hook files and other artifacts with Ed25519 certificates (binary `up-si`) |

### Core Library (`upac-lib`)

The core library exposes a C-compatible ABI through `libupac.so`. All strings cross the boundary as `{ ptr, len }` pairs rather than null-terminated C strings. All functions return an integer error code.

### Decoders (`decoders/`)

Decoders are separate shared libraries that handle format-specific package unpacking. Each decoder receives a package path, an output directory, and a SHA-256 checksum; it verifies the checksum, extracts the package, parses the metadata, and returns a `PackageMeta` struct, its dependencies, and any declarative (package-format-native) trigger names it declares. All four decoders (`alpm`, `deb`, `rpm`, `xbps`) are written in [Rust](https://www.rust-lang.org/) and can also be statically linked into `upac-lib` via the `builtin-alpm`/`builtin-deb`/`builtin-rpm`/`builtin-xbps` Cargo features.

| Decoder | Formats | Distributions |
|---|---|---|
| **`libupac_decoder_alpm.so`** | `.pkg.tar.zst`, `.pkg.tar.xz`, `.pkg.tar.gz` | Arch Linux, Manjaro, etc. |
| **`libupac_decoder_deb.so`** | `.deb` | Debian, Ubuntu, etc. |
| **`libupac_decoder_rpm.so`** | `.rpm` | Fedora, RHEL, openSUSE, etc. |
| **`libupac_decoder_xbps.so`** | `.xbps` | Void Linux |

Adding support for a new package format means writing a new decoder `.so` — the core library does not need to change.

### CLI (`upac-cli`)

A command-line frontend written in Rust that dynamically loads `libupac.so` and the appropriate decoder at runtime. Subcommands are grouped under `pkg` (packages), `file` (standalone tracked files), `commit` (deploy history/rollback), and `mime` (desktop/mime-type integration), plus two top-level commands that don't belong to any single family: `gc` and `diff` (combined package + untracked-file diff).

## 🔧 Building

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable — pinned via `rust-toolchain.toml`)
- `libblkid`, `libmount` (`util-linux`) — used by `upac-lib` for filesystem/mount handling

### Build the Rust workspace

```sh
cargo build --workspace
```

### Static linking

By default `up` dlopens `libupac.so` at startup, and `upac-lib` in turn dlopens boot-plugin
`.so`s (`booters/{uki,systemd-boot,grub,refind}`) and decoder `.so`s (`decoders/{alpm,deb,rpm,xbps}`)
described by on-disk manifests — this is the `dynamic-plugins` feature, on by default on both
`upac-cli` and `upac-lib`.

Each crate also has a `static-link`/`builtin-*` axis for producing self-contained binaries with
no dlopen at all — `builtin-uki`/`builtin-systemd-boot`/`builtin-grub`/`builtin-refind` (bundled
as `builtin-all`) for boot plugins, `builtin-alpm`/`builtin-deb`/`builtin-rpm`/`builtin-xbps` for
decoders (no bundle yet):

```sh
# libupac.so with uki+systemd-boot+grub compiled in, `up` still dlopens it
cargo build --workspace --no-default-features --features upac-cli/dynamic-plugins,upac-lib/builtin-all

# one self-contained `up` binary, no dlopen anywhere
cargo build --workspace --no-default-features --features upac-cli/builtin-all
```

`dynamic-plugins` and `static-link` are mutually exclusive within a crate (both active means two
conflicting `impl` definitions, a compile error) — Cargo can't express that as a hard constraint,
so verify the whole feature graph with [`cargo-hack`](https://github.com/taiki-e/cargo-hack)
(`cargo install cargo-hack`) instead of guessing combinations by hand:

```sh
cargo hack build -p upac-cli -p upac-lib --feature-powerset \
    --mutually-exclusive-features dynamic-plugins,static-link \
    --at-least-one-of dynamic-plugins,static-link
```

### Build a decoder

`alpm`/`deb`/`rpm`/`xbps` are normal Rust workspace members, built along with everything else:

```sh
cargo build -p alpm -p deb -p rpm -p xbps
```

> **Note:** packaging (Arch/RPM/deb) has not been ported to this branch yet.

### Docs tooling (`xtask`)

The repo tree embedded in each design chapter under `doc/` is generated, not hand-edited. `xtask` is its own standalone workspace (see `xtask/Cargo.toml`) so it doesn't affect the main workspace's MSRV/edition:

```sh
cargo xtask gen-tree          # regenerate the tree in every marked doc file
cargo xtask gen-tree --check  # verify it's up to date, no writes
```
