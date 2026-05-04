// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::Result;

use indicatif::{ProgressBar, ProgressStyle};

use strum::{Display, EnumProperty, EnumString};

use std::fmt::Debug;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PackageField {
    Name = 0,
    Version = 1,
    Arch = 2,
    Author = 3,
    License = 5,
    Url = 6,
    Packager = 7,
    Size = 9,
}

impl PackageField {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Version => "version",
            Self::Arch => "architecture",
            Self::Author => "author",
            Self::License => "license",
            Self::Url => "url",
            Self::Packager => "packager",
            Self::Size => "size",
        }
    }
}

// ── Backend Definition ────────────────────────────────────────
// Represents the type of backend (ALPM, RPM, DEB) for a package
#[derive(Debug, Clone, Hash, Eq, PartialEq, Display, EnumString, EnumProperty)]
pub enum BackendKind {
    #[strum(serialize = "arch", to_string = "alpm", props(so = "libupac-alpm.so"))]
    Alpm,
    #[strum(serialize = "rpm", props(so = "libupac-rpm.so"))]
    Rpm,
    #[strum(serialize = "deb", props(so = "libupac-deb.so"))]
    Deb,
    #[strum(serialize = "xbps", props(so = "libupac-xbps.so"))]
    XBPS,
    #[strum(serialize = "upaclib", props(so = "libupac.so"))]
    UpacLib,
}

impl BackendKind {
    pub fn detect(file_path: &str) -> Option<Self> {
        let known_extensions = [
            (".pkg.tar.zst", Self::Alpm),
            (".pkg.tar.xz", Self::Alpm),
            (".pkg.tar.gz", Self::Alpm),
            (".rpm", Self::Rpm),
            (".deb", Self::Deb),
            (".xbps", Self::XBPS),
        ];

        known_extensions
            .iter()
            .find(|(extension, _)| file_path.ends_with(extension))
            .map(|(_, backend_kind)| backend_kind.clone())
    }

    pub fn from_flag(flag_string: &str) -> Result<Self> {
        flag_string.parse().map_err(|_| {
            anyhow::anyhow!("unknown backend: '{flag_string}'. Available: arch, rpm, deb, xbps")
        })
    }

    pub fn so_name(&self) -> &'static str {
        self.get_str("so").expect("so property not defined")
    }
}

pub fn spinner(pb: &ProgressBar, msg: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_owned());
    pb.enable_steady_tick(Duration::from_millis(80));
}
