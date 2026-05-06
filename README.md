# 📦 Upac

[![License: LGPL-3.0-or-later](https://img.shields.io/badge/License-LGPL_3.0+-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![GitHub](https://img.shields.io/badge/GitHub-justpav05%2Fupac-181717?logo=github)](https://github.com/justpav05/upac)
[![Version](https://img.shields.io/badge/version-0.1.4-green)](https://github.com/justpav05/upac/releases)

A modular package management library for Linux systems with OSTree integration.

## 🔍 Overview

**Upac** is a package manager for Linux-compatible systems. It manages the updating, removal, and installation of various package formats using different backends. It also supports [OSTree](https://ostreedev.github.io/ostree/) for rolling back the state of binaries to specific commits.

**upac-cli** is a command-line frontend written in [Rust](https://www.rust-lang.org/) that utilizes upac-lib as its backend. It supports formatted input and output, a polished installation interface, and display of errors and debugging information.

**upac-lib** is a low-level package management library written in [Zig](https://ziglang.org/), designed to be embedded into package managers through a stable C ABI. It handles the core operations of package installation, database management, and system snapshotting — without imposing any policy on how packages are fetched or what format they come in.

The library is intentionally split into independent components: backends handle format-specific unpacking, the core library handles installation and database operations, and OSTree integration is optional.

## 🚀 Usage

```sh
upac install   # installs files into the system using the selected backend, with support for checksum verification
upac remove    # removes an installed package from the system by name
upac rollback  # reverts the system state to a specified commit ID
upac list      # lists packages, with optional display of versions, commit history, or full details
upac init      # initializes the upac working environment in a specified mode (default: archive)
```

## 🧩 Components

### Core Library (`upac-lib`)

The core library exposes a C-compatible ABI through `libupac.so`. All strings cross the boundary as `{ ptr, len }` pairs rather than null-terminated C strings. All functions return an integer error code.

### Backends

Backends are separate shared libraries that handle format-specific package unpacking. Each backend receives a package path, an output directory, and a SHA-256 checksum; it verifies the checksum, extracts the package, parses the metadata, and returns a `PackageMeta` struct.

| Backend | Formats | Distributions |
|---|---|---|
| **`libupac-alpm.so`** | `.pkg.tar.zst`, `.pkg.tar.xz`, `.pkg.tar.gz` | Arch Linux, Manjaro, etc. |
| **`libupac-rpm.so`** | `.rpm` | Fedora, RHEL, openSUSE, etc. |
| **`libupac-deb.so`** | `.deb` | Debian, Ubuntu, etc. |
| **`libupac-xbps.so`** | `.xbps` | Void Linux |

Adding support for a new package format means writing a new backend `.so` — the core library does not need to change.

### CLI (`upac-cli`)

A command-line frontend written in Rust that dynamically loads `libupac.so` and the appropriate backend at runtime using [`libloading`](https://docs.rs/libloading). The backend is selected automatically by file extension or via an explicit `--backend` flag.

## 🔧 Building

### Prerequisites

- [Zig](https://ziglang.org/download/) ≥ 0.16.0
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- `libostree-1` — required for OSTree support (needed when building upac-lib)
- `libarchive` — required for backends
- `libglib-2.0`, `libgio-2.0` — pulled in by libostree

### Build everything

```sh
make build
```

By default, builds in **debug** mode. For an optimized release build:

```sh
make build MODE=release
```

To target a specific CPU architecture:

```sh
make build MODE=release CPU=native
```

### Build individual components

```sh
make build-lib              # → build/lib/libupac.so
make build-alpm-backend     # → build/lib/libupac-alpm.so
make build-rpm-backend      # → build/lib/libupac-rpm.so
make build-deb-backend      # → build/lib/libupac-deb.so
make build-xbps-backend     # → build/lib/libupac-xbps.so
make build-cli              # → build/bin/upac
```

## 📦 Packaging

> **Note:** Building packages requires the appropriate packaging tools installed on your system.

Build **and** package in one step:

```sh
make pkg-arch-local   # Arch Linux (.pkg.tar.zst) — requires makepkg
make pkg-rpm-local    # RPM-based (.rpm)          — requires rpmbuild
make pkg-deb-local    # Debian-based (.deb)        — requires dpkg-deb
```

Package only (assumes binaries are already built):

```sh
make pkg-arch
make pkg-rpm
make pkg-deb
```

### Version syncing

After bumping the version in `cli/Cargo.toml`, sync it across all build files and package specs:

```sh
make sync
```

## 🧹 Cleaning

```sh
make clean          # full cleanup (build artifacts + packages)
make clean-build    # only compilation results
make clean-pkg      # only built packages and the package build tree
```
