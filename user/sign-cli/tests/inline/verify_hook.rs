// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::write;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use super::{Args, run};

fn signed_hook(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let root_key = dir.join("root.key.pem");
    let root_cert = dir.join("root.cert.pem");
    crate::commands::generate_root::run(crate::commands::generate_root::Args {
        common_name: "upac test root".to_owned(),
        key_out: root_key.clone(),
        cert_out: root_cert.clone(),
    })
    .unwrap();

    let signing_key = dir.join("signing.key.pem");
    let signing_cert = dir.join("signing.cert.pem");
    crate::commands::generate_cert::run(crate::commands::generate_cert::Args {
        common_name: "upac test signing".to_owned(),
        root_key: root_key.clone(),
        root_cert: root_cert.clone(),
        key_out: signing_key.clone(),
        cert_out: signing_cert.clone(),
    })
    .unwrap();

    let hook = dir.join("hook.sh");
    write(&hook, b"#!/bin/sh\necho hi\n").unwrap();
    let signature = dir.join("hook.sig");
    crate::commands::sign_hook::run(crate::commands::sign_hook::Args {
        hook: hook.clone(),
        key: signing_key,
        cert: signing_cert,
        signature: signature.clone(),
    })
    .unwrap();

    (hook, signature, root_cert)
}

#[test]
fn verifies_a_correctly_signed_hook() {
    let dir = tempdir().unwrap();
    let (hook, signature, root_cert) = signed_hook(dir.path());

    assert!(
        run(Args {
            hook,
            signature,
            root_cert
        })
        .is_ok()
    );
}

#[test]
fn rejects_a_hook_that_was_modified_after_signing() {
    let dir = tempdir().unwrap();
    let (hook, signature, root_cert) = signed_hook(dir.path());

    write(&hook, b"#!/bin/sh\necho tampered\n").unwrap();

    assert!(
        run(Args {
            hook,
            signature,
            root_cert
        })
        .is_err()
    );
}
