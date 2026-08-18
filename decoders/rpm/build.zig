// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const strip = b.option(bool, "strip", "Strip debug symbols") orelse false;
    const stack_check = b.option(bool, "stack-check", "Check for stack overflows") orelse false;

    // ── C libs ────────────────────────────────────────────────────────────────
    const translated_libs = b.addTranslateC(.{
        .root_source_file = b.path("src/imports.h"),
        .target = target,
        .optimize = optimize,
    });
    translated_libs.link_libc = true;
    translated_libs.linkSystemLibrary("archive", .{});

    const c_libs_module = translated_libs.createModule();

    // ── Types ─────────────────────────────────────────────────────────────────
    const upac_backend_types = b.createModule(.{
        .root_source_file = b.path("src/types/types.zig"),
        .target = target,
        .optimize = optimize,
    });

    // ── FFI ─────────────────────────────────────────────────────────────────
    const upac_backend_ffi = b.createModule(.{
        .root_source_file = b.path("src/ffi.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_backend_ffi.addImport("upac-backend-types", upac_backend_types);

    // ── Root ──────────────────────────────────────────────────────────────────
    const upac_backend_root = b.createModule(.{
        .root_source_file = b.path("src/symbols.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_backend_root.strip = strip;
    upac_backend_root.stack_check = stack_check;

    // ── Shared library ────────────────────────────────────────────────────────
    const shared_lib = b.addLibrary(.{
        .name = "upac-rpm",
        .linkage = .dynamic,
        .root_module = upac_backend_root,
    });

    shared_lib.root_module.addImport("c-libs", c_libs_module);
    shared_lib.root_module.addImport("upac-backend-types", upac_backend_types);
    shared_lib.root_module.addImport("upac-backend-ffi", upac_backend_ffi);

    shared_lib.root_module.strip = strip;
    shared_lib.root_module.stack_check = stack_check;
    shared_lib.bundle_compiler_rt = false;
    shared_lib.link_gc_sections = false;

    b.installArtifact(shared_lib);
}
