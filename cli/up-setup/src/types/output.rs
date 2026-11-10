// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

pub struct Output {
    porcelain: bool,
}

impl Output {
    pub fn new(porcelain: bool) -> Self {
        Self { porcelain }
    }

    pub fn print(&self, values: &[(&str, &str)], human: impl FnOnce() -> String) {
        if !self.porcelain {
            println!("{}", human());
            return;
        }

        for (key, value) in values {
            println!("{key} {value}");
        }
    }
}
