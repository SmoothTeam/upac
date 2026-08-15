<!--
SPDX-FileCopyrightText: 2026 JustPav

SPDX-License-Identifier: CC-BY-SA-4.0
-->

## 4. Project repository structure.

### Map of files in the repository.

<!-- tree:start -->
```text
upac/
├── .cargo/
│   └── config.toml
├── .claude/
│   └── settings.local.json
├── .github/
│   ├── workflows/
│   └── PULL_REQUEST_TEMPLATE.md
├── decoders/
│   ├── alpm/
│   ├── deb/
│   ├── rpm/
│   └── xbps/
├── doc/
│   ├── eng/
│   └── rus/
├── lib/
│   ├── abi/
│   ├── lib/
│   ├── macro/
│   └── pki/
├── LICENSES/
│   ├── CC-BY-SA-4.0.txt
│   ├── GPL-3.0-only.txt
│   └── LGPL-3.0-or-later.txt
├── user/
│   ├── sign-cli/
│   └── upac-cli/
├── xtask/
│   ├── src/
│   ├── Cargo.lock
│   └── Cargo.toml
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── CONTRIBUTING.md
├── LICENSE
├── README.md
├── REUSE.toml
├── rust-toolchain.toml
├── rustfmt.toml
└── SECURITY.md
```
<!-- tree:end -->

---


### Legend for the file map in the repository.

- **(1)** `.cargo/` - a folder for managing the project's Cargo build manager;
- **(2)** `config.toml` - a file that sets the project's build parameters;
- **(3)** `.github/` - a folder for managing the files behind the GitHub remote repository's automation;
- **(4)** `workflows/` - a folder for managing the files that create automated jobs with strictly defined input and output data on the GitHub remote repository;
- **(5)** `PULL_REQUEST_TEMPLATE.md` - a template for creating a pull request on GitHub;
- **(6)** `decoders/` - a folder with package format decoder plugins, one per format (no individual description is included);
- **(7)** `doc` - the project's documentation folder. There is a folder with Russian-language documentation and one with English-language documentation;
- **(8)** `lib/` — the Rust core of the library;
- **(9)** `abi/` — a Rust library for working with FFI;
- **(10)** `lib/` — the library's main working logic;
- **(11)** `macro/` — a library of procedurally-generated macros for the library's main code;
- **(12)** `pki/` — a library for generating and verifying all levels of certificates;
- **(13)** `LICENSES/` — a folder with licenses for reuse to work with;
- **(14)** `user/` — a folder with user-facing utilities;
- **(15)** `sign-cli/` — a CLI for working with certificates (signing them, verifying them);
- **(16)** `upac-cli/` — a CLI for working with the main library, carrying out the core operations;
- **(17)** `xtask/` — a script for automatically updating the repository map in the documentation, based on Cargo;
- **(18)** `.gitignore` — the configuration file for files ignored by the Git version control system;
- **(19)** `Cargo.toml` — the workspace file that manages the other nested projects;
- **(20)** `CONTRIBUTING.md` — a file describing how to get involved in contributing to the project;
- **(21)** `README.md` — the project's brief reference file;
- **(22)** `REUSE.toml` — the configuration file for the reuse licensing-check utility;
- **(23)** `rust-toolchain.toml` — the configuration file for the project's Cargo build system version;
- **(24)** `rustfmt.toml` — the configuration file for formatting the project's source code files;
- **(25)** `SECURITY.md` — a file describing how vulnerabilities in the program's code are handled and tracked in the project, along with reference information on the subject;
