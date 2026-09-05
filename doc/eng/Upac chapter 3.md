<!--
SPDX-FileCopyrightText: 2026 JustPav
SPDX-FileCopyrightText: 2026 SmoothTeam

SPDX-License-Identifier: CC-BY-SA-4.0
-->

# **UPAC — the unified project document.**

Project document.
Project branch: **`lib-rs`**, crate `lib-rust/`.

## **§3.** Disk structure.

This paragraph describes what physically resides on the disks of a deployed system.

### Map

```
[block device, GPT]
│
├── ESP  (FAT32)                                     (1)
│   ├── EFI/Linux/upac-from.efi                      (2)
│   ├── EFI/Linux/upac-to.efi                        (2)
│   └── loader/entries/*.conf                        (3)
│
├── deployment partition   →  /sysroot                (4)
│   ├── composefs/                                   (5)
│   │   ├── meta.json                                (6)
│   │   ├── objects/<ab>/<hash…>                     (7)
│   │   ├── images/                                  (8)
│   │   │   ├── <digest>   →  ../objects/<ab>/…      (9)
│   │   │   └── refs/<name> →  ../images/<digest>    (10)
│   │   └── streams/                                 (11)
│   │       ├── <digest>   →  ../objects/<ab>/…
│   │       └── refs/<name>
│   └── state/deploy/<usr-digest>/                   (12)
│       ├── meta.json                                (12)
│       └── etc/{upper, work}                        (13)
│
├── /var partition    →  /var                        (14)
└── /home partition   →  /home                       (15)
```

### Map legend

- **(1)** ESP (EFI System Partition) — a separate FAT partition read by the UEFI firmware, mounted at `/boot` or `/efi`;
- **(2)** `upac-from.efi` / `upac-to.efi` — two fixed UKI slots for direct-UKI boot. An operation writes the new UKI into the inactive slot; the switch happens via `BootNext`;
- **(3)** `loader/entries/*.conf` — BLS entries for machines with a boot manager: systemd-boot and similar. Requires support for reading BLS entries.
- **(4)** deployment partition — the physical root holding all of the system's content; while the system is running it is mounted at `/sysroot` for changes. The file system requirement is **fs-verity** support. For example, this mechanism is supported by ext4, btrfs, xfs;
- **(5)** `composefs/` — the composefs repository: the content-addressed store for all files and images. The default composefs path for system mode;
- **(6)** `meta.json` — repo metadata: format version + fs-verity algorithm (`fsverity-<hash>-<lgbs>`);
- **(7)** `objects/` — the content-addressed store, in which objects are laid out into subdirectories named after the first 2 hex characters of the hash. Identical content is stored once;
- **(8)** `images/` — EROFS images: content-addressed snapshots of the `/usr` **and** `/etc` trees. They carry the tree's metadata; file data is taken from `objects/`;
- **(9)** `<digest>` — an image = a symlink to an object in `objects/`, determined by the image's hash;
- **(10)** `refs/<name>` — a human-readable named pointer to an image;
- **(11)** `streams/` — splitstreams: imported layers/commits, also symlinks into `objects/`, with their own refs added;
- **(12)** `state/deploy/<usr-digest>/` — a deploy record, in which the **key = `usr-digest`**. It holds `meta.json` inside.
- **(13)** `etc/upper` — **live `/etc`**: edits not included in the deploy, as the overlayfs upper layer over the current `working_etc`. It is sealed into `etc-digest` when `/usr` changes or on `upac commit`;
- **(14)** `etc/work` — **live `/etc`**: `work` — the overlayfs service directory;
- **(15)** `/var` — the directory is placed on a separate disk partition, thanks to which all changing data, logs, and databases are preserved directly and are not lost on a system rollback;
- **(16)** `/home` — user data: a separate directory holding user data, outside of versioning.
