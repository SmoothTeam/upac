use strum::AsRefStr;

// ── Package field indices ────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum PackageField {
    Name = 0,
    Version = 1,
    Architecture = 2,
    Author = 3,
    License = 5,
    Url = 6,
    Packager = 7,
    Size = 9,
}

impl PackageField {
    pub fn display(&self) -> String {
        gettextrs::gettext(self.as_ref())
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
