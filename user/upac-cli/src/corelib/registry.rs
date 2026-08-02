// ── Imports ─────────────────────────────────────────────────────────────────
use anyhow::{Context, Result};

use std::fs;
use std::path::Path;

use crate::corelib::backend::Backend;
use crate::types::backend::BackendConfig;

// ── BackendRegistry ───────────────────────────────────────────────────────────
pub struct BackendRegistry {
    backends: Vec<BackendConfig>,
}

impl BackendRegistry {
    pub fn scan(backends_dir: &Path) -> Result<Self> {
        let mut backends = Vec::new();

        if !backends_dir.exists() {
            return Ok(Self { backends });
        }

        for entry in fs::read_dir(backends_dir)
            .with_context(|| format!("{}: {}", gettextrs::gettext("err_read"), backends_dir.display()))?
        {
            let path = entry
                .with_context(|| format!("{}: {}", gettextrs::gettext("err_read"), backends_dir.display()))?
                .path();
            if path.extension().is_some_and(|extension| extension == "toml") {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("{}: {}", gettextrs::gettext("err_read"), path.display()))?;

                let backend_config: BackendConfig = toml::from_str(&content)
                    .with_context(|| format!("{}: {}", gettextrs::gettext("err_parse"), path.display()))?;
                backends.push(backend_config);
            }
        }

        Ok(Self { backends })
    }

    pub fn by_extension(&self, file_path: &str) -> Option<&BackendConfig> {
        self.backends.iter().find(|backend_config| {
            backend_config
                .extensions
                .iter()
                .any(|ext| file_path.ends_with(ext.as_str()))
        })
    }

    pub fn by_flag(&self, flag: &str) -> Option<&BackendConfig> {
        self.backends
            .iter()
            .find(|backend_config| backend_config.flags.iter().any(|backend_flag| backend_flag == flag))
    }

    pub fn load(&self, backend_config: &BackendConfig) -> Result<Backend> {
        Backend::load(&backend_config.so)
    }
}
