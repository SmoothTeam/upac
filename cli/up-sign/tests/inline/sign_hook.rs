// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::{read, write};
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use super::{Args, run};

fn generate_signing_identity(dir: &Path) -> (PathBuf, PathBuf) {
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
        root_key,
        root_cert,
        key_out: signing_key.clone(),
        cert_out: signing_cert.clone(),
    })
    .unwrap();

    (signing_key, signing_cert)
}

#[test]
fn signs_a_hook_file_and_writes_a_signature() {
    let dir = tempdir().unwrap();
    let (key, cert) = generate_signing_identity(dir.path());

    let hook = dir.path().join("hook.sh");
    write(&hook, b"#!/bin/sh\necho hi\n").unwrap();
    let signature = dir.path().join("hook.sig");

    run(Args {
        hook,
        key,
        cert,
        signature: signature.clone(),
    })
    .unwrap();

    assert!(!read(&signature).unwrap().is_empty());
}
