# UPAC — Project Document

Project document.
Project branch: **`lib-rs`**, crate `lib-rust/`.

---

## 0. Introduction and definitions

This section explains terms so the document reads even without a systems-programming background. Later on, words from it are used without explanation.

### 0.1 Basic concepts

- **File** — a named piece of data on disk.
- **Directory (folder)** — a container for files and other directories.
- **Path** — a file's address in the directory tree, e.g. `/usr/bin/bash`.
- **Filesystem (FS)** — the way data is laid out on disk so the OS sees it as files and folders.
- **Disk / block device** — a storage device (physical or virtual).
- **Partition** — a dedicated slice of a disk that holds one filesystem.
- **GPT** — the modern disk partitioning scheme.
- **Mount** — "attaching" a partition's filesystem at a point in the directory tree; afterwards its contents are reachable at that path.
- **Kernel** — the core of the OS: manages hardware, memory and processes.
- **initramfs** — a tiny temporary filesystem the kernel brings up first, to prepare and mount the real system root.
- **Bootloader** — the program the firmware (UEFI) launches, which in turn launches the kernel.
- **cmdline (kernel command line)** — the set of parameters passed to the kernel at start.
- **Hash / digest** — a short "fingerprint" of content: identical data yields the same hash, any change yields a different one.
- **Package** — a bundled set of files (a program plus its data and metadata), installed as a unit.
- **Package manager** — the program that installs, updates and removes packages.
- **Atomicity** — an "all or nothing" property: an operation either applies in full or not at all, with no half-applied intermediate states.
- **Immutable (system)** — a system whose core part is mounted read-only and is not changed in place.

### 0.2 System folders

What each is responsible for — and how it maps onto the "main item → sub-item" hierarchy.

- **`/usr`** — the system base: libraries, binaries, everything responsible for running and booting. Immutable. The **main item** — the axis of failure rollbacks.
- **`/etc`** — system configuration. The **sub-item** — bound to its `/usr`, versioned separately, and on rollback it follows its main item.
- **`/var`** — mutable runtime state: logs, data, DB. Persistent, not versioned, not lost on rollback.
- **`/home`** — user data. Persistent, outside versioning.
- **`/boot` and ESP** — the partition read by the UEFI firmware: boot entries and the kernel/initramfs (UKI). The system starts here before the real root appears.
- **`/sysroot`** — the point where the physical root partition is mounted; the real `/` is assembled over it.

### 0.3 Project jargon

Terms the rest of the document operates with.

- **Content-addressed store (CAS)** — storage where a file is addressed by the hash of its content, not by name; identical files are stored once (deduplication).
- **Image** — a whole content-addressed snapshot of a file tree (`/usr` or `/etc`) at a specific version.
- **Digest** — an image's hash, its identity; names the deploy in the cmdline.
- **Deployment** — a specific deployed version of the system; in this model = a pair `(usr-digest, etc-digest)`.
- **Ref** — a human-readable named pointer to an image.
- **overlay (lower / upper)** — stacking a writable layer (upper) over a read-only one (lower); this is how the live `/etc` sits over the image.
- **fs-verity** — a kernel feature that cryptographically attests a file's content and catches any change to it.
- **One-shot entry** — a boot entry the firmware/bootloader selects exactly once, after which it reverts to the persistent default (the basis of auto-rollback, §5.2).
- **`base` / `new` / `live`** — the three inputs of the 3-way `/etc` merge (§5.1): the old default, the new default, the user's current state.
- **`.upac-new`** — a new config default placed alongside on a conflict, so as not to overwrite the user's edit.
- **`seq`** — a monotonic deploy sequence number; the authority for history order and rollbacks.
- **Pin** — a deploy protected from garbage collection (active, rollback target, user-pinned).
- **Rollback** — returning to a previous version: of the main item (`/usr`) on a failure, or of the sub-item (`/etc`) separately.

---

## 1. Problem statement

---

**The state problem.**
On an ordinary Linux system, installing, updating and removing software changes the running system *in place* — editing its files right where it currently lives. Because of this the system has no single verifiable state at any moment: it is a heap of individually mutated files. Three consequences follow: an interrupted or failed change leaves the system in a broken half-state; you cannot reproduce or prove a specific "known-good" state; and there is no clean way to go back.

**Solution to the state problem.**
UPAC treats every system state as a **whole, content-addressed, verifiable image**. Any operation (install / update / remove) does not touch the running system but **builds a new image from the current one**. Switching to the new image is a single atomic step, and the previous image stays intact. Three properties follow directly and answer the problem: an operation either completed in full or the system stayed as it was (**atomicity**); any state is identified and attested by hash (**reproducibility and verifiability**); rollback is simply booting the previous image (**reversibility**). Meanwhile disk layout, bootloader and kernel stay fully under the user's control.

---

**The access problem.**
The system's base tree (`/usr`, etc.) is immutable and manager-owned. If a user just wants to add their own files there — say drop wallpapers or assets a package expects in `/usr` — they cannot simply copy them in. They have to wrap a couple of files into a full package (metadata, build, install) just to place them. The barrier to adding your own files to the managed tree is unreasonably high.

**Solution to the access problem.**
UPAC lets you add arbitrary user files into the managed tree (including `/usr`) directly, with a single command, without authoring a package. The file lands in the built image as first-class content, but the user needs none of the packaging pipeline.

---

**The compatibility problem.**
For one and the same Linux kernel there are many incompatible package formats (deb, rpm, pkg.tar, etc.) and distributions. Software built for one format won't install into another without contortions; the user is locked into the ecosystem of their package format, even though the kernel and ABI are shared by all.

**Solution to the compatibility problem.**
UPAC is not tied to a single package format. Parsing a specific format is moved into separate backends (one per format) that bring a package to a common internal representation — a file tree plus metadata. Thanks to this a single manager installs packages of different formats onto one system, and the format stops being a compatibility boundary.

---

**The management problem.**
Even once a file is in the system, it cannot be attached to a package as a user file — so the manager would track it and clean it up together with the package. This hurts especially in `/usr`: manually added files remain "orphans" outside any accounting — invisible on removal and not cleaned automatically.

**Solution to the management problem.**
UPAC lets you attach a file to a package as a user file, fully accounted for in the database. Such a file inherits the package's lifecycle: it is tracked, shown as part of the package, and removed together with it, wherever it lives (including `/usr`). User additions stop being untracked clutter.

---

## 2. Non-goals

Things reasonably expected of a manager of this kind that UPAC deliberately does NOT do — to draw the boundaries and head off future "just add this too".

- **Not a distribution.** UPAC is a package manager and a deployment mechanism, not an OS. It does not ship a curated repo, a default software set or a release cycle; it manages whatever content it is pointed at.
- **Not a configuration manager.** UPAC merges `/etc` and preserves user edits across updates, but it does not generate or enforce config policy — it is not Ansible or NixOS modules. It preserves and reconciles, it does not author.
- **Not a container runtime.** UPAC uses the same building blocks as containers (composefs, OCI) but deploys a host system, not containers. It does not replace docker/podman.
- **No in-place changes — by design.** Any change to the system produces a new image; there is no hot in-place file replacement on the live system, not even as an option. This follows directly from the project's principle (see §1, "The state problem").
- **Not a repository server.** UPAC is only a client to existing external repos (distro mirrors, OCI registries, etc.); it does not stand up its own repository or server. The only local alternative to a repo is delivering the image as a file (`--file`).
- **Does not fix the filesystem or the disk.** UPAC owns the correctness of its own operations (package verification, atomic images, repo integrity) and via fs-verity **detects** content corruption, refusing to boot a corrupted deploy. But repairing the FS itself, bad blocks, a degraded drive or hardware errors is out of scope: that's the job of `fsck`, SMART and disk replacement. System breakage caused by a bad disk is not a UPAC failure.

---

## 3. Disk structure

From here on the document describes the concrete implementation. Its core is the **composefs** dependency: it provides the content-addressed store, building a verifiable image, and mounting it. This section describes what physically lives on the disks of a deployed system.

### Map

```
[block device, GPT]
│
├── ESP  (FAT32)                                    (1)
│   ├── EFI/Linux/upac-from.efi                      (2)
│   ├── EFI/Linux/upac-to.efi                        (2)
│   └── loader/entries/*.conf                        (3)
│
├── deployment partition   →  /sysroot              (4)
│   ├── composefs/                                   (5)
│   │   ├── meta.json                                (6)
│   │   ├── objects/<ab>/<hash…>                     (7)
│   │   ├── images/                                  (8)
│   │   │   ├── <digest>   →  ../objects/<ab>/…      (9)
│   │   │   └── refs/<name> →  ../images/<digest>   (10)
│   │   └── streams/                                (11)
│   │       ├── <digest>   →  ../objects/<ab>/…
│   │       └── refs/<name>
│   └── state/deploy/<usr-digest>/                  (12)
│       ├── meta.json                               (12)
│       └── etc-upper/{upper, work}                 (13)
│
├── /var partition    →  /var                       (14)
└── /home partition   →  /home                       (15)
```

### Legend

- **(1)** ESP (EFI System Partition) — a separate FAT partition read by the UEFI firmware; mounted at `/boot` (or `/efi`). It holds everything needed to start up before the real root appears.
- **(2)** `upac-from.efi` / `upac-to.efi` — two fixed UKI slots (a signed kernel + initramfs + cmdline image) for direct-UKI boot. An operation writes the new UKI to the inactive slot; switching goes through `BootNext`.
- **(3)** `loader/entries/*.conf` — BLS entries for machines with a boot manager (systemd-boot, etc.), an alternative to direct-UKI. Read/written via `BootconfigParser`.
- **(4)** deployment partition — the physical root with all the system's content; at runtime mounted at `/sysroot`. FS requirement — **fs-verity** support (ext4 / btrfs / xfs). The real root `/` is assembled over the image by mounting (overlay); there is no unpacked file tree on disk.
- **(5)** `composefs/` — the composefs repository: the content-addressed store of all files and images. The composefs default path in system mode.
- **(6)** `meta.json` — repo metadata: format version + fs-verity algorithm (`fsverity-<hash>-<lgbs>`).
- **(7)** `objects/` — the content-addressed store; objects are laid out under subdirectories from the first 2 hex chars of the hash. Identical content is stored once (deduplication).
- **(8)** `images/` — EROFS images: content-addressed snapshots of `/usr` **and** `/etc` trees (a deploy references the pair). They carry the tree metadata, file data is taken from `objects/`.
- **(9)** `<digest>` — an image = a symlink to an object in `objects/`. The image digest = its identity (also in the boot cmdline).
- **(10)** `refs/<name>` — a human-readable named pointer to an image.
- **(11)** `streams/` — splitstreams (imported layers/commits as a content source), also symlinks into `objects/` plus their own refs.
- **(12)** `state/deploy/<usr-digest>/` — a deploy record, **keyed by `usr-digest`** (one per distinct `/usr`, deduped). `meta.json` carries: `usr_digest`, `seq` (birth order, the rollback authority), `timestamp` (for display), `etc_history` (an ordered list of this `/usr`'s `etc-digest`s) and `working_etc` (the working sub-item, set by boot-confirm). Re-arriving at the same `usr-digest` = switching to this record, not a duplicate. See §5.7.
- **(13)** `etc-upper/{upper, work}` — the **live `/etc`**: uncommitted edits as an overlayfs upper layer over the current `working_etc` (§5.1). Sealed into an `etc-digest` on a `/usr` change or via `upac commit`; `work` is the overlayfs service directory.
- **(14)** `/var` — persistent runtime state (logs, data, DB): a **separate real partition**, not versioned and not lost on rollback. The composefs reference per-digest overlay for `/var` is NOT used here — a real partition is mounted.
- **(15)** `/home` — user data: a separate persistent partition, outside versioning.

---

## 4. Repository structure

The target layout of the project repository.

### Map

```
upac/
├── Cargo.toml         (1)
├── rustfmt.toml
├── README.md
├── CHANGELOG.md
├── LICENSE
├── SECURE.MD          (2)
├── .github/           (3)
├── .gitignore
├── doc/               (4)
├── lib/               (5)
├── derive-static/     (6)
├── cli/               (7)
├── decoders/          (8)
│   ├── alpm/
│   ├── deb/
│   ├── rpm/
│   └── xbps/
└── tests/             (9)
```

### Legend

- **(1)** `Cargo.toml` — the workspace (`lib`, `derive-static`, `cli`).
- **(2)** `SECURE.MD` — the project security policy.
- **(3)** `.github/` — CI configuration.
- **(4)** `doc/` — project documents (this note, etc.).
- **(5)** `lib/` — the Rust core of the library.
- **(6)** `derive-static/` — a proc-macro crate: variables from a constants file.
- **(7)** `cli/` — the CLI frontend.
- **(8)** `decoders/` — package-format decoder plugins, one per format.
- **(9)** `tests/` — integration tests.

---

## 5. Mechanisms

Separate mechanisms the core (`lib/`) implements, either additionally or by using composefs mechanisms.

### 5.1 Config merge (`/etc`)

The live `/etc` at runtime is `overlay(lower = the committed etc-digest, ro; upper = uncommitted edits, rw)`. `/etc` is versioned content-addressed: each snapshot = an `etc-digest` (see §5.7). The merge's job — on a `/usr` change, build the new `/etc` (carry over the user's edits, pull in the new package defaults) and seal the result as the first `etc-digest` of the new `/usr`.

The mechanism is library-side and runs at the `merge` stage, before the new deploy becomes bootable.

**Three inputs (3-way):**
- **base** — the `/etc` defaults of the current `/usr` (the one the live system was built from);
- **new** — the `/etc` defaults of the new `/usr` being deployed;
- **live** — the user's current live `/etc` (the committed `working_etc` + the previous deploy's uncommitted upper).

**Per-file classification:**
- the user did NOT touch the file (`live == base`) → the **new default** goes into the result;
- the user edited it, and the new default equals the old one (the package didn't change the file) → the user's version is preserved;
- the user edited it AND the new default changed (a conflict) → the user's version stays live, and the new default is placed alongside as `<file>.upac-new` (excluded from future classification — it is not "the user's file").

**Conflicts — via a hook, non-blocking.** The operation does not stall: the deploy goes through, and `.upac-new` files signal to the user via a hook event (in the CLI) that there is something to reconcile.

**The result** is sealed into a new `etc-digest`, which becomes the new deploy's `working_etc`; its live upper starts empty. Unchanged files are deduplicated by composefs at the object level, so an `etc-digest` is a full snapshot of `/etc` without duplicating content.

On `upac commit` the same mechanism seals the current live `/etc` without a `/usr` change — a new `etc-digest` under the same `/usr`.

### 5.2 System boot and rollback on failure

Rollback is built on a one-shot boot choice plus a late confirmation that the system started successfully; there is no separate attempt counter — an image that failed to boot the first time, for whatever reason, is not retried.

**The one-shot choice mechanism.** The bootloader/firmware has a pair "one-shot entry / persistent default": UKI-direct — `BootNext` / `BootOrder`; systemd-boot — `LoaderEntryOneShot` / `LoaderEntryDefault`; grub — `grub-reboot` / the persistent default in its config. The firmware/bootloader clears the one-shot variable on any boot, so it is itself a single-attempt auto-rollback.

**Staging and boot:**

1. On deploying D' a boot entry with `composefs.digest=D'` is written (a UKI to the inactive `upac-to.efi` slot, or a BLS conf via `BootconfigParser`), but it is NOT made the persistent default — it is set as the one-shot entry for the next boot; the persistent default stays on the previous working deploy.
2. Reboot: the bootloader boots D' once, the one-shot variable is cleared. initramfs mounts the digest from the cmdline (composefs overlay), then pivot and PID1.
3. The system reached a healthy state — a late hook / init unit makes D' the persistent default and marks the **pair as working**: it updates the current `/usr`'s `working_etc` (§5.7). This is the confirmation.
4. The confirmation did not fire (the system went down earlier, for any reason) — the one-shot variable is already cleared, so the next boot goes to the persistent default, i.e. the previous deploy. This is the auto-rollback.

**Rollback tiers** (which level catches what):

1. Kernel or initramfs did not come up — the firmware itself goes to the persistent default (one-shot cleared) = the previous deploy.
2. Booted, but PID1 did not come up — the confirmation did not arrive, the next boot rolls back; on a manager the previous deploy can also be picked manually from the menu.
3. PID1 came up, but services/network/GUI are dead — rollback from the live system via `upac rollback`, or reboot to the menu.
4. Full brick — the firmware menu, or a Live-USB + `upac rollback --root`.

If the system formally reached a working state and confirmed, but some subsystems or tools did not come up or work incorrectly — a manual rollback is available: `upac rollback` from the live system, or the firmware/bootloader menu.

**Limitations (deliberate):**

- one attempt, not N: a broken atomic image is deterministically broken, retrying makes no sense;
- auto-confirmation proves "reached a healthy target", not "the user is happy" — deeper breakage is rolled back manually via `upac rollback`;
- a pure hang (PID1 alive but stuck, no panic and no reboot) needs a manual power-cycle for the one-shot variable to take effect.

### 5.3 Deployment staging (stage)

Input — image D', already in the repo (`images/D'`); output — a deploy ready for a one-shot boot. It links operations (§5.4) to boot (§5.2).

1. `/etc` merge (§5.1): the merge seals an `etc-digest` for D' and sets it as `working_etc`; the live upper (`etc-upper/`) starts empty.
2. Persistent partitions (`/var`, `/home`) — real, mounted as-is, untouched.
3. Write the boot entry with `composefs.digest=D'`:
   - UKI-direct — build and sign the UKI (kernel + initramfs + cmdline), write it to the inactive `upac-to.efi` slot;
   - manager — `BootconfigParser` writes a BLS conf (`options composefs.digest=D'`) into `loader/entries/`.
4. Set D' as the one-shot entry for the next boot (§5.2): UKI-direct — `BootNext` on the slot; manager — `LoaderEntryOneShot` / `grub-reboot`. The persistent default is not touched — it stays on the previous deploy.

Then — reboot and §5.2 (boot, confirmation or auto-rollback).

### 5.4 Operations: add / remove / update

All three are one shape: change the tree → commit a new image → hand it to deployment staging (§5.3). The old image is not touched until the switch (atomicity). This is where decoders and the resolver work, and where the package DB is written.

Common pipeline:

1. Build a new tree from the current one (the difference is per-operation, below).
2. Commit the tree as a new image D' into the repo (`objects/` + `images/D'`); the package DB is written inside the image.
3. Hand D' to deployment staging (§5.3).
4. Light deploy-prune (§5.5) as the final stage.

Difference in step 1:

- **add (install):** the decoder parses the package → the resolver adds dependencies → new tree = current + the package(s) files.
- **remove:** new tree = current − the package's files − attached user files.
- **update:** the decoder parses the new version → new tree = current with the package's files replaced; the `/etc` merge (§5.1) at staging carries in the new defaults.

### 5.5 Garbage collection (GC)

Two levels: deployments (what we keep) and objects (what to sweep). The retention policy is set by the user; the object-sweep engine is composefs.

**Immutable pins** (never removed):

- the active (booted) deploy;
- the rollback target (the persistent default);
- the staged-but-unconfirmed deploy (the one-shot entry).

Plus the user's manual pins (pinned deploys) and the last N within the user-set depth.

**Triggers:**

1. **Light deploy-prune — as an internal stage** after each mutating operation: drop the image ref and remove `state/deploy/<D>/` for deploys outside the policy. Cheap, the pins hold what's needed.
2. **Heavy object-sweep — manual only**, via the `upac gc` command: walk `objects/` and `streams/` and sweep the unreachable (composefs `ObjectCollector`).
3. GC is never hung on the boot/confirm path, nor on a timer.

### 5.6 OCI (planned)

This section is future work. OCI here is a portable image-artifact format, not a network protocol; UPAC does not stand up its own network stack beyond the repo.

**Directions:**

- **import** — take an OCI image and deploy it as a host system;
- **export** — produce a portable OCI image artifact from a deploy (a ready "reference copy").

**Image delivery — two existing paths:**

1. **from an external repo** (as a client; the default) — the same mechanism as for packages;
2. **`--file <image>`** — a local file, taken and deployed directly.

Fleet deploy (a reference image → a fleet of machines) travels by the same two paths: image in the repo → machines pull, or handed out as a file. There is no separate fleet transport and no registry push.

Building blocks (`composefs-oci`): `create_filesystem` (layers → image), `generate_boot_image`, `pull_image`.

### 5.7 History and rollback by N deploys

History is stored NOT in the image (it would break content-addressing) nor in ESP, but on the writable partition. The source of truth is the `state/deploy/<usr-digest>/` directories themselves; there is no separate journal.

**Two axes.**

- **`/usr` — the linear deploy history.** Each distinct `/usr` = one record, keyed by `usr-digest`. `seq` is the birth order of records (monotonic, one per digest; high-water-mark in `state/next-seq`, written tmp+rename). Re-arriving at an existing `usr-digest` **switches** to its record rather than creating a duplicate — so that `/usr`'s `/etc` sub-history stays intact when you return. The record carries its own **commit message**: `subject` (short, required) + an optional long `message` — the commit message of the operation that gave birth to this `/usr` (install/uninstall/update).
- **`/etc` — a sub-history within `/usr`.** The record's `meta.json` carries `etc_history` — an ordered list of `{etc_digest, subject, message}` records taken under this `/usr` (on a `/usr` change and via `upac commit`, §5.1). Each record carries its own `subject` + optional `message`; the first record, created by the automatic merge on a `/usr` change (§5.1), inherits the subject+message of that `/usr` event itself — later explicit `upac commit` calls get their own, independent subject+message.

**The active deploy is a separate pointer** (the booted `composefs.digest` / boot default), not `max(seq)`: after switching to an old record, its `seq` stays as it was.

**Rollback:**

- **failure (`/usr`)** — to the Nth existing deploy in `seq` order (by actual presence, **not** by arithmetic `seq−N`: GC, pins and burned numbers leave gaps in `seq`, which we skip). The **pair** is restored: the target `usr-digest` + its `working_etc` (the last confirmed sub-item) — config edits are not lost.
- **config (`/etc`)** — `upac rollback --etc` to a previous `etc-digest` from the current `/usr`'s `etc_history`; the same rank-based mechanics and its own retention depth.

**`seq`** is authoritative for order and rollbacks; **`timestamp`** in `meta.json` is for display only (`upac history`), the order is always by `seq`, so that clock drift or a time change does not reorder history.

Relation to GC (§5.5): the retention depth on each axis must be **≥ the max rollback N** for that axis, otherwise history is shorter than the promise — the deploy at position N is already swept.

### 5.8 Hooks (pre/post triggers)

A hook is not code but a declarative, **signed** file: it describes a trigger, a priority, and a composition of **primitives**. A primitive is a closed set of low-level actions baked into `lib` (spawning a process, touch/move a file, etc.) — the only thing that requires a code change in `lib`. The hook as a unit is not an `enum` and is not enumerated anywhere in code: it is entirely described by data in the file, and `lib` is just a generic executor of that composition. The signature guards against an arbitrary, unsigned hook file being dropped in (primitives are privileged enough that a file without one can't be trusted).

**Compatibility table.** The hook file separately carries a table: for decoder `D` (§6 — a package-format plugin: deb, rpm, native, …) this hook covers such-and-such of its NATIVE trigger name (e.g. deb's `Triggers-Interest: update-mime-database`). This way, compatibility with a foreign trigger convention is also data in the file, not hardcoded inside the decoder.

**Priority and criticality.** `priority` is a plain signed integer (default 0), needed ONLY to resolve a conflict when several different hook files claim the same native trigger name (the same key `k` in the compatibility table) — the higher `priority` wins; an exact tie is an unresolvable conflict, and `lib` fails outright rather than silently picking one (`build_trigger_table` resolves this before a table is ever sent to a decoder — a tie is a hard `Err`, same treatment as two decoders claiming the same package format, see below). Unmatched entries (a hook's native trigger name simply isn't present in this particular package) need no separate reporting at all — not being matched is the normal, expected outcome for most hooks on most packages, not an error. Whether a *non-fatal* warning should still surface through `MessageHook` for either case (a conflict, or a hook that structurally never matches anything) is still an open question. `priority` sets no execution ordering — all hooks matched to a single trigger point run concurrently, with no tiers. Criticality (abort vs. best-effort on hook failure) is a field on the hook file itself (`critical = true/false`), not a property of the primitive: a primitive is neutral, and whether "failure means abort the whole operation, or just warn" is known only to the hook's author, who already signed the file (trust is already established through the CA chain, no extra primitive-level veto is needed).

**Division of labor:**

- **`lib`** — the only side that reads hook files off disk, verifies the signature, and parses the primitive composition and the compatibility table. It executes the composition through its own primitives.
- **Decoder (plugin)** — does not parse hook files itself. It receives from `lib` the compatibility table already as a ready k:v map **scoped to its own `D`** (a deb decoder never gets entries meant for rpm/native, for instance), where k is the decoder's native trigger name and v is our hook. The decoder itself matches this against the native trigger names it read out of the package (e.g. the deb decoder reads the package's own `Triggers-Interest`), and hands back to `lib` over FFI the list of hooks to execute (the matched v's). Matching stays on the decoder's side, but its input is already-parsed data from `lib`, not a raw hook file.

**Decoder resolution.** Decoders are found via declarative TOML manifests in `/etc/upac.d/decoders/` (one per decoder — `format`, `extensions`, `library`), never by scanning or dlopen-probing `.so` files directly: `format` is the canonical package-format identity, the same string used as the key `D` in a hook's compatibility table (one decoder never covers more than one format); `extensions` lists the file variants that format actually ships as (e.g. alpm packages as `pkg.tar`/`pkg.tar.gz`/`pkg.tar.xz`/`pkg.tar.zst` — one format, several file shapes); `library` names the `.so` to `dlopen` on first use, lazily, only once a package of that format is actually being processed. A duplicate `format` across two manifests is a hard error at manifest-load time, same treatment as the priority-tie case above. Since none of this touches actual shared-library resolution, decoders don't need to (and don't) self-report their own identity over FFI — the manifest is the sole source of truth.

**Hook file format and signing.** The hook file is TOML, living in `/etc/upac.d/hooks/` (the path is baked in as a constant, `Lib.toml`-style). The signature is a separate sidecar file (`name.hook` + `name.hook.sig`), not a field inside the TOML (otherwise byte canonicalization on re-serialize/re-parse would have to be pinned down). The signature chains through 2 tiers: a root CA (offline key, only ever signs the next tier) → a signing certificate per trust domain (e.g. "upac-core", or one per distro/maintainer), which directly signs the `.hook` file's bytes; there is no separate leaf tier — extra key rotation with no real benefit at this narrow a scope. The `.sig` file carries both the signature and the signing certificate itself in full — verification is self-contained (root + `.sig`, nothing else to look up). **The root is a configurable file**, not baked into the binary — that's the whole point of this system: a distro/OEM plugs in its own root without rebuilding `upac`. The signature scheme is Ed25519 over X.509 certificates, via the `x509-cert` crate (pure Rust, RustCrypto — no OpenSSL/CMS dependency). Implemented in its own crate, `upac-pki` (`lib/pki/`, LGPL, no dependency on `upac-abi`/`upac-lib`) — `RootIdentity`/`SigningIdentity` (generation), `HookSignature::sign`/`verify`, and the `Identity` trait (`to_bytes`/`from_bytes`) for saving/loading key+cert pairs across process invocations. Depended on by both `upac-lib` (verification) and `upac-sign-cli` (signing), so the `.sig` byte format can't drift between the two sides.

**Execution model.** Hook execution is asynchronous, but entirely inside `lib`: the FFI stays synchronous (the `extern "C" fn` calls a plain function, which spins up a `tokio` runtime and does `.block_on(...)`). The scope is limited to concurrently running the N independent hooks of a single trigger point **inside one `Stage::run()`**; the `Orchestrator` itself does not become async — stages must still run linearly. For CPU-bound work (hashing the tree on add/remove/update, §5.4) — `rayon`/plain threads, not async: local `tokio::fs` on Linux without io_uring is itself blocking under the hood (`spawn_blocking`), so async gives no benefit there. `tokio-uring` (io_uring) is deliberately not adopted: it's a different, cancellation-unsafe buffer-ownership model, Linux-only, younger, and viewed cautiously from a security standpoint — revisit only if profiling actually shows syscall overhead dominating on huge trees.

### 5.9 Operation cancellation

A second hook channel, independent of §5.8 — not declarative pre/post triggers, but a system-level cancellation signal. `CancelToken` (`#[repr(C)]`, an atomic flag) is created by the calling side (CLI/GUI) and passed into `lib` as a pointer with every request.

**`Lock`** is a pure mutual-exclusion mechanism between rw operations, with no connection to cancellation at all (bind on a fixed abstract Unix address shared by all rw calls: taken — `EADDRINUSE` → `LockError::Busy`; free — proceed). The `Orchestrator` itself holds it for the whole rw operation; ro doesn't take it at all.

**Access to cancellation is an explicit parameter, no wrapper.** `&CancelToken` is passed directly into `Stage::run`/`Orchestrator::run`, and each stage reads `.is_cancelled()` itself, including inside its own loops. There is no separate wrapper combining `Lock` and the token — these are two independent mechanisms with no relation to each other, exactly as intended at the start of this section.

### 5.10 Operation progress

The same hook channel as §5.9 (`MessageHook`), but for progress tracking rather than cancellation. The point of `data` isn't an abstract "extra info" tack-on — it's specifically tracking what's happening INSIDE a stage: each event is a transition into a named sub-step (phase) of the current stage's mini-FSM, not a generic completion percentage.

The payload is one common `#[repr(C)]` type for every event, not a separate shape per event (a tagged union would be awkward over a C-ABI, and there's nothing yet to check the shape against — stage bodies aren't written): `stage` (which stage, as before — `StateId as u32`), `phase` (which sub-step within the stage; the meaning is owned by the stage itself, like `StateId`), `subject` (a borrowed string — what this concerns: a package, a file, a hook), `current`/`total` (an item counter, `0` if not applicable/unknown). `MessageHook::send` takes one self-describing parameter instead of the former separate event/data pair — nothing to unpack separately. The `Orchestrator` creates the builder (with `stage` already set, from its own index in the pipeline — see §5.11) and passes it to the stage as a parameter; the stage fills in `phase`/`subject`/`progress()` and hands the builder back — the `Orchestrator` itself finalizes it (`.build()`) and sends it to the hook.

### 5.11 Stage orchestration

The mechanism that actually runs mutating commands (and later read-only ones too): a linear list of stages plus an engine that walks through them — both hook channels from §5.9/§5.10 plug in here.

**A stage is always flat.** One stage does exactly one atomic unit of work per call, never loops internally. Each call itself decides what happens next — advance to the next stage, repeat itself (e.g. process one more file from an already-started list), or jump BACK to an earlier stage by its TYPE (not by a numeric index — so inserting new stages anywhere in the pipeline never breaks it). Through such a backward jump, a group of several stages (e.g. "verify package → unpack → register") can repeat as a whole — on a jump, the engine searches backward from the current position for the nearest stage of that type; if no such stage exists, that's a pipeline-assembly bug, not user input, and it comes back as an ordinary error, not a crash.

**Every stage call brings its own rollback.** A stage doesn't accumulate state between calls — each call constructs its own, independent rollback object, carrying exactly the data needed to undo what THAT call did (even if the same stage was called many times before with different data — each call stays independent). If a stage has nothing to roll back this time, it must still return such an object, just in an "empty" mode: this is guaranteed at the compiler level (the "empty" rollback constructor is a mandatory part of the contract), not a convention that can be forgotten. The real, data-carrying rollback constructor (specific to each stage) is not part of the shared contract — every stage has its own, with its own signature.

**The engine** holds the linear stage list; exclusivity isn't a stored mode but a choice of which METHOD you call to run it: one holds the system lock (unrelated to the cancellation in §5.9, a separate mutual-exclusion mechanism between rw operations) for the whole run, the other never touches it at all — for read-only. On any failure (a stage error, cancellation, an unresolved backward jump) it unwinds every rollback object collected so far, in reverse order, without stopping if one of them fails to roll back itself — everything that can be undone gets attempted, rather than bailing out halfway. Every successful stage call hands the engine two separate things at once — the progress builder (§5.10, which the engine itself created and passed to the stage before the call) and its own rollback object: the former is passed straight through to the hook as-is, the latter is accumulated on the stack for a possible unwind.

**Engine failure** comes in two distinct kinds: either the pipeline couldn't even start (e.g. the lock was busy), or a specific stage at position N failed — the command that invoked the engine tells them apart, because in the first case no stage number exists at all.

**Pipeline validation, before the first call.** Each stage can declare what it `requires` from the shared context and what it `provides` into it (by type); before running, the engine walks the whole list once and checks every stage's requirements are satisfied by what's already in the context plus what earlier stages provide — a missing dependency fails fast, before any stage actually runs, instead of surfacing as a confusing panic or `None` deep inside some later stage. This check is uniform across every command (both the exclusive and concurrent run paths). As of this writing no stage declares real requirements yet (stage bodies are still unwritten), so the check is currently a no-op everywhere — it activates automatically as stage bodies start declaring their real dependencies.

---

## 6. FFI and boundaries

`lib` holds all the logic and a stable C-ABI. CLI and (in the future) GUI are thin frontends: they only parse input and render events. Decoders are plugins that `lib` loads. Below are only the flows, without structs: the code behind the FFI lives its own life.

```
CLI ─┐
GUI ─┼──(C-ABI)──▶ lib ──(dlopen)──▶ decoders/*   (format parsing + resolve)
…   ─┘             │
                   └──(calls)───▶ composefs        (repo / image / mount / boot)

lib ──(hook callbacks)──▶ CLI/GUI                    (progress, events, /etc conflicts)
```

What travels along the arrows:

- **frontend → lib:** a command with arguments (what to do) + a cancel token.
- **lib → decoders:** the path to a package; back — files, metadata, dependencies.
- **decoders:** dynamic `.so` by default (add a format = drop a plugin); optionally — a static single-binary build for distro maintainers. Loading and calling them is always owned by `lib`, not the CLI.
- **lib → composefs:** primitives (commit an image, mount, prune, write boot entries).
- **lib → frontend (hooks):** operation progress, confirmation events, `.upac-new` conflicts.

**Boundary rule:** "touches state / needs root / must be atomic" → `lib`; "presents or collects input" → frontend. CLI and GUI are equal thin frontends over one FFI, with no logic duplicated.

---

## 7. Planned modules

A design guide, refined as we go.

**`lib/`** — core and FFI (actual module layout, kept in sync as it's built):

- `export` — the C-ABI: entry points for all 15 commands, ABI version, cancel, response freeing.
- `orchestrator` — the generic engine (§5.9–§5.11): `Stage`/`ConcurrentStage`, `Cursor`, `RollbackGuard`, and two orchestrators behind one `Orchestrator` trait — `SequentialOrchestrator` (linear, holds the system lock) and `ParallelOrchestrator` (concurrent stages, used for §5.8 hooks).
- `mutated` / `unmutated` — the 15 commands themselves, one submodule per command, each just assembling its own stage pipeline via `orchestrator`.
- `scripts` — §5.8: the hook TOML format (`HookFile`), primitives (`Primitive`/`TouchFile`/`MoveFile`/`CreateSymlink`, each `impl Step { execute, rollback }`), native trigger matching (`Operation`/`Timing`). `HookStage::run()` is fully wired for native triggers: get-or-build the shared `tokio` runtime off `Context`, load + verify + parse hook files (`load_hooks`, via `upac-pki`), filter by `NativeTrigger`, run the matched hooks concurrently through a `ParallelOrchestrator` (`HookFile` itself `impl ConcurrentStage`, executing its `steps` and respecting `critical`), and it's wired into all 6 mutating commands' pipelines (Pre/Post around each). Still open: the compatibility-table matching against a decoder's native trigger names (`HookFile.triggers` is parsed but not yet consulted) and `priority`-based conflict resolution.
- `plugin` — decoder loading; concretely just the `decoder` submodule today (dlopen, ABI-version check, `decode`/`match_triggers`) — the `plugin` parent is reserved for other plugin kinds later, none exist yet. Also holds `manifest` (`DecoderManifest`, `load_decoder_manifests()` — reads the declarative `/etc/upac.d/decoders/*.toml` descriptors, no `.so` scanning/probing) and `triggers` (`build_trigger_table()` — builds a decoder-scoped native-trigger→hook table out of loaded `HookFile`s, resolving `priority` conflicts with a hard error). Not yet wired to anything — still needs the actual call site, tied to the not-yet-written install/update stage bodies.
- `composefs` — access to the composefs repo (`objects` / `images` / `streams`).
- `deploy` — deployment staging (§5.3): sysroot, mounts, the one-shot entry.
- `database` — the package DB (redb) inside the image; built via a custom in-memory `StorageBackend`, read at runtime with `ReadOnlyDatabase` from the file in the image.
- `types` / `errors` — shared domain types, per-command error enums, `CommonError`.

Not started yet: `etc_merge` (the 3-way `/etc` merge, §5.1), `boot` (boot entries, one-shot/confirmation/rollback, §5.2; uses `composefs-boot`, grub via `blscfg` — no separate boot plugins, only BLS-capable bootloaders by design), `gc` (retention policy and pruning, §5.5), and the actual dependency-graph resolution on `lib`'s side (the decoder only reports a package's raw dependency list today, via `decode` — nothing walks the graph yet, and there's no network layer to fetch resolved packages either).

Build-time config (table names, deploy paths, the lock address) is `Lib.toml` + `build.rs`, generating plain constants — not a separate `derive-static` crate; that idea was superseded.

**`cli/`** — a thin frontend:

- `args` — argument parsing;
- `commands` — one module per command (install / remove / update / rollback / gc / …);
- `render` — rendering progress, events and conflicts from hooks;
- `ffi` — binding to the core's C-ABI.

**`decoders/`** — plugins (`.so`, one per format: alpm / deb / rpm / xbps). Loaded and called by `lib` (the `decoder` module, inside a `plugin` parent reserved for other plugin kinds later), not the CLI. Which `.so` to load for which format is resolved from a declarative manifest (`/etc/upac.d/decoders/*.toml`, see §5.8), not by scanning the `decoders/` directory itself — `lib` never probes a `.so` just to ask what it is. By default — dynamic `.so`; optionally distro maintainers build statically (a single binary + linked-in decoders). Each exports `decode` (package → files, metadata, and dependencies, all in one call — `resolve` was folded into it, since the decoder already has everything parsed by then) and `match_triggers` (the §5.8 compatibility-table matching).
