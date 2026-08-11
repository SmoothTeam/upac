// SPDX-FileCopyrightText: 2026 JustPav
//
// SPDX-License-Identifier: GPL-3.0-only

//! `cargo xtask gen-tree [--check] [--depth N]`
//!
//! Walks the repository from its root and renders an ASCII directory tree
//! (dirs first, alphabetical, Unicode box-drawing characters — matching the
//! style already used in `doc/`). The tree is then spliced into every
//! markdown file that contains a `<!-- tree:start -->` / `<!-- tree:end -->`
//! marker pair, replacing everything between the markers (inclusive) with a
//! freshly generated fenced code block.
//!
//! `--check` doesn't write anything: it exits non-zero if any tracked file
//! would change, which is what CI should call.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const IGNORED_DIRS: &[&str] = &[".git", "target", "node_modules", ".zig-cache", "zig-out", "zig-pkg"];

const MARKER_START: &str = "<!-- tree:start -->";
const MARKER_END: &str = "<!-- tree:end -->";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();

    if cmd != "gen-tree" {
        eprintln!("usage: cargo xtask gen-tree [--check] [--depth N]");
        return ExitCode::FAILURE;
    }

    let mut check_only = false;
    let mut depth: usize = 2;

    let rest: Vec<String> = args.collect();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--check" => check_only = true,
            "--depth" => {
                i += 1;
                depth = rest.get(i).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("--depth needs an integer argument");
                    std::process::exit(2);
                });
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::FAILURE;
            }
        }
        i += 1;
    }

    let repo_root = repo_root();
    let root_name = repo_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "./".to_string());

    let mut tree_text = format!("{root_name}/\n");
    render_dir(&repo_root, "", depth, &mut tree_text);
    let tree_text = tree_text.trim_end().to_string();

    let targets = find_marked_files(&repo_root);
    if targets.is_empty() {
        eprintln!(
            "no file under {} contains {MARKER_START} / {MARKER_END} — nothing to do",
            repo_root.display()
        );
        return ExitCode::FAILURE;
    }

    let mut any_stale = false;
    for path in targets {
        let original = fs::read_to_string(&path).expect("read doc file");
        let updated = match splice(&original, &tree_text) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        };

        if original == updated {
            continue;
        }

        any_stale = true;
        let rel = path.strip_prefix(&repo_root).unwrap_or(&path);
        if check_only {
            println!("stale: {}", rel.display());
        } else {
            fs::write(&path, updated).expect("write doc file");
            println!("updated: {}", rel.display());
        }
    }

    if check_only {
        if any_stale {
            eprintln!("repo tree in docs is out of date — run `cargo xtask gen-tree`");
            return ExitCode::FAILURE;
        }
        println!("repo tree in docs is up to date");
    } else if !any_stale {
        println!("repo tree in docs was already up to date");
    }

    ExitCode::SUCCESS
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("xtask must live directly under the repo root")
        .to_path_buf()
}

fn render_dir(dir: &Path, prefix: &str, depth: usize, out: &mut String) {
    if depth == 0 {
        return;
    }

    let mut entries: Vec<_> = match fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    entries.retain(|e| {
        let name = e.file_name();
        let name = name.to_string_lossy();
        !IGNORED_DIRS.contains(&name.as_ref())
    });

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
            out.push_str(&format!("{prefix}{connector}{name}/\n"));
            let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
            render_dir(&entry.path(), &child_prefix, depth - 1, out);
        } else {
            out.push_str(&format!("{prefix}{connector}{name}\n"));
        }
    }
}

fn find_marked_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(root, &mut found);
    found
}

fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if IGNORED_DIRS.contains(&name.as_ref()) {
            continue;
        }
        if path.is_dir() {
            walk(&path, found);
            continue;
        }

        // Only markdown is a splice target — this also keeps xtask's own
        // source (which necessarily mentions the marker strings) out of it.
        let is_markdown = path
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !is_markdown {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&path) {
            if content.contains(MARKER_START) && content.contains(MARKER_END) {
                found.push(path);
            }
        }
    }
}

fn splice(original: &str, tree_text: &str) -> Result<String, String> {
    let start = original
        .find(MARKER_START)
        .ok_or_else(|| format!("missing {MARKER_START}"))?;
    let end = original
        .find(MARKER_END)
        .ok_or_else(|| format!("missing {MARKER_END}"))?;
    if end < start {
        return Err(format!("{MARKER_END} appears before {MARKER_START}"));
    }
    if original[start + MARKER_START.len()..].matches(MARKER_START).count() > 0 {
        return Err(format!("more than one {MARKER_START} in file"));
    }

    let end = end + MARKER_END.len();
    let block = format!("{MARKER_START}\n```text\n{tree_text}\n```\n{MARKER_END}");

    Ok(format!("{}{}{}", &original[..start], block, &original[end..]))
}
