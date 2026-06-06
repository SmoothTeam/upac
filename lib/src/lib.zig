// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi_symbols = @import("symbols/ffi.zig");
export const version_abi = ffi_symbols.version_abi;
export const cancel = ffi_symbols.cancel;
export const free_response = ffi_symbols.free_response;

const installer_symbols = @import("symbols/mutated/installer.zig");
export const install = installer_symbols.install;

const update_symbols = @import("symbols/mutated/update.zig");
export const update = update_symbols.update;

const uninstaller_symbols = @import("symbols/mutated/uninstaller.zig");
export const uninstall = uninstaller_symbols.uninstall;

const rollback_symbols = @import("symbols/mutated/rollback.zig");
export const rollback = rollback_symbols.rollback;

const file_symbols = @import("symbols/mutated/file.zig");
export const files = file_symbols.files;

const init_symbols = @import("symbols/mutated/init.zig");
export const init = init_symbols.init;

const commit_symbols = @import("symbols/mutated/commit.zig");
export const commit = commit_symbols.commit;

const diff_packages_symbols = @import("symbols/unmutated/diff/packages.zig");
export const diff_packages = diff_packages_symbols.diff_packages;

const diff_files_symbols = @import("symbols/unmutated/diff/files.zig");
export const diff_files = diff_files_symbols.diff_files;

const list_packages_symbols = @import("symbols/unmutated/list/meta.zig");
export const list_metas = list_packages_symbols.list_metas;

const list_commits_symbols = @import("symbols/unmutated/list/commit.zig");
export const list_commits = list_commits_symbols.list_commits;

const search_symbols = @import("symbols/unmutated/search/meta.zig");
export const search_meta = search_symbols.search_meta;

const search_files_symbols = @import("symbols/unmutated/search/files.zig");
export const search_files = search_files_symbols.search_files;
