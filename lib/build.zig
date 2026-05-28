// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const strip = b.option(bool, "strip", "Strip debug symbols") orelse false;
    const stack_check = b.option(bool, "stack-check", "Check for stack overflows") orelse false;

    // ── C libs ─────────────────────────────────────────────────────────────────
    const translated_libs = b.addTranslateC(.{
        .root_source_file = b.path("src/imports.h"),
        .target = target,
        .optimize = optimize,
    });

    translated_libs.link_libc = true;

    translated_libs.linkSystemLibrary("ostree-1", .{});
    translated_libs.linkSystemLibrary("glib-2.0", .{});
    translated_libs.linkSystemLibrary("gio-2.0", .{});
    translated_libs.linkSystemLibrary("gobject-2.0", .{});

    const translated_libs_module = translated_libs.createModule();

    // ── LMDBX ──────────────────────────────────────────────────────────────
    const lmdbx = b.dependency("lmdbx", .{ .target = target, .optimize = optimize });

    // ── Serde ──────────────────────────────────────────────────────────────
    const serde = b.dependency("serde", .{ .target = target, .optimize = optimize });

    // ── Config ZON modules ─────────────────────────────────────────────────
    const paths_file = b.createModule(.{ .root_source_file = b.path("config/paths.zon") });
    const database_file = b.createModule(.{ .root_source_file = b.path("config/database.zon") });

    // ── Types ──────────────────────────────────────────────────────────────
    const upac_types = b.createModule(.{
        .root_source_file = b.path("src/types/types.zig"),
        .target = target,
        .optimize = optimize,
    });
    upac_types.addImport("paths.zon", paths_file);
    upac_types.addImport("database.zon", database_file);

    // ── FFI ─────────────────────────────────────────────────────────────────
    const upac_ffi = b.createModule(.{
        .root_source_file = b.path("src/ffi.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_ffi.addImport("c-libs", translated_libs_module);
    upac_ffi.addImport("upac-types", upac_types);

    // ── Database ──────────────────────────────────────────────────────────────
    const upac_database = b.createModule(.{
        .root_source_file = b.path("src/database/database.zig"),
        .target = target,
        .optimize = optimize,
    });
    upac_database.addImport("database.zon", database_file);
    upac_database.addImport("upac-types", upac_types);

    upac_database.addImport("lmdbx", lmdbx.module("lmdbx"));
    upac_database.addImport("serde", serde.module("serde"));

    // ── Installer ─────────────────────────────────────────────────────────────
    const upac_installer = b.createModule(.{
        .root_source_file = b.path("src/installer/installer.zig"),
        .target = target,
        .optimize = optimize,
    });
    upac_installer.addImport("c-libs", translated_libs_module);

    upac_installer.addImport("upac-types", upac_types);
    upac_installer.addImport("upac-ffi", upac_ffi);

    upac_installer.addImport("upac-database", upac_database);

    // ── Uninstaller ───────────────────────────────────────────────────────────
    const upac_uninstaller = b.createModule(.{
        .root_source_file = b.path("src/uninstaller/uninstaller.zig"),
        .target = target,
        .optimize = optimize,
    });
    upac_uninstaller.addImport("c-libs", translated_libs_module);

    upac_uninstaller.addImport("upac-types", upac_types);
    upac_uninstaller.addImport("upac-ffi", upac_ffi);

    upac_uninstaller.addImport("upac-database", upac_database);

    // ── Rollback ────────────────────────────────────────────────────────────────
    const upac_rollback = b.createModule(.{
        .root_source_file = b.path("src/rollback/rollback.zig"),
        .target = target,
        .optimize = optimize,
    });
    upac_rollback.addImport("c-libs", translated_libs_module);

    upac_rollback.addImport("upac-types", upac_types);
    upac_rollback.addImport("upac-ffi", upac_ffi);

    upac_rollback.addImport("upac-database", upac_database);

    // ── Files ────────────────────────────────────────────────────────────────
    const upac_files = b.createModule(.{
        .root_source_file = b.path("src/files/files.zig"),
        .target = target,
        .optimize = optimize,
    });
    upac_files.addImport("c-libs", translated_libs_module);

    upac_files.addImport("upac-types", upac_types);
    upac_files.addImport("upac-ffi", upac_ffi);

    upac_files.addImport("upac-database", upac_database);

    // ── Diff ────────────────────────────────────────────────────────────────
    const upac_diff = b.createModule(.{
        .root_source_file = b.path("src/diff/diff.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_diff.addImport("c-libs", translated_libs_module);

    upac_diff.addImport("upac-types", upac_types);
    upac_diff.addImport("upac-ffi", upac_ffi);

    upac_diff.addImport("upac-database", upac_database);

    // ── List ────────────────────────────────────────────────────────────────
    const upac_list_metas = b.createModule(.{
        .root_source_file = b.path("src/meta/meta.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_list_metas.addImport("c-libs", translated_libs_module);

    upac_list_metas.addImport("upac-types", upac_types);
    upac_list_metas.addImport("upac-ffi", upac_ffi);

    upac_list_metas.addImport("upac-database", upac_database);

    // ── Commits ────────────────────────────────────────────────────────────────
    const upac_list_commits = b.createModule(.{
        .root_source_file = b.path("src/commit/commit.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_list_commits.addImport("c-libs", translated_libs_module);

    upac_list_commits.addImport("upac-types", upac_types);
    upac_list_commits.addImport("upac-ffi", upac_ffi);

    // ── Init ──────────────────────────────────────────────────────────────────
    const upac_init = b.createModule(.{
        .root_source_file = b.path("src/init/init.zig"),
        .target = target,
        .optimize = optimize,
    });
    upac_init.addImport("c-libs", translated_libs_module);

    upac_init.addImport("upac-types", upac_types);
    upac_init.addImport("upac-ffi", upac_ffi);

    upac_init.addImport("upac-database", upac_database);

    // ── Root ──────────────────────────────────────────────────────────────────
    const upac_lib_root = b.createModule(.{
        .root_source_file = b.path("src/lib.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_lib_root.strip = strip;
    upac_lib_root.stack_check = stack_check;

    // ── Shared library ────────────────────────────────────────────────────────
    const shared_lib = b.addLibrary(.{
        .name = "upac",
        .linkage = .dynamic,
        .root_module = upac_lib_root,
    });

    shared_lib.root_module.link_libc = true;
    shared_lib.root_module.addImport("clibs", translated_libs_module);
    shared_lib.root_module.addImport("serde", serde.module("serde"));
    shared_lib.root_module.addImport("lmdbx", lmdbx.module("lmdbx"));

    shared_lib.root_module.addImport("upac-types", upac_types);
    shared_lib.root_module.addImport("upac-ffi", upac_ffi);

    shared_lib.root_module.addImport("upac-database", upac_database);

    shared_lib.root_module.addImport("upac-installer", upac_installer);
    shared_lib.root_module.addImport("upac-uninstaller", upac_uninstaller);
    shared_lib.root_module.addImport("upac-rollback", upac_rollback);
    shared_lib.root_module.addImport("upac-files", upac_files);
    shared_lib.root_module.addImport("upac-init", upac_init);

    shared_lib.root_module.addImport("upac-diff", upac_diff);
    shared_lib.root_module.addImport("upac-list-packages", upac_list_metas);
    shared_lib.root_module.addImport("upac-list-commits", upac_list_commits);

    b.installArtifact(shared_lib);

    // ── Tests for shared library ────────────────────────────────────────────────────────
    const upac_test_root = b.createModule(.{
        .root_source_file = b.path("../tests/lib/test.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_lib_root.strip = strip;
    upac_lib_root.stack_check = stack_check;

    const tests = b.addTest(.{
        .name = "lib-test",
        .root_module = upac_test_root,
    });

    tests.root_module.addImport("upac-lib", upac_lib_root);

    const run_tests = b.addRunArtifact(tests);

    const test_step = b.step("test", "Run Lib tests");
    test_step.dependOn(&run_tests.step);
}
