// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::read_to_string;

use tempfile::tempdir;

use super::{Args, run};

#[test]
fn generates_a_pem_encoded_key_and_certificate() {
    let dir = tempdir().unwrap();
    let key_out = dir.path().join("root.key.pem");
    let cert_out = dir.path().join("root.cert.pem");

    run(Args {
        common_name: "upac test root".to_owned(),
        key_out: key_out.clone(),
        cert_out: cert_out.clone(),
    })
    .unwrap();

    assert!(read_to_string(&key_out).unwrap().starts_with("-----BEGIN"));
    assert!(read_to_string(&cert_out).unwrap().starts_with("-----BEGIN"));
}
