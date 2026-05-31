// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi_symbols = @import("symbols/ffi.zig");
export const get_abi_version = ffi_symbols.get_abi_version;
export const cancel = ffi_symbols.cancel;
export const response_free = ffi_symbols.response_free;

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

const diff_symbols = @import("symbols/unmutated/diff/files.zig");
export const diff_files = diff_symbols.diff_files;

const list_packages_symbols = @import("symbols/unmutated/list/meta.zig");
export const list_metas = list_packages_symbols.list_metas;

const list_commits_symbols = @import("symbols/unmutated/list/commit.zig");
export const list_commits = list_commits_symbols.list_commits;

const search_symbols = @import("symbols/unmutated/search/meta.zig");
export const search_meta = search_symbols.search_meta;

const search_files_symbols = @import("symbols/unmutated/search/files.zig");
export const search_files = search_files_symbols.search_files;
