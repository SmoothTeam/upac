<!--
SPDX-FileCopyrightText: 2026 JustPav

SPDX-License-Identifier: CC-BY-SA-4.0
-->

# **UPAC — the unified project document.**

Project document.
Project branch: **`lib-rs`**, crate `lib-rust/`.

---

## **§1.** Problem statement.

---

**State problem.**

On a typical Linux system, installing, updating, and removing software changes the running system *in place* — it edits its files right where they currently live. Because of this, at any given moment the system has no single, verifiable state: it is a pile of individually modified files. There are three consequences:
1. An interrupted or failed change leaves the system in a broken, half-finished state;
2. There is no way to reproduce or prove a specific *"known-working"* state;
3. There is no clean way to go back.

**Solution to the state problem.**

UPAC treats every system state as a **whole, content-addressed, verifiable image**. Any operation — install, update, remove — does not touch the running system, but instead **builds a new image from the current one**. Switching to the new image is a single atomic step, as a result of which the previous image remains untouched. This directly gives rise to three properties that answer the problem:
1. **The system has the property of atomicity**: an operation either completes in full, or the system remains as it was;
2. **The system has the property of reproducibility and verifiability**: every state is identified and attested by its hash, and a rollback is simply booting the previous image. The disk layout, bootloader, and kernel, meanwhile, remain fully under the user's control.

---

**Access problem.**

The system's base tree (`/usr`) is immutable and is managed by the package manager. If a user simply wants to add their own files there — for example, to drop in wallpapers or assets that a package expects under `/usr` — they cannot just copy them in. A couple of files have to be wrapped into a full-blown package: metadata, build, install, just for the sake of placing them there. The barrier to adding one's own content to the managed tree is unjustifiably high.

**Solution to the access problem.**

UPAC allows adding arbitrary user files to the managed tree (`/usr`) directly, with a single command, without writing a package. The file becomes full-fledged content of the image being built, but the user does not need the whole packaging pipeline.

---

**Compatibility problem.**

For one and the same Linux kernel there exist many incompatible package formats, for example: deb, rpm, pkg.tar, and so on. A program built for one package format will not install through another package manager, which locks the user into the ecosystem of their package format, even though the kernel and the ABI are shared by all of them.

**Solution to the compatibility problem.**

UPAC is not tied to a single package format. Parsing a specific format is delegated to separate backends — one per format — which bring the package to a common internal representation: a file tree and metadata. This lets a single manager install packages of different formats on one system, and the format stops being a compatibility boundary.

---

**Management problem.**

Even when a file is already on the system, it cannot be attached to a package as a user file — in a way that lets the manager track it and clean it up together with the package. This is especially painful under `/usr`: manually added files remain *"orphans"* outside of any tracking — they are invisible on removal and cannot be cleaned up automatically.

**Solution to the management problem.**

UPAC allows attaching a file to a package as a user file, with full accounting in the database. Such a file inherits the package's lifecycle: it is tracked, shown as part of the package, and removed along with it.
