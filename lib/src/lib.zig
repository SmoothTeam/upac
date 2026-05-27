// ── Imports ─────────────────────────────────────────────────────────────────────
const ffi_symbols = @import("symbols/ffi.zig");
export const get_abi_version = ffi_symbols.get_abi_version;
export const cancel = ffi_symbols.cancel;

const installer_symbols = @import("symbols/mutated/installer.zig");
export const install = installer_symbols.install;

const uninstaller_symbols = @import("symbols/mutated/uninstaller.zig");
export const uninstall = uninstaller_symbols.uninstall;

const rollback_symbols = @import("symbols/mutated/rollback.zig");
export const rollback = rollback_symbols.rollback;

const diff_symbols = @import("symbols/unmutated/diff.zig");
export const diff = diff_symbols.diff;
export const diff_free = diff_symbols.diff_free;

const list_packages_symbols = @import("symbols/unmutated/meta.zig");
export const list_metas = list_packages_symbols.list_metas;
export const metas_free = list_packages_symbols.metas_free;

const list_commits_symbols = @import("symbols/unmutated/commit.zig");
export const list_commits = list_commits_symbols.list_commits;
export const commits_free = list_commits_symbols.commits_free;

const init_symbols = @import("symbols/mutated/init.zig");
//export const init = init_symbols.init;
