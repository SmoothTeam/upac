// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

use colored::Colorize;
use strum::AsRefStr;

use crate::ffi::packages::CPackageMeta;

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
                println!("{}", unsafe { meta.name.as_str() }.bold());
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
            .map(|meta| all_fields.iter().map(|f| unsafe { field_value(meta, *f) }).collect())
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
}

// ── Owned package types ──────────────────────────────────────────────────────
pub struct PackageMeta {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub author: String,
    pub license: String,
    pub url: String,
    pub packager: String,
    pub size: u64,
}

// ── Helpers ───────────────────────────────────────────────────────────────────
unsafe fn field_value(meta: &CPackageMeta, field: PackageField) -> String {
    unsafe {
        match field {
            PackageField::Name => meta.name.as_str().to_owned(),
            PackageField::Version => meta.version.display(),
            PackageField::Architecture => {
                let arch = meta.arch.as_str();
                if !meta.arch_sub.ptr.is_null() && meta.arch_sub.len > 0 {
                    format!("{}/{}", arch, meta.arch_sub.as_str())
                } else {
                    arch.to_owned()
                }
            }
            PackageField::Author | PackageField::Packager => meta.maintainer.as_str().to_owned(),
            PackageField::License => meta.license.as_str().to_owned(),
            PackageField::Url => meta.url.as_str().to_owned(),
            PackageField::Description => meta.description.as_str().to_owned(),
            PackageField::Checksum => hex::encode(meta.sha256),
            PackageField::Size => format_size(meta.installed_size),
        }
    }
}

fn format_size(bytes: u64) -> String {
    match bytes {
        byte if byte < 1024 => format!("{byte} B"),
        byte if byte < 1024 * 1024 => format!("{} KB", byte / 1024),
        byte if byte < 1024 * 1024 * 1024 => format!("{:.1} MB", byte as f64 / (1024.0 * 1024.0)),
        byte => format!("{:.1} GB", byte as f64 / (1024.0 * 1024.0 * 1024.0)),
    }
}
