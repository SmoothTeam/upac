// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter, Result as FmtResult};

use colored::Colorize;
use strum::AsRefStr;

use upac_abi::package::{CPackageMeta, CVersion};

macro_rules! str_field {
    ($field:expr) => {
        <&str>::try_from(&$field).unwrap_or_default()
    };
}

macro_rules! optional_str_field {
    ($field:expr) => {
        Option::<&str>::try_from(&$field).unwrap_or_default()
    };
}

macro_rules! required_str {
    ($field:expr) => {
        str_field!($field).to_owned()
    };
}

macro_rules! optional_str {
    ($field:expr) => {
        optional_str_field!($field).unwrap_or_default().to_owned()
    };
}

// ── Package field indices ────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, AsRefStr)]
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
        gettextrs::gettext(self.as_ref())
    }
}

// ── PackageFormatter ─────────────────────────────────────────────────────────
pub struct PackageFormatter<'a> {
    pub extra_fields: &'a [PackageField],
    pub metas: &'a [CPackageMeta],
}

impl<'a> PackageFormatter<'a> {
    pub fn print(&self) {
        if self.extra_fields.is_empty() {
            for meta in self.metas {
                println!("{}", required_str!(meta.name).bold());
            }
        } else {
            self.print_table();
        }
    }

    fn print_table(&self) {
        let all_fields: Vec<PackageField> = std::iter::once(PackageField::Name)
            .chain(self.extra_fields.iter().copied())
            .collect();

        let headers: Vec<String> = all_fields.iter().map(PackageField::display).collect();

        let rows: Vec<Vec<String>> = self
            .metas
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

    fn field_value(meta: &CPackageMeta, field: PackageField) -> String {
        match field {
            PackageField::Name => required_str!(meta.name),
            PackageField::Version => VersionDisplay(&meta.version).to_string(),
            PackageField::Architecture => {
                let arch = str_field!(meta.arch);
                match optional_str_field!(meta.arch_sub) {
                    Some(arch_sub) => format!("{arch}/{arch_sub}"),
                    None => arch.to_owned(),
                }
            }
            PackageField::Author | PackageField::Packager => required_str!(meta.maintainer),
            PackageField::License => optional_str!(meta.license),
            PackageField::Url => optional_str!(meta.url),
            PackageField::Description => required_str!(meta.description),
            PackageField::Checksum => hex::encode(meta.sha256),
            PackageField::Size => SizeDisplay(meta.installed_size).to_string(),
        }
    }
}

// ── Display wrappers ─────────────────────────────────────────────────────────
pub(crate) struct VersionDisplay<'a>(pub &'a CVersion);

impl Display for VersionDisplay<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let version = self.0;
        let parts = unsafe { version.parts.as_slice() };
        let version_str = parts.iter().map(|part| part.to_string()).collect::<Vec<_>>().join(".");

        if version.epoch > 0 {
            write!(formatter, "{}:{}", version.epoch, version_str)?;
        } else {
            write!(formatter, "{version_str}")?;
        }

        if version.release > 0 {
            write!(formatter, "-{}", version.release)?;
        }

        if let Some(pre) = optional_str_field!(version.pre) {
            write!(formatter, "~{pre}")?;
        }

        Ok(())
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
