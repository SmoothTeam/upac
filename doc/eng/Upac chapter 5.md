<!--
SPDX-FileCopyrightText: 2026 JustPav

SPDX-License-Identifier: CC-BY-SA-4.0
-->

## **§5.** Operating mechanisms

A separate description of the mechanisms implemented by the core (`lib/`).

### **§5.1** The mechanism for merging configs in the `/etc` directory.

In a booted system, the `/etc` directory is an `overlay` consisting of `lower` = the immutable etc-digest, in `ro` mode, and `upper` = edits not recorded by the system, in `rw` mode.

The `/etc` directory is versioned content-addressably: every file system snapshot = an `etc-digest` (see **§5.7**). The job of the merge mechanism is, whenever the `/usr` directory changes, to build a new `/etc` directory: carry over the user's config edits, pull in the packages' new standard configuration files, and seal the result into the first `etc-digest` snapshot of the new `/usr` directory.

The mechanism lives in the library and runs at the `merge` stage, before the new snapshot becomes bootable.

**Three inputs (3-way):**
- **base** — the current `/etc` files corresponding to the current `/usr` files that the booted system was built from;
- **new** — the new `/etc` files of the `/usr` directory being deployed, which include the packages' new standard configuration files;
- **live** — the current state of the user's `/etc` configuration files, combining the `working_etc` sealed into the snapshot and the current running system's user edits not yet sealed into a snapshot.

**Per-file classification:**
- If the user did NOT modify the file, i.e. `live == base`, the result is the package's **new standard configuration file**;
- If the user edited the file AND the new default matches the old one, i.e. the package did not change the file, the user's version of the file is kept;
- If the user edited the file AND the new standard configuration file has changed, creating a conflict, the user's version of the file is carried over, and the new file with the standard settings is placed alongside it as `<file_name>.upac-new`, BUT it is excluded from future classification — it is not a *"user file."*

**Conflicts described in the last item are resolved via a special hook, and do not block the operation from proceeding.** The user is informed about the package's new standard `.upac-new` configuration files through the message-hook call mechanism, passed on to the CLI.

**The mechanism's result** is sealed into a new `etc-digest` snapshot, which becomes the `working_etc` of the new system snapshot; the live overlayfs upper layer of `/etc` starts out empty. Unchanged files are deduplicated by composefs at the object level, so `etc-digest` is a complete snapshot of `/etc` without duplicating content.

On the `upac commit new` command, this mechanism seals the current state of `/etc` without changing the files in the `/usr` directory — the new `etc-digest` runs under the same `/usr`.

### **§5.2** The mechanism for booting a new system version and rolling back to the previous working version if the boot fails.

The system rollback is built on a one-shot selection of the new boot option and a late confirmation that the startup succeeded. There is no separate counter of boot attempts — an image that fails to boot on the first try, for whatever reason, is automatically rolled back to the previous successful boot option.

**The one-shot selection mechanism.** The bootloader/firmware has a pair of boot options: a *"one-shot boot option / persistent boot option."* For example, for UKI-direct boot this is `BootNext` / `BootOrder`; for the systemd-boot bootloader — `LoaderEntryOneShot` / `LoaderEntryDefault`; for grub — `grub-reboot` / the persistent option in the config. The firmware/bootloader uses the one-shot variable exactly once on any boot, which is precisely why it is, by itself, a single-attempt auto-rollback.

**Staging and booting:**

1. For snapshot D' a boot entry is written with `composefs.digest=D'`. For UKI-direct boot, into the inactive `upac-to.efi` slot; for bootloaders compatible with BLS config, into the configuration file, via the `BootconfigParser` mechanism built into composefs;
2. After the reboot: the bootloader boots D' once, and the one-shot boot option is removed. Initramfs mounts the digest from the cmdline (composefs overlay), followed by pivot and PID1.
3. The system reaches a fully successful boot — a late hook / init unit makes D' the persistent boot option and marks the **pair as working**: it updates `working_etc` for the current `/usr` (see **§5.7**);
4. If confirmation did not happen, i.e. for whatever reason the system did not reach its final state — the one-shot variable has already been removed from the bootloader's/firmware's memory, so the next boot starts the persistent boot option, i.e. the previous system snapshot. This is the auto-rollback;

**D' is the usr-digest, not a composite pair.** `composefs.digest=D'` carries precisely the `usr-digest` — the same value that names the `state/deploy/<usr-digest>/` directory, and that `open_tree()` (see the composefs module, **§7**) accepts directly. This is the same way the booted system finds out *"which system snapshot is currently active"* without a separate pointer file on disk. The `etc` directory from the pair is deliberately **NOT** present in the cmdline — as soon as `usr-digest` is known, `working_etc` is read from the file in its `state/deploy/<usr-digest>/meta.json` directory (see **§5.7**), which carries information about the currently confirmed `etc-digest`.

**Why does a kernel parameter unrecognized by the kernel, such as `composefs.digest=`, even survive into `/proc/cmdline`?**

`/proc/cmdline` is not a filtered list of parameters that only the kernel understands, but the raw, untouched string that the bootloader or the UKI passed to the kernel. When the kernel's argument parser encounters an unfamiliar parameter, it does not discard it; instead, it prints *"Unknown kernel command line parameters ..., will be passed to user space"* and leaves the string as-is, for `/proc/cmdline` and for PID1's own cmdline.

This is standard, documented kernel behavior, and many userspace programs rely on it — it is exactly how `systemd.*`, dracut's `rd.*`, `luks.uuid=`, and OSTree's own `ostree=` already work: none of them are kernel parameters either, all of them are userspace-only, all of them survive the same way.

**Rollback echelons.** A description of each boot-failure level and what that level catches:

1. If the kernel or initramfs failed to start — the firmware/bootloader itself falls back to the persistent boot entry;
2. If the kernel and initramfs started, but PID1 failed to come up — no boot-success confirmation arrives, and the next boot rolls back;
3. If PID1 came up successfully, but services/network/GUI are dead — the user can simply restart the system or invoke the `upac commit rollback` command;

If the system formally reaches a working state and confirms it, but some subsystems or tools failed to come up or are working incorrectly — a manual rollback is available: `upac commit rollback` from the live system, or the firmware/bootloader menu.

**Deliberate limitations of this mechanism:**

- Only one attempt: a broken atomic image is deterministically broken, so repeated attempts to boot it are pointless;
- Auto-confirmation only proves that *"the boot successfully reached a certain point,"* not that *"things are good for the user"* — deeper breakage is rolled back manually via the `upac commit rollback` command;
- A plain *"hang"*: PID1 is alive but frozen, with no panic and no reboot, requiring a manual restart through physical interaction so that the one-shot variable takes effect.

### **§5.3** Staging a system snapshot for boot (stage).

Input: a ready-made image D', already sitting in the repository directory: `images/D'`.
Output: a system snapshot ready for a one-shot boot.
It links operations (see **§5.4**) with booting (see **§5.2**).

1. Merging the `/etc` directory (see **§5.1**): merge seals the `etc-digest` directory for D' and declares it `working_etc`. The layer of undocumented user edits, upper (`etc-upper/`), starts out empty;
2. Persistent partitions/directories (`/var`, `/home`) — real, mounted as-is, untouched;
3. Writing the boot entry with `composefs.digest=D'`:
   - UKI-direct — assemble and sign the UKI, write it into the inactive `upac-to.efi` slot;
   - Manager — `BootconfigParser` writes a BLS conf into `loader/entries/`;
4. Set D' as the one-shot entry for the next boot (see **§5.2**): UKI-direct — `BootNext` on the slot; manager — `LoaderEntryOneShot` / `grub-reboot` and other options specific to a particular manager's implementation.

After this: reboot, and item **§5.2**.

### **§5.4** The mechanism for file system operations: add, remove, update, rename, and so on.

All operations follow one form:
1. Change the file tree;
2. Commit the new image;
3. Hand the image off to staging (**§5.3**).

The old image is not touched until the switchover (the atomicity property). This is where the decoders and the resolver operate, and where the package DB is written.

The general operation pipeline:

1. Build the new file tree from the current one;
2. Commit the file tree as the new image D' into the repository (`objects/` + `images/D'`), embedding the package DB inside the image;
3. Pass D' to image staging (see **§5.3**);
4. Light cleanup of old, unused images (see **§5.5**).

### **§5.5** Garbage collection.

It has two levels: deploys (what to keep) and objects (what to sweep away). The retention policy is set by the user. The object-sweep engine is composefs.

**Immutable pins** (never removed):

- The active (booted) deploy;
- The rollback target (the persistent or previous deploy);
- A staged but unconfirmed deploy (one-shot boot).

Plus the user's manual pins (manually pinned deploys) and the last N within the depth set by the user.

**Cleanup triggers:**

1. **Light deploy cleanup by an internal operation stage** after every operation that changes the file system: drop the image's ref and remove `state/deploy/<D>/` for deploys outside the retention policy. The number of disk write operations is small, so the operation is cheap, and the pins hold onto what's needed;
2. **Heavy object cleanup is only run manually**, via the `upac package gc` command: the program walks the `objects/` and `streams/` directories and sweeps away unreachable ones, i.e. objects that nothing references, using the composefs `ObjectCollector` mechanism.

### **§5.6.** The mechanism for creating and deploying OCI (planned for development).

In this paragraph, OCI is a portable image-artifact format, not a network protocol.

**Directions of image entry:**

- **Import** — take an OCI image and deploy it as the host system;
- **Export** — produce a portable OCI image artifact from a deploy, creating a ready-made *"reference copy."*

**Image delivery options:**

1. **From an external repository** — the same networking subsystem used for delivering packages is used here. Applied by default;
2. **From a local file, via a command** — using `--file <image>`, a local file is taken and deployed directly.

Only these delivery paths are used for deploying an image to multiple machines. There is no separate mechanism yet for deploying to multiple devices. Implementation as a plugin is possible.

Deploying an image relies on the `composefs-oci` mechanism: `create_filesystem` (layers → image), `generate_boot_image`, `pull_image`.

### **§5.7** The mechanism for the deployment history of images and rolling back N deploys.

The deployment history is stored NOT inside the image (which would break content-addressability), but on the read-write (`rw`) partition. The source of truth is the `state/deploy/<usr-digest>/` directories themselves; there is no separate log.

**Two axes of history preservation:**

- **The `/usr` directory** — a linear deploy history. Each distinct `/usr` directory = one record, whose key is the `usr-digest`. `seq` is the order in which records are born: monotonic, one per digest. It has a built-in marker for finding the array element following it, in the form of `state/next-seq`. Re-arriving at an existing `usr-digest` **switches** to its record as the current one, rather than creating a duplicate — so this `/usr`'s `/etc` sub-history stays intact when returning to an old `/usr`. A record carries its own **commit message**: `subject` — short, required, and an optional long `message` — the commit message of the operation that produced this `/usr`;
- **The `/etc` directory** — a sub-history within `/usr`. The record's `meta.json` carries `etc_history` — an ordered list of records of the type `{etc_digest, subject, message}`, created under this `/usr`. On `/usr` change and on the `upac commit` command, see **§5.1**. Each record carries its own `subject` and an optional `message`. The first record is created automatically when `/usr` changes (see **§5.1**), inheriting the subject and message of the `/usr` event itself — later, explicit `upac commit` calls get their own, independent subject and message.

**The active deploy** is the booted `composefs.digest` and the boot default. It is **NOT** `max(seq)`: after switching to an old record, its `seq` stays the same, unchanged.

**Rollback variants:**

- **Emergency (`/usr`)** — to every N-th existing deploy in `seq` order (counted by actual presence, **not** by `seq−N` arithmetic). Burned numbers are allowed; we simply skip them. The **pair** is restored: the target `usr-digest` + its `working_etc` (the last confirmed commit of the sub-item) — thanks to this, config edits are not entirely lost;
- **Config (`/etc`)** — `upac commit rollback --etc` to the previous `etc-digest` from the `etc_history` of the current `/usr`. The same rank-based mechanics apply, with its own retention depth.

The authority and source of truth for the rollback mechanism's operation is **`seq`**. Timestamps, in the form of **`timestamp`** in `meta.json`, are for display only (for example, `upac commit history`), since clock drift or a clock change could easily break the history and make the system rollback impossible.

Connection with the cleanup mechanism (see **§5.5**): the retention depth of an image reference on each axis (configs or system files) must be **inclusive** with respect to the number N.

### **§5.8** The mechanism for hooks and for locating decoders (the operation of pre/post triggers).

**Hook** — a declarative, **signed** file: it describes the trigger itself, the hook's priority over other hooks with the same trigger, and a composition of **primitives**;
**Primitive** — a closed set of low-level actions built into `lib`. For example, running a process, touch (creating) or move (moving) a file, and so on — the only thing that requires editing `lib`'s code to add any new primitives;
**Signature** — protection against an arbitrary, unsigned hook file, since primitives are privileged enough within the permissions system that an unsigned file cannot be trusted.

**Correspondence table.**

A hook file separately carries a table: for decoder `D` (see **§6** — a package format plugin: deb, rpm, etc.), this hook covers the trigger name that is NATIVE to that package format (for example, deb has the `update-mime-database` trigger, which the decoder will translate into the universal format). Compatibility with other trigger conventions is also described in the file, while the decoder is simply handed the ready-made correspondence table, from which it determines which ones need to be executed and which are not satisfied, passing everything on to the calling side.

**Hook file priority.**

**Priority** is an ordinary signed integer (default 0), used ONLY to resolve a conflict when several different hook files claim the same native trigger name (the same key `k` in the correspondence table). In a conflict, the higher `priority` wins; however, if there is a tie, this is an unresolvable conflict, and `lib` immediately returns a critical error and cancels the operation. Automatic selection is not provided, by design. Unmatched entries (the hook's native trigger simply does not exist in a particular package) do not need any separate reporting at all — this is a normal, expected outcome for most hooks on most packages, not an error. `priority` does not define any execution order, since all trigger hooks that need to run execute concurrently (in parallel).

Whether a non-fatal warning should still be surfaced through `MessageHook` for any of these cases (a conflict, or a hook that structurally can never match anything, or other cases that come up later) — remains an open question for now.

**Hook file criticality.**

**Criticality** — a field (`critical = true/false`). It marks hook files that are critical to the operation, whose execution failure causes the entire operation to fail and be canceled.

**Division of labor:**

- **`lib`** — the only party that reads hook files from disk, verifies the signature, and parses the composition of primitives and the correspondence table. It executes the composition through its own primitives;
- **The decoder (plugin)** — receives from `lib` the correspondence table already as a ready-made key:value map **for its own `D`** (i.e. the deb package format does not receive other formats' trigger entries, for example rpm's), where the key is the decoder's native trigger name and the value is our hook name. The decoder itself matches it against the native trigger names it read from the package (for example, the deb decoder reads the package's `Triggers-Interest` itself), and hands back to `lib`, through FFI, the list of hooks to execute (the required values);

**Decoder resolution.**

Decoders are located through declarative, signed TOML manifests in `/etc/upac.d/decoders/` (each decoder gets exactly one `format`, `extensions`, `library`): `format` is the canonical identity of the package format, the same string as the key `D` in the hook correspondence table; `extensions` lists the file variants in which this format is actually shipped (for example, alpm packages have the file extension `pkg.tar`/`pkg.tar.gz`/`pkg.tar.xz`/`pkg.tar.zst`); `library` names the `.so` that needs to be opened. The plugin library itself is opened lazily, that is, only and exactly when the file is required for execution (unpacking the format). A duplicate `format` between two manifests produces a hard error at manifest-loading time, the same logic as for the priority tie above.

**Hook file format and signature.**

A hook file is TOML, and lives in `/etc/upac.d/hooks/` (the path is baked in as a constant from the `lib.toml` file at the library's build time). The signature is built as a chain of 2 trust levels: the root CA (an offline key, which only signs the next level), then a signing certificate for the trust domain, which directly signs the hook file's bytes; the `.sig` file itself carries both the signature and the entire signing certificate — verification is self-contained.

**The root is a configurable file**, not baked into the compiled version of the program: a distro/OEM, or the user, plugs in their own root without rebuilding `upac`.

**The signature scheme** — the Ed25519 encryption algorithm on top of X.509 certificates.

**Execution model.**

Running hooks is asynchronous, but entirely inside `lib`: the FFI remains fully synchronous. The scope of application is only the concurrent execution of N independent hooks within a single stage of a command's execution.

### **§5.9** The mechanism for canceling a failed operation.

It works through the `CancelToken` (cancel token) mechanism, operating on the principle of an atomic flag, which is created by the calling side (CLI/GUI) and passed into `lib` as a pointer with every request.

**`Lock`** — a mutual-exclusion mechanism between rw operations, working on the basis of a bind to an abstract Unix address, strictly known and fixed in `lib.toml`, shared by all rw calls).

### **§5.10** The mechanism for reporting operation progress.

It uses the same message-delivery channel as in item **§5.9**. It contains:
- `stage` - the stage's ordinal number, in `u16` format;
- `phase` - which sub-step within the specific stage, in `u16` format;
- `subject` - the object string, identifying what is currently being worked on (a file, a hook, or another object);
- `current`/`total` - an element counter. `0` if not applicable/unknown.

The `MessageHook::send` mechanism accepts a single, self-describing parameter instead of the former separate event/data — there is nothing to unpack separately.

### **§5.11** The stage orchestration mechanism.

The mechanism by which commands are actually executed: a linear list of stages plus an engine that walks through them — both hook channels from items **§5.9** and **§5.10** are also plugged in here.

**A stage is always a flat structure.**

A single stage performs exactly one atomic unit of work per call. Each call itself decides what happens next — moving forward to the next stage, repeating itself (for example, processing one more file from an already-started list), or jumping BACK to an earlier stage by its TYPE (not by numeric index, so that changing the stage list doesn't break things). Through such a backward jump, a group of several stages (for example, *"verify package → unpack → register"*) can be repeated as a single unit — on a jump, the engine looks for the nearest matching stage by type, going backward from the current position. If no such stage exists, this is a pipeline-assembly bug, not user input, and it is returned as an ordinary error, aborting the operation.

**Every stage call brings its own rollback.**

A stage does not accumulate state between calls — on every call it creates its own, self-contained rollback object for the changes made on disk, carrying exactly the data needed to undo exactly what THIS call did (even if the same stage was called many times before with different data — each call remains independent). If a stage has nothing to roll back this time, it must still return such an object, simply in *"empty"* mode: this is guaranteed at the compiler level (an *"empty"* rollback constructor).

**The engine** holds a linear list of stages. Exclusivity for the difference between rw and ro commands is achieved by choosing the launch METHOD: in rw mode, it holds a system lock file for the entire duration of the run, whereas the other mode does not create it at all. On any failure (a stage error, a cancellation, a backward jump that finds nothing), it unwinds every rollback object accumulated up to that point, in reverse order, without stopping if one of them itself fails to roll back — it is skipped, since the mechanism aims to roll back everything it can. Every successful stage call hands the engine two separate things at once — the progress constructor (see **§5.10**, which the engine itself created and passed to the stage before the call) and its own rollback object.

**Engine failures:**

- The pipeline failed to even start (for example, the lock file indicates another rw operation is running);
- A specific stage, number N, failed — the command that invoked the engine distinguishes between these two, because in the first case there is simply no stage number at all.

**Pipeline validation, before the first call.**

Every stage can declare what it `requires` from the shared context and what it `provides` back into it (by type). Before running, the engine walks the entire list once and checks that each stage's requirements are satisfied by what is already sitting in the operation's context, plus what earlier stages have provided — a missing dependency fails immediately, before any stage actually runs, rather than surfacing somewhere deep inside a later stage. The check is uniform across all commands (it also accounts for both the exclusive and the concurrent launch paths).
