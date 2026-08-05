// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fs;
use std::str::from_utf8;

use upac_pki::signature::{HookSignature, RootCertificate};

use crate::scripts::error::HookError;
use crate::scripts::file::HookFile;

pub fn load_hooks(
    hooks_dir: &str, root_cert_path: &str, hook_extension: &str, signature_extension: &str,
) -> Result<Vec<HookFile>, HookError> {
    let root_bytes = fs::read(root_cert_path)?;
    let root_certificate = RootCertificate::from_bytes(&root_bytes)?;

    let mut hooks = Vec::new();

    for entry in fs::read_dir(hooks_dir)? {
        let path = entry?.path();

        if path.extension().and_then(|extension| extension.to_str()) != Some(hook_extension) {
            continue;
        }

        let mut signature_path = path.clone().into_os_string();
        signature_path.push(".");
        signature_path.push(signature_extension);

        let hook_bytes = fs::read(&path)?;
        let signature_bytes = fs::read(&signature_path)?;

        let signature = HookSignature::from_bytes(&signature_bytes)?;
        signature.verify(&hook_bytes, &root_certificate)?;

        let hook_text = from_utf8(&hook_bytes)?;
        let hook_file = HookFile::parse(hook_text)?;

        hooks.push(hook_file);
    }

    Ok(hooks)
}
