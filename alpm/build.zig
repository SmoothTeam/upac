// ── Imports ─────────────────────────────────────────────────────────────────────
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const strip = b.option(bool, "strip", "Strip debug symbols") orelse false;
    const stack_check = b.option(bool, "stack-check", "Check for stack overflows") orelse false;

    // ── Root ──────────────────────────────────────────────────────────────────
    const upac_backend_root = b.createModule(.{
        .root_source_file = b.path("src/backend.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_backend_root.strip = strip;
    upac_backend_root.stack_check = stack_check;

    // ── Shared library ────────────────────────────────────────────────────────
    const shared_lib = b.addLibrary(.{
        .name = "upac-alpm",
        .linkage = .dynamic,
        .root_module = upac_backend_root,
    });

    shared_lib.root_module.link_libc = true;
    shared_lib.root_module.linkSystemLibrary("archive", .{});

    shared_lib.root_module.strip = strip;
    shared_lib.root_module.stack_check = stack_check;
    shared_lib.bundle_compiler_rt = false;
    shared_lib.link_gc_sections = false;

    b.installArtifact(shared_lib);

    // ── Tests for shared library ────────────────────────────────────────────────────────
    const upac_test_root = b.createModule(.{
        .root_source_file = b.path("../tests/alpm/test.zig"),
        .target = target,
        .optimize = optimize,
    });

    upac_backend_root.strip = strip;
    upac_backend_root.stack_check = stack_check;

    const tests = b.addTest(.{
        .name = "alpm-test",
        .root_module = upac_test_root,
    });

    tests.root_module.addImport("upac-alpm", upac_backend_root);

    const run_tests = b.addRunArtifact(tests);

    const test_step = b.step("test", "Run ALPM tests");
    test_step.dependOn(&run_tests.step);
}
