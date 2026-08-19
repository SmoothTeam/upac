<!--
SPDX-FileCopyrightText: 2026 JustPav
SPDX-FileCopyrightText: 2026 SmoothTeam

SPDX-License-Identifier: CC-BY-SA-4.0
-->

## **§6.** FFI and the boundaries of interaction between the components.

Upac-lib (`lib`) is all of the program's core logic and the stable C-ABI. Decoders and boot plugins are loaded under its control.
Upac-cli and GUI programs built on the library (in the future) are thin wrappers: they only interpret input and emit events.

``` Diagram of control flow during the program's runtime.
CLI ─┐
GUI ─┼──(C-ABI)──▶ lib ──(dlopen)──▶ decoders/*     (format parsing + resolve)
…   ─┘             │
                   └──(calls)──▶ composefs         (repo / image / mount / boot)

lib ──(hooks)──▶ CLI/GUI                             (progress, events, /etc conflicts)
```

### Description of the diagram's directions:

- **External call → lib:** a command with arguments (what to do) + a cancel token;
- **Lib → decoders:** the path to the package; in return — files, metadata, dependencies;
- **Lib → composefs:** primitives, for example committing an image, mount, prune (cleanup), writing boot entries;
- **Lib → external call (hooks):** operation progress, confirmation events, `.upac-new` conflicts.

**Boundary rules:**
1. *"Touches state / needs full access to system files / must be atomic"* → `lib`;
2. *"Displays or gathers input"* → external call, external control. CLI and GUI are equal, thin wrappers over one library.

**`lib` has two public contracts.**

Besides the stable C-ABI (implemented via `export` and the OS's `dlopen` mechanism), the Rust layer itself is also public for direct static linking: `orchestrator`, `scripts`, `plugin`, `composefs`, `database`, `deploy`, `errors`, `lock`.

`export` remains private, since the C-ABI itself does not need to be invoked under static linking, as do the internal mechanisms implementing the logic of external types and functions, for example `Cursor` inside `orchestrator`. Inside `composefs`, the mechanisms `repository::open`/`repository::open_tree` are additionally cut from external export, since they are not part of the public contract — the sole point for obtaining an open repository/file tree from outside the API is the `deploy::Deploy` module, so that the repository cannot be opened bypassing an already-mounted sysroot, which would leave the system in an inconsistent state.
