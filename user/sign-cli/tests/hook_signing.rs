// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::{remove_dir_all, write};

mod support;

use support::{bin, generate_signing_chain, scratch_dir, sign_hook, verify_hook};

#[test]
fn full_workflow_signs_and_verifies_a_hook() {
    let dir = scratch_dir("full-workflow");
    let chain = generate_signing_chain(&dir);

    let hook_path = dir.join("hook.sh");
    write(&hook_path, "#!/bin/sh\necho hook\n").unwrap();

    let signature_path = dir.join("hook.sig");
    assert!(sign_hook(&chain, &hook_path, &signature_path));
    assert!(verify_hook(&chain.root_cert, &hook_path, &signature_path));

    let _ = remove_dir_all(&dir);
}

#[test]
fn verify_fails_on_tampered_hook() {
    let dir = scratch_dir("tampered-hook");
    let chain = generate_signing_chain(&dir);

    let hook_path = dir.join("hook.sh");
    write(&hook_path, "#!/bin/sh\necho hook\n").unwrap();

    let signature_path = dir.join("hook.sig");
    assert!(sign_hook(&chain, &hook_path, &signature_path));

    write(&hook_path, "#!/bin/sh\necho tampered\n").unwrap();

    assert!(!verify_hook(&chain.root_cert, &hook_path, &signature_path));

    let _ = remove_dir_all(&dir);
}

#[test]
fn verify_fails_against_unrelated_root() {
    let dir = scratch_dir("unrelated-root");
    let chain = generate_signing_chain(&dir);
    let unrelated_dir = scratch_dir("unrelated-root-other");
    let unrelated_chain = generate_signing_chain(&unrelated_dir);

    let hook_path = dir.join("hook.sh");
    write(&hook_path, "#!/bin/sh\necho hook\n").unwrap();

    let signature_path = dir.join("hook.sig");
    assert!(sign_hook(&chain, &hook_path, &signature_path));

    assert!(!verify_hook(&unrelated_chain.root_cert, &hook_path, &signature_path));

    let _ = remove_dir_all(&dir);
    let _ = remove_dir_all(&unrelated_dir);
}

#[test]
fn sign_hook_fails_on_missing_key_file() {
    let dir = scratch_dir("missing-key");
    let chain = generate_signing_chain(&dir);

    let hook_path = dir.join("hook.sh");
    write(&hook_path, "#!/bin/sh\necho hook\n").unwrap();

    let status = bin()
        .arg("sign-hook")
        .arg("--hook")
        .arg(&hook_path)
        .arg("--key")
        .arg(dir.join("does-not-exist.pem"))
        .arg("--cert")
        .arg(&chain.signing_cert)
        .arg("--signature")
        .arg(dir.join("hook.sig"))
        .status()
        .unwrap();
    assert!(!status.success());

    let _ = remove_dir_all(&dir);
}
