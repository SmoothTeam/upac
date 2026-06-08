// ── Commit field indices ─────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CommitField {
    Checksum = 0,
    Subject = 1,
}

// ── Owned decoded types ──────────────────────────────────────────────────────
pub struct Commit {
    pub checksum: String,
    pub subject: String,
}
