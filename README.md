# 📦 Upac

### Links for repositories:

---

[![GitHub](https://img.shields.io/badge/GitHub-SmoothTeam%2Fupac-181717?logo=github)](https://github.com/SmoothTeam/upac)
[![Codeberg](https://img.shields.io/badge/Codeberg-justpav05%2Fupac-2185D0?logo=codeberg)](https://codeberg.org/justpav05/upac)

---

### General information:

---

[![Version](https://img.shields.io/badge/version-0.1.5-green)](https://github.com/SmoothTeam/upac/releases)
[![REUSE status](https://api.reuse.software/badge/github.com/SmoothTeam/upac)](https://api.reuse.software/info/github.com/SmoothTeam/upac)

---

### Licensing:

---

[![lib: LGPL-3.0-or-later](https://img.shields.io/badge/lib-LGPL--3.0--or--later-blue.svg)](LICENSES/LGPL-3.0-or-later.txt)
[![cli: GPL-3.0-only](https://img.shields.io/badge/cli-GPL--3.0--only-blue.svg)](LICENSES/GPL-3.0-only.txt)

---

> **⚠️ Branch in progress.** This branch (`lib-rs`) is a from-scratch rewrite of upac's core library in Rust, built around [composefs](https://github.com/containers/composefs) instead of OSTree. The FFI/orchestration engine is done; the actual command bodies, the hook system, and packaging are still being implemented — expect gaps and `todo!()`s.

A modular package management library for Linux systems with composefs-based atomic deploys.

## 🔍 Overview

**Upac** is a package manager for Linux-compatible systems. It manages the updating, removal, and installation of various package formats using per-format decoders, and keeps every change behind an atomic, [composefs](https://github.com/containers/composefs)-backed deploy so the system can always be rolled back to a previous commit.

**upac-cli** (binary: `up`) is a command-line frontend written in [Rust](https://www.rust-lang.org/) that dynamically loads `libupac.so` and drives it through its C ABI.

**upac-lib** is the core library, also written in Rust, designed to be embedded into package managers through a stable C ABI. It handles installation, database management (via [`redb`](https://github.com/cberner/redb)), and composefs-based system snapshotting — without imposing any policy on how packages are fetched or what format they come in.

The library is intentionally split into independent components: decoders handle format-specific unpacking, the core library handles installation and database operations, and everything crosses the FFI boundary through a shared `upac-abi` crate.

It covers the disk layout, the deploy/rollback model, the `/etc` merge, GC, the FFI boundary, and the planned module structure.

## 📖 Design

Full architecture and design decisions live in the project design notes:

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
up pkg install <path>    # installs a package into the system using the matching decoder, with checksum verification
up pkg remove <name>     # removes an installed package by name (alias: uninstall)
up pkg update <name>     # updates an installed package to a new version
up pkg list              # lists installed packages
up pkg diff              # diffs installed package versions against a commit or another package set
up pkg search            # searches package metadata

up file add <path>       # tracks a standalone file outside of a package
up file remove <path>    # untracks a standalone file
up file diff             # diffs tracked files against a commit
up file search           # searches tracked files by path

up commit new            # creates a new commit of the current deploy state
up commit list           # lists commit history
up commit rollback <id>  # reverts the system state to a specified commit ID
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

Decoders are separate shared libraries, still written in [Zig](https://ziglang.org/), that handle format-specific package unpacking. Each decoder receives a package path, an output directory, and a SHA-256 checksum; it verifies the checksum, extracts the package, parses the metadata, and returns a `PackageMeta` struct.

| Decoder | Formats | Distributions |
|---|---|---|
| **`libupac-alpm.so`** | `.pkg.tar.zst`, `.pkg.tar.xz`, `.pkg.tar.gz` | Arch Linux, Manjaro, etc. |
| **`libupac-rpm.so`** | `.rpm` | Fedora, RHEL, openSUSE, etc. |
| **`libupac-deb.so`** | `.deb` | Debian, Ubuntu, etc. |
| **`libupac-xbps.so`** | `.xbps` | Void Linux |

Adding support for a new package format means writing a new decoder `.so` — the core library does not need to change.

### CLI (`upac-cli`)

A command-line frontend written in Rust that dynamically loads `libupac.so` and the appropriate decoder at runtime. Subcommands are grouped under `pkg` (packages), `file` (standalone tracked files), and `commit` (deploy history/rollback).

## 🔧 Building

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable — pinned via `rust-toolchain.toml`)
- [Zig](https://ziglang.org/download/) ≥ 0.16.0 — for the decoders under `decoders/`
- `libblkid`, `libmount` (`util-linux`) — used by `upac-lib` for filesystem/mount handling

### Build the Rust workspace

```sh
cargo build --workspace
```

### Build a decoder

```sh
cd decoders/alpm
zig build
```

> **Note:** packaging (Arch/RPM/deb) and the old `make`-based build pipeline from the Zig implementation have not been ported to this branch yet.
