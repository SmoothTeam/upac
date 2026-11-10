// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::collections::HashMap;

use upac_types::CancelToken;
use upac_types::diff::{FileDiffKind, PackageDiffKind};
use upac_types::error::ErrorKind;
use upac_types::package::{PackageMeta, Version};
use upac_types::response::entry::{DiffFileCommonEntry, DiffPackageEntry, DiffPrefixFileEntry, DiffUntrackedFileEntry};
use upac_types::response::unmutated::DiffResponse;

use upac_database::attribution::FileAttribute;

use upac_macro::stage;

use upac_orchestrator::context::Context;
use upac_orchestrator::stage::Stage;

use super::DiffSnapshot;

type PackageIdentity = (String, String, Option<String>);

pub struct ComparingStage;

#[stage]
impl Stage<ErrorKind> for ComparingStage {
    fn run(
        &self, context: &mut Context, _cancel: &CancelToken, _progress: &dyn Fn(Option<&str>, u64, u64),
    ) -> Result<(), ErrorKind> {
        let snapshot = context.take::<DiffSnapshot>()?;

        let mut packages = Self::diff_packages(snapshot.from_packages, snapshot.to_packages);
        let mut unattached_files = Vec::new();

        for (path, kind, source) in snapshot.changed_files {
            let database = match kind {
                FileDiffKind::Removed => &snapshot.from_database,
                FileDiffKind::Added | FileDiffKind::Modified => &snapshot.to_database,
            };

            let path = path.to_string_lossy().into_owned();
            match database.attribute_file(&path)? {
                Some(attribution) => {
                    let identity = Self::identity(&attribution.package_meta);

                    let entry = packages.entry(identity).or_insert_with(|| DiffPackageEntry {
                        name: attribution.package_meta.name.clone(),
                        kind: PackageDiffKind::FilesChanged,
                        version: attribution.package_meta.version.clone(),
                        files: Vec::new(),
                    });

                    entry.files.push(DiffPrefixFileEntry {
                        common: DiffFileCommonEntry { path, kind },
                        source,
                        package_name: attribution.package_meta.name,
                        is_user: attribution.file_entry.is_user,
                    });
                }
                None => unattached_files.push(DiffUntrackedFileEntry {
                    common: DiffFileCommonEntry { path, kind },
                    source,
                }),
            }
        }

        context.put(DiffResponse {
            diff_packages: packages.into_values().collect(),
            unattached_files,
        });

        Ok(())
    }
}

impl ComparingStage {
    fn identity(meta: &PackageMeta) -> PackageIdentity {
        (meta.name.clone(), meta.arch.clone(), meta.arch_sub.clone())
    }

    fn diff_packages(from: Vec<PackageMeta>, to: Vec<PackageMeta>) -> HashMap<PackageIdentity, DiffPackageEntry> {
        let from: HashMap<_, _> = from.into_iter().map(|meta| (Self::identity(&meta), meta)).collect();
        let mut to: HashMap<_, _> = to.into_iter().map(|meta| (Self::identity(&meta), meta)).collect();

        let mut packages = HashMap::new();

        for (identity, from_meta) in from {
            match to.remove(&identity) {
                Some(to_meta) if to_meta.sha256 != from_meta.sha256 => {
                    Self::insert(
                        &mut packages,
                        identity,
                        to_meta.name,
                        PackageDiffKind::Modified,
                        to_meta.version,
                    );
                }
                Some(_) => {}
                None => {
                    Self::insert(
                        &mut packages,
                        identity,
                        from_meta.name,
                        PackageDiffKind::Removed,
                        from_meta.version,
                    );
                }
            }
        }

        for (identity, to_meta) in to {
            Self::insert(
                &mut packages,
                identity,
                to_meta.name,
                PackageDiffKind::Added,
                to_meta.version,
            );
        }

        packages
    }

    fn insert(
        packages: &mut HashMap<PackageIdentity, DiffPackageEntry>, identity: PackageIdentity, name: String,
        kind: PackageDiffKind, version: Version,
    ) {
        packages.insert(
            identity,
            DiffPackageEntry {
                name,
                kind,
                version,
                files: Vec::new(),
            },
        );
    }
}
