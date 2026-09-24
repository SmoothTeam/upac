// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter, Result as FmtResult};

use clap::{Args as ClapArgs, ValueEnum};
use colored::Colorize;
use strum::AsRefStr;

use upac_types::package::PackageMeta;
use upac_types::package::Version;

use crate::locale::LOADER;

#[cfg(test)]
#[path = "../../tests/inline/display.rs"]
mod tests;

#[derive(Debug, Clone, Copy, AsRefStr, ValueEnum)]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum PackageField {
    Name = 0,
    Version = 1,
    Architecture = 2,
    Author = 3,
    Description = 4,
    License = 5,
    Url = 6,
    Packager = 7,
    Checksum = 8,
    Size = 9,
}

impl PackageField {
    pub fn display(&self) -> String {
        LOADER.get(self.as_ref())
    }
}

#[derive(ClapArgs, Default)]
pub struct DisplayPakcageMetaArgs {
    #[arg(long)]
    pub version: bool,
    #[arg(long)]
    pub arch: bool,
    #[arg(long)]
    pub author: bool,
    #[arg(long)]
    pub license: bool,
    #[arg(long)]
    pub url: bool,
    #[arg(long)]
    pub packager: bool,
    #[arg(long)]
    pub size: bool,
    #[arg(long)]
    pub description: bool,
    #[arg(long)]
    pub checksum: bool,
    #[arg(long, value_enum)]
    pub sort: Option<PackageField>,
}

impl DisplayPakcageMetaArgs {
    pub fn print(&self, metas: &[PackageMeta]) {
        PackageFormatter {
            extra_fields: &self.extra_fields(),
            metas,
            sort: self.sort,
        }
        .print();
    }

    fn extra_fields(&self) -> Vec<PackageField> {
        [
            (self.version, PackageField::Version),
            (self.arch, PackageField::Architecture),
            (self.author, PackageField::Author),
            (self.license, PackageField::License),
            (self.url, PackageField::Url),
            (self.packager, PackageField::Packager),
            (self.size, PackageField::Size),
            (self.description, PackageField::Description),
            (self.checksum, PackageField::Checksum),
        ]
        .into_iter()
        .filter_map(|(enabled, field)| enabled.then_some(field))
        .collect()
    }
}

pub struct PackageFormatter<'a> {
    pub extra_fields: &'a [PackageField],
    pub metas: &'a [PackageMeta],
    pub sort: Option<PackageField>,
}

impl<'a> PackageFormatter<'a> {
    pub fn print(&self) {
        let metas = self.ordered_metas();
        if self.extra_fields.is_empty() {
            for meta in &metas {
                println!("{}", meta.name.bold());
            }
        } else {
            self.print_table(&metas);
        }
    }

    fn ordered_metas(&self) -> Vec<&'a PackageMeta> {
        let mut metas: Vec<&PackageMeta> = self.metas.iter().collect();
        match self.sort {
            Some(PackageField::Version) => metas.sort_by(|a, b| a.version.cmp(&b.version)),
            Some(PackageField::Size) => metas.sort_by_key(|meta| meta.installed_size),
            Some(field) => metas.sort_by_key(|meta| Self::field_value(meta, field)),
            None => {}
        }
        metas
    }

    fn print_table(&self, metas: &[&PackageMeta]) {
        let all_fields: Vec<PackageField> = std::iter::once(PackageField::Name)
            .chain(self.extra_fields.iter().copied())
            .collect();

        let headers: Vec<String> = all_fields.iter().map(PackageField::display).collect();

        let rows: Vec<Vec<String>> = metas
            .iter()
            .map(|meta| all_fields.iter().map(|f| Self::field_value(meta, *f)).collect())
            .collect();

        let widths: Vec<usize> = (0..all_fields.len())
            .map(|col| {
                let header_w = headers[col].len();
                let data_w = rows.iter().map(|row| row[col].len()).max().unwrap_or(0);
                header_w.max(data_w)
            })
            .collect();

        let header_line = headers
            .iter()
            .zip(&widths)
            .map(|(h, &w)| format!("{:<w$}", h))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{}", header_line.bold());

        for row in &rows {
            let line = row
                .iter()
                .zip(&widths)
                .map(|(v, &w)| format!("{:<w$}", v))
                .collect::<Vec<_>>()
                .join("  ");
            println!("{}", line);
        }
    }

    fn field_value(meta: &PackageMeta, field: PackageField) -> String {
        match field {
            PackageField::Name => meta.name.clone(),
            PackageField::Version => VersionDisplay(&meta.version).to_string(),
            PackageField::Architecture => match meta.arch_sub.as_deref() {
                Some(arch_sub) => format!("{}/{arch_sub}", meta.arch),
                None => meta.arch.clone(),
            },
            PackageField::Author | PackageField::Packager => meta.maintainer.clone(),
            PackageField::License => meta.license.clone().unwrap_or_default(),
            PackageField::Url => meta.url.clone().unwrap_or_default(),
            PackageField::Description => meta.description.clone(),
            PackageField::Checksum => hex::encode(meta.sha256),
            PackageField::Size => SizeDisplay(meta.installed_size).to_string(),
        }
    }
}

pub(crate) struct VersionDisplay<'a>(pub &'a Version);

impl Display for VersionDisplay<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let version = self.0;

        if version.epoch > 0 {
            write!(formatter, "{}:{}", version.epoch, version.raw)
        } else {
            write!(formatter, "{}", version.raw)
        }
    }
}

struct SizeDisplay(u64);

impl Display for SizeDisplay {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self.0 {
            byte if byte < 1024 => write!(formatter, "{byte} B"),
            byte if byte < 1024 * 1024 => write!(formatter, "{} KB", byte / 1024),
            byte if byte < 1024 * 1024 * 1024 => write!(formatter, "{:.1} MB", byte as f64 / (1024.0 * 1024.0)),
            byte => write!(formatter, "{:.1} GB", byte as f64 / (1024.0 * 1024.0 * 1024.0)),
        }
    }
}
