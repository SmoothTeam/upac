// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::{Context, Result};

use gettextrs::gettext;

use serde::Deserialize;

use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_PATH: &str = "/etc/upac/config.toml";

// ── Main config ─────────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub verbose: bool,
    pub paths: Paths,
    pub ostree: OstreeConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Paths {
    pub repo_path: CString,
    pub root_path: CString,
    pub backends_dir: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OstreeConfig {
    pub mode: CString,
    pub branch: CString,
    #[serde(default)]
    pub symlinks: Vec<CString>,
}

// ── Validation ─────────────────────────────────────────────────────────────────
impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let config_file_content =
            fs::read_to_string(path).with_context(|| format!("{}: {path:?}", gettext("err_read_config")))?;

        let config: Config = toml::from_str(&config_file_content)
            .map_err(|err| anyhow::anyhow!("{} {}: {err}", gettext("err_parse_config"), path.display()))?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.paths.repo_path.is_empty() {
            anyhow::bail!(gettext("err_config_repo_path_empty"));
        }
        if self.paths.root_path.is_empty() {
            anyhow::bail!(gettext("err_config_root_path_empty"));
        }
        if self.paths.backends_dir.as_os_str().is_empty() {
            anyhow::bail!(gettext("err_config_backends_dir_empty"));
        }
        if self.ostree.branch.is_empty() {
            anyhow::bail!(gettext("err_config_branch_empty"));
        }
        if self.ostree.mode.is_empty() {
            anyhow::bail!(gettext("err_config_mode_empty"));
        }
        for symlink in &self.ostree.symlinks {
            if symlink.is_empty() {
                anyhow::bail!(gettext("err_config_symlinks_empty_entry"));
            }
            if symlink.as_bytes().contains(&b'/') {
                anyhow::bail!("{}: {:?}", gettext("err_config_symlink_slash"), symlink);
            }
        }

        Ok(())
    }
}
