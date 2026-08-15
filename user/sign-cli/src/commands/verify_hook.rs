// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::{read, read_to_string};
use std::path::PathBuf;

use anyhow::{Context, Result};

use upac_pki::signature::{HookSignature, RootCertificate};

use crate::errors::LocalizedPkiError;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub hook: PathBuf,
    #[arg(long)]
    pub signature: PathBuf,
    #[arg(long)]
    pub root_cert: PathBuf,
}

pub fn run(args: Args) -> Result<()> {
    let hook_bytes =
        read(&args.hook).with_context(|| format!("{}: {}", gettextrs::gettext("err_read"), args.hook.display()))?;
    let signature_bytes = read(&args.signature)
        .with_context(|| format!("{}: {}", gettextrs::gettext("err_read"), args.signature.display()))?;
    let root_cert_pem = read_to_string(&args.root_cert)
        .with_context(|| format!("{}: {}", gettextrs::gettext("err_read"), args.root_cert.display()))?;

    let signature = HookSignature::from_bytes(&signature_bytes).map_err(LocalizedPkiError)?;
    let root_certificate = RootCertificate::from_pem(&root_cert_pem).map_err(LocalizedPkiError)?;

    signature
        .verify(&hook_bytes, &root_certificate)
        .map_err(LocalizedPkiError)?;

    println!("{}", gettextrs::gettext("signature_valid"));

    Ok(())
}
