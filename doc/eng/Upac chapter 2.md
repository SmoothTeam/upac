<!--
SPDX-FileCopyrightText: 2026 JustPav
SPDX-FileCopyrightText: 2026 SmoothTeam

SPDX-License-Identifier: CC-BY-SA-4.0
-->

# **UPAC — the unified project document.**

Project document.
Project branch: **`lib-rs`**, crate `lib-rust/`.

## **§2.** Defining what the project is NOT (Non-goals).

---

- **Not a distribution:** UPAC is a package manager and a deployment mechanism, not an operating system. It does not ship a curated repo, a default set of software, or a release cycle — it only manages whatever content it is pointed at;
- **Not a configuration manager:** UPAC reconciles `/etc` and preserves the user's edits across updates, but it does not generate or enforce config policy — it is not Ansible and not NixOS modules. It preserves and reconciles, it does not generate;
- **Not a container runtime:** UPAC uses the same building blocks as containers — composefs, OCI — but it deploys the host system, not containers. It is not a replacement for docker/podman;
- **No in-place changes — by design:** Any change to the system produces a new image; there is no hot-swapping of files on a live system, not even as an option. This is a direct consequence of the project's principles;
- **Not a repository server:** UPAC is only a client for ready-made external repos: distribution mirrors, OCI registries, and the like — it does not stand up its own repository or server. The only local alternative to a repo is delivering an image as a file: `--file`;
- **Does not repair the file system or the disk:** UPAC is responsible for the correctness of its own operations: package verification, image atomicity, repo integrity, and, via fs-verity, it **detects** content corruption, refusing to boot a damaged deploy. But recovering the file system itself, bad blocks, a degraded storage medium, or hardware errors is outside its scope: that is the job of `fsck`, SMART, and replacing the disk. A system falling apart because of a failing disk is not a failure of UPAC.
