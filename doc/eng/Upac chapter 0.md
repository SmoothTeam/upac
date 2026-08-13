<!--
SPDX-FileCopyrightText: 2026 JustPav

SPDX-License-Identifier: CC-BY-SA-4.0
-->

# **UPAC — the unified project document.**

Project document.
Project branch: **`lib-rs`**, crate `lib-rust/`.

## **§0.** Introduction and definitions.

This paragraph defines the terms used further in the text without explanation.

### **Item 1.** Basic concepts.

- **File** — a named area of memory in which a certain amount of information is stored;
- **File extension** — a suffix at the end of a file name, separated by a dot, which tells the operating system and programs the format of the data contained in the file and determines which application should be used to open or process it;
- **Directory (folder)** — a container for files and other directories;
- **Path** — the address of a file or directory that specifies its exact location in the file system.
- **File system (FS)** — the order and rules by which the operating system organizes, writes, finds, and stores files on a disk or flash drive, for example EXT4, Btrfs, NTFS.
- Ref (reference / link) — an exact address or pointer in the file system that references a specific file, state, or data slice, allowing it to be accessed without duplication;
- **Disk / block device** — a storage device (physical or virtual);
- **Partition** — a dedicated part of a disk on which a single FS resides;
- **MBR (Master Boot Record)** — an outdated boot sector structure format, in use since 1983, which stores the primary bootloader code and the partition table, limiting the drive to a maximum of 4 primary partitions and a capacity of up to 2 TB;
- **GPT (GUID Partition Table)** — a drive partition table standard that allows creating many partitions and working with disks larger than 2 terabytes;
- **Mounting (mount)** — the *"attaching"* of a partition's FS to a point in the directory tree, after which the contents of the file system become available at that path;
- **Kernel** — the main program of the operating system, which acts as an intermediary between applications and the computer's hardware, distributing resources (memory, processor, devices);
- Userspace — the area of memory in which all regular programs and applications run: browsers, text editors, the system interface, isolated from direct access to the memory area used to manage hardware, for the sake of the security of the whole system;
- Kernelspace — a protected area of memory in which the operating system kernel and drivers run; it has full and direct access to the processor, RAM, and all of the computer's hardware;
- **Initramfs** — a temporary file system in RAM (random-access memory) that contains the drivers and programs needed for the kernel to find, decrypt, or prepare the main disk and boot the main system from it;
- **Unified Kernel Image (UKI)** — a single executable file in EFI format that combines the Linux kernel, the initialization image (initramfs), the kernel command-line parameters, and the EFI bootloader into one indivisible module;
- **Firmware** — basic software written into the non-volatile memory of a chip on a motherboard or device — a graphics card, an SSD, a controller — serving as the main bridge between the physical hardware and the operating system, initializing components at power-on and providing low-level instructions for controlling them;
- **BIOS (Basic Input Output System)** — an outdated 1975 firmware standard that cannot read file systems and reads boot code from the disk's first 512-byte sector (MBR), which makes it poll hardware sequentially, work slowly, and be limited to drives of up to 2 TB;
- **UEFI (Unified Extensible Firmware Interface)** — a modern 2005 firmware standard capable of fully reading the FAT32 file system on a special partition (ESP) and launching .efi files from it directly, which allows it to work with GPT disks of any size, poll hardware in parallel, and verify code signatures via Secure Boot;
- **ESP (EFI System Partition)** — a system partition, usually with a FAT32 file system, that is read directly by the UEFI firmware when the computer is powered on. It stores bootloader files in .efi format, Secure Boot keys, and the startup components needed for the first phase of the hardware boot.
- **Secure Boot** — a UEFI firmware feature that blocks the execution of any third-party code at PC power-on by verifying that code's signature;
- **Bootloader** — a program launched by the board's firmware (UEFI/BIOS) to give the user a choice of OS, load the required parameters, and start the kernel from disk;
- **Cmdline (kernel command line)** — a text string of instructions and settings that the bootloader passes to the kernel at startup to determine the system's operating mode (for example, to specify the root disk, enable debugging, or disable a driver);
- **Hash function** — an algorithm that converts any amount of data into a sequence of fixed length. Identical data always produces an identical code, while any different data produces, for all practical purposes, a guaranteed unique one;
- **Hash (digest)** — a unique short sequence obtained by processing data with a hash function. If the data has not changed, the hash is always identical; if even one element of the sequence changes, the resulting code changes completely;
- **Package** — an archive of a program's files and its metadata: a list of files the system needs for it to work, and installation instructions, which the package manager installs, updates, or removes as a single unit;
- **Repository** — a package store in which program files sit alongside their signatures and a single index/catalog. This lets the package manager automatically find the required versions, verify their authenticity, and download the correct dependencies;
- **Digital signature** — an encrypted stamp from the developer, attached to a package: it guarantees authorship, i.e. the fact that the current code belongs to a specific party, and integrity, i.e. the absence of any changes to the program since its creation by the developer;
- **Package manager** — a program that automatically downloads, installs, updates, and removes programs from a repository, and that also finds and installs, by itself, all the programs required for them to work (dependencies);
- **Atomicity** — the property of indivisibility: an operation either applies in full or does not apply at all, with no intermediate, half-applied states;
- **Immutable system** — an operating system architecture in which system files are protected from modification while the system is running, and any updates are applied atomically and placed alongside the existing state, allowing an instant rollback to the previous working state at any time;
- Persistent (or persistence) — the property of data or settings being preserved even after the computer is turned off, the system is rebooted, or the program is closed;
- **POSIX (Portable Operating System Interface)** — a family of IEEE and ISO standards defining a unified programming interface (API), system utilities, and command-line behavior for Unix-like operating systems;
- **FHS (Filesystem Hierarchy Standard)** — a standard defining the structure, naming, and purpose of the main directories in Unix-like operating systems;
- Script — a small program or sequence of commands, most often implemented as a text file of instructions, that the system or another program executes step by step to automate everyday tasks;
- Hook — an automatic handler that runs at a specific point during a program's operation, for example before or after a package is installed, to execute a given command or script;
- Trigger — an automatic condition-signal that, when a specific event occurs, immediately launches a predefined action or program;
- DB (database) — an organized store of information on disk from which programs can quickly find, add, modify, and safely save the data they need;
- Commit — a saved point, or *"snapshot,"* of state in a version control system, for example Git, which records all changes to files at a given moment and allows returning to them if necessary;
- Deploy (deployment) — the process of transferring, configuring, and launching a finished program, system image, or file tree onto a real working server or device, where the deployed content becomes available for use;
- CLI (command line interface) — a text-based way of controlling a program, in which the user enters commands into a terminal from the keyboard instead of clicking buttons with a mouse.
- Pivot (pivot root) — the operation of switching a running system from a temporary initial disk, for example initramfs during boot, to the main, real disk partition, which becomes the new root `/`;
- PID 1 (process ID 1) — the first and most important process launched by the Linux kernel at power-on, for example systemd, responsible for starting all other programs and managing the entire operation of the system until it shuts down;
- Unit — a basic control element in a system manager, for example systemd, representing a text file of settings for starting and controlling a specific service, task, device, or mount point;
- Engine — a base program or subsystem that performs all the complex internal work — calculations, logic, data processing — so as to simplify development and avoid recreating these functions from scratch for every new application;
- OCI (Open Container Initiative) — an open industry standard for the container image format and its runtime environment, guaranteeing that containers run identically on any platform and engine (for example Docker, Podman, Kubernetes/CRI-O);
- CA (certificate authority) — a body or service that issues and signs digital SSL/TLS certificates. A CA's signature confirms that a public key genuinely belongs to the specified domain, server, or user;
- Root CA — the topmost, principal level in the chain of trust. A Root CA owns a self-signed root certificate that is pre-embedded in operating systems and browsers as fully trusted, and that vouches for the other certificates signed on its behalf;
- Public key — a key that is publicly available and used to encrypt data or verify a digital signature. It can be freely given to anyone;
- Private key — a secret key known only to its owner, used to decrypt data or create a digital signature. Loss or leakage of this key compromises the entire protection.

### **Item 2.** Definitions of the folders used by the system.

- **`/`** — the topmost directory in the file system hierarchy (FHS), from which all other folders and mounted disks branch off;
- **`/usr`** — the system folder containing executable files (programs): binaries, libraries, and other system resources. On immutable systems this directory is mounted read-only (ro), and it is updated entirely, as a full replacement of one version with another, which is exactly what allows the whole operating system to be rolled back instantly on failure;
- **`/etc`** — the directory of configuration files. On immutable systems it remains writable so the user can change settings, and when the /usr layer is updated or rolled back its contents are automatically merged to keep the settings consistent;
- **`/var`** — the directory for mutable data that programs create and update while running: logs, databases, cache, print queues. This folder is always open for writing and is fully preserved across any system updates or rollbacks;
- **`/home`** — the directory for users' personal files: documents, downloads, projects, and their individual program settings. This folder is fully isolated from the OS files, is writable, and is not affected by system updates or rollbacks;
- **`/boot` (or `/efi`)** — a directory or separate partition holding the files needed for the operating system to continue booting: the compressed kernel file (vmlinuz), the RAM disk initialization image (initramfs), or unified boot executable images (UKI);
- **`/sysroot`** — a temporary mount directory for the physical disk at an early stage of boot, while initramfs is running, or in atomic systems. A disk partition is mounted here so that the needed directories can subsequently be selected to boot the required version of the operating system, turning it into the system root `/`;

### **Item 3.** Definitions of project-specific terminology.

- **Content-addressed storage (CAS)** — a data storage method in which a file's address is determined by the hash of its content, which provides automatic deduplication of identical files, guarantees protection against tampering, and allows data integrity to be verified instantly;
- **Image** — an immutable (ro) snapshot of a file system at a specific version, deployed as a single unit and guaranteeing an identical OS state on any device;
- Distribution (distro) — a ready-to-use operating system, built on top of a kernel with the addition of a system environment, utilities, system services, basic programs, and a package manager;
- **Ref** — a human-readable name that points to a specific image hash and is updated whenever a new version of the system image is released;
- **OverlayFS (lower / upper)** — a virtual file system that combines a read-only layer (lower, the base image) and a writable layer (upper, changes), presenting the user with a single folder in which system files stay untouched while any edits are stored separately;
- **fs-verity** — a file integrity protection mechanism built into the Linux kernel that makes a file immutable (ro) and, on every read, verifies its blocks via a Merkle tree, instantly blocking access at the slightest corruption or tampering of the data;
- **One-shot (one-shot / BootNext)** — a single-use UEFI boot entry that is activated for exactly one system startup, allowing a new update to be tested and automatically rolled back to the previous working version on failure;
- **Three-way merge (3-way merge)** — an algorithm for automatically merging text files that compares two changed versions against their common ancestor in order to preserve the user's edits and apply system updates without conflicts;
- **`base`** — the original configuration file from the previous (current) version of the system, which serves as an untouched reference for computing the changes made by both the developers and the user;
- **`new`** — the fresh version of the original configuration file from the incoming system update, containing the current settings from the developers;
- **`live`** — the current configuration file in the running system, containing the user's manual edits and individual settings;
- **`.upac-new`** — the file extension assigned to new system settings when an unresolvable conflict occurs during a three-way merge, in order to keep them alongside the original file without overwriting the user's manual edits;
- **`seq` (sequence number)** — a strictly increasing counter assigned to every new deploy, which fixes the exact chronology of system versions and serves as a guaranteed reference point when automatically switching or rolling back to previous states, independent of the system clock;
- **Pin** — a protection flag in a deploy's metadata that blocks the removal of a specific system version during automatic cleanup (garbage collection), guaranteeing the preservation of the current working version, the base rollback point, or states the user has explicitly marked;
- **Rollback** — an operation that instantly returns the system to a known-working state, which can be performed either wholly, by switching the bootloader to the previous atomic deploy along with its /usr and /etc versions, or selectively, by resetting the changes in the /etc configuration layer to their original state;
- **Garbage collection (GC)** — an automatic or manual storage cleanup process that finds and removes old file system layers, files, and CAS objects no longer used by any active, current, or pinned deploy, freeing disk space without risking damage to the working system;
- Token — a compact piece of data (a string or number) that serves as a digital pass or key for confirming rights, transferring data, or securely accessing the system;
- Linking — the build step in which the linker combines compiled object files and external libraries into a single finished executable file or dynamic library, resolving references to functions and variables to their real addresses;
- API (application programming interface) — a set of rules and functions at the source code level — for example header files, function names, parameters — that defines how programs interact with each other when built into an executable file;
- ABI (application binary interface) — a set of rules at the machine code level — for example calling conventions, the size and alignment of types in memory, system call numbers — that defines how binaries and libraries built from source code interact with each other while the program is running;
- FFI (foreign function interface) — a mechanism that lets a program written in one programming language directly call functions and use libraries written in another language;
- Wrapper — an intermediate layer of code that hides complex internal machinery and provides a more convenient, safer, or language-appropriate interface;
- CLI wrapper — a program with a command-line interface that accepts commands and flags from the user in a terminal, translates them into calls to an internal library's functions, and returns the result back to the console;
- Mapping — the process of linking or converting data from one structure, format, or address space into another according to defined rules;
- Comptime (compile time) — the stage at which a program's source code is checked, analyzed, and converted by the compiler into machine code. All computations, type checks, and macros carried out at this stage happen before the program runs and do not add any load to the finished program;
- Runtime — the stage at which the compiled/built program is directly executed by the processor within the operating system;
- Race condition — a design flaw in multithreaded or concurrent systems in which the result of the program's execution depends on the uncontrolled order or timing of execution of third-party processes or threads;
- TOCTOU (time-of-check to time-of-use) — a race-condition vulnerability that arises in a system when the state of a resource, for example a file or a set of permissions, is checked at one point in time but is changed by a third-party process before the system manages to use it;
- OEM (original equipment manufacturer) — a company that manufactures parts, components, or finished devices that are then sold under another company's brand, or that are used by that company to assemble its own products;
- Ed25519 — a modern, high-speed elliptic-curve digital signature scheme (EdDSA) that uses the Curve25519 curve;
- X.509 — a widely accepted international standard (ITU-T / RFC 5280) for the structure of public-key digital certificates (PKI).
