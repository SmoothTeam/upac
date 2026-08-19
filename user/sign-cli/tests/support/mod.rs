// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("up-si-test-{}-{name}", std::process::id()));
    create_dir_all(&dir).unwrap();

    dir
}

pub fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_up-si"))
}

pub struct SigningChain {
    pub root_cert: PathBuf,
    pub signing_key: PathBuf,
    pub signing_cert: PathBuf,
}

pub fn generate_signing_chain(dir: &Path) -> SigningChain {
    let root_key = dir.join("root.key.pem");
    let root_cert = dir.join("root.cert.pem");

    let status = bin()
        .arg("generate-root")
        .arg("--common-name")
        .arg("test root")
        .arg("--key-out")
        .arg(&root_key)
        .arg("--cert-out")
        .arg(&root_cert)
        .status()
        .unwrap();
    assert!(status.success());

    let signing_key = dir.join("signing.key.pem");
    let signing_cert = dir.join("signing.cert.pem");

    let status = bin()
        .arg("generate-cert")
        .arg("--common-name")
        .arg("test signer")
        .arg("--root-key")
        .arg(&root_key)
        .arg("--root-cert")
        .arg(&root_cert)
        .arg("--key-out")
        .arg(&signing_key)
        .arg("--cert-out")
        .arg(&signing_cert)
        .status()
        .unwrap();
    assert!(status.success());

    SigningChain {
        root_cert,
        signing_key,
        signing_cert,
    }
}

pub fn sign_hook(chain: &SigningChain, hook_path: &Path, signature_path: &Path) -> bool {
    bin()
        .arg("sign-hook")
        .arg("--hook")
        .arg(hook_path)
        .arg("--key")
        .arg(&chain.signing_key)
        .arg("--cert")
        .arg(&chain.signing_cert)
        .arg("--signature")
        .arg(signature_path)
        .status()
        .unwrap()
        .success()
}

pub fn verify_hook(root_cert: &Path, hook_path: &Path, signature_path: &Path) -> bool {
    bin()
        .arg("verify-hook")
        .arg("--hook")
        .arg(hook_path)
        .arg("--signature")
        .arg(signature_path)
        .arg("--root-cert")
        .arg(root_cert)
        .status()
        .unwrap()
        .success()
}
