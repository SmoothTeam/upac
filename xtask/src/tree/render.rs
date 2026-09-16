// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use super::walk::read_dir_filtered;

use crate::error::XtaskError;

pub struct TreeRenderer {
    output: String,
}

impl TreeRenderer {
    pub fn render(root: &Path, depth: usize) -> Result<String, XtaskError> {
        let root_name = root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "./".to_owned());

        let mut renderer = Self {
            output: format!("{root_name}/\n"),
        };
        renderer.render_dir(root, "", depth)?;

        Ok(renderer.output.trim_end().to_owned())
    }

    fn render_dir(&mut self, dir: &Path, prefix: &str, depth: usize) -> Result<(), XtaskError> {
        if depth == 0 {
            return Ok(());
        }

        let mut entries = read_dir_filtered(dir)?;
        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();

            b_is_dir.cmp(&a_is_dir).then_with(|| {
                a.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .cmp(&b.file_name().to_string_lossy().to_lowercase())
            })
        });

        let last_idx = entries.len().saturating_sub(1);
        for (idx, entry) in entries.iter().enumerate() {
            let is_last = idx == last_idx;
            let connector = if is_last { "└── " } else { "├── " };
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = entry.path().is_dir();

            if is_dir {
                self.output.push_str(&format!("{prefix}{connector}{name}/\n"));

                let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
                self.render_dir(&entry.path(), &child_prefix, depth - 1)?;
            } else {
                self.output.push_str(&format!("{prefix}{connector}{name}\n"));
            }
        }

        Ok(())
    }
}
