// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use super::{Args, run};

fn generate_root(dir: &Path) -> (PathBuf, PathBuf) {
    let key_out = dir.join("root.key.pem");
    let cert_out = dir.join("root.cert.pem");

    crate::commands::generate_root::run(crate::commands::generate_root::Args {
        common_name: "upac test root".to_owned(),
        key_out: key_out.clone(),
        cert_out: cert_out.clone(),
    })
    .unwrap();

    (key_out, cert_out)
}

#[test]
fn generates_a_pem_encoded_signing_key_and_certificate() {
    let dir = tempdir().unwrap();
    let (root_key, root_cert) = generate_root(dir.path());
    let key_out = dir.path().join("signing.key.pem");
    let cert_out = dir.path().join("signing.cert.pem");

    run(Args {
        common_name: "upac test signing".to_owned(),
        root_key,
        root_cert,
        key_out: key_out.clone(),
        cert_out: cert_out.clone(),
    })
    .unwrap();

    assert!(read_to_string(&key_out).unwrap().starts_with("-----BEGIN"));
    assert!(read_to_string(&cert_out).unwrap().starts_with("-----BEGIN"));
}
