// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi_symbols = @import("symbols/ffi.zig");
export const cancel = ffi_symbols.cancel;

export const deinit = ffi_symbols.deinit;

const installer_symbols = @import("symbols/installer.zig");
export const install = installer_symbols.install;

const uninstaller_symbols = @import("symbols/uninstaller.zig");
export const uninstall = uninstaller_symbols.uninstall;

const rollback_symbols = @import("symbols/rollback.zig");
export const rollback = rollback_symbols.rollback;

const diff_symbols = @import("symbols/diff.zig");
export const diff_packages = diff_symbols.diff_packages;
export const diff_packages_free = diff_symbols.diff_packages_free;

export const diff_files = diff_symbols.diff_files;
export const diff_files_free = diff_symbols.diff_files_free;

const list_symbols = @import("symbols/list.zig");
export const list_packages = list_symbols.list_packages;
export const get_packages_count = list_symbols.get_packages_count;
export const get_package_at = list_symbols.get_package_at;
export const get_package_slice_field = list_symbols.get_package_slice_field;
export const get_package_int_field = list_symbols.get_package_int_field;
export const packages_free = list_symbols.packages_free;

export const list_commits = list_symbols.list_commits;
export const get_commits_count = list_symbols.get_commits_count;
export const get_commit_at = list_symbols.get_commit_at;
export const get_commit_slice_field = list_symbols.get_commit_slice_field;
export const commits_free = list_symbols.commits_free;

const init_symbols = @import("symbols/init.zig");
export const init = init_symbols.init;
