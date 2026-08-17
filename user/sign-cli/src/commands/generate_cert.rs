// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fs::{read_to_string, write};
use std::path::PathBuf;

use anyhow::{Context, Result};

use clap::Args as ClapArgs;

use upac_pki::generate::{Identity, PemIdentity, RootIdentity, generate_signing_cert};

use crate::errors::LocalizedPkiError;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    pub common_name: String,
    #[arg(long)]
    pub root_key: PathBuf,
    #[arg(long)]
    pub root_cert: PathBuf,
    #[arg(long)]
    pub key_out: PathBuf,
    #[arg(long)]
    pub cert_out: PathBuf,
}

pub fn run(args: Args) -> Result<()> {
    let root_pem = PemIdentity {
        key_pem: read_to_string(&args.root_key)
            .with_context(|| format!("{}: {}", gettextrs::gettext("err_read"), args.root_key.display()))?,
        certificate_pem: read_to_string(&args.root_cert)
            .with_context(|| format!("{}: {}", gettextrs::gettext("err_read"), args.root_cert.display()))?,
    };
    let root = RootIdentity::from_pem(&root_pem).map_err(LocalizedPkiError)?;

    let signing = generate_signing_cert(&args.common_name, &root).map_err(LocalizedPkiError)?;
    let pem = signing.to_pem().map_err(LocalizedPkiError)?;

    write(&args.key_out, &pem.key_pem)
        .with_context(|| format!("{}: {}", gettextrs::gettext("err_write"), args.key_out.display()))?;
    write(&args.cert_out, &pem.certificate_pem)
        .with_context(|| format!("{}: {}", gettextrs::gettext("err_write"), args.cert_out.display()))?;

    Ok(())
}
