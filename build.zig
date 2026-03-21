const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const mod = b.addModule("zig_ast", .{
        .root_source_file = b.path("zig_ast.zig"),
        .target = target,
        .optimize = optimize,
    });

    const lib = b.addLibrary(.{
        .name = "zig_ast",
        .root_module = mod,
    });

    b.installArtifact(lib);

    const gen = b.addExecutable(.{
        .name = "generate_rust",
        .root_module = b.createModule(.{
            .root_source_file = b.path("generate_rust.zig"),
            .target = b.graph.host,
            .imports = &.{
                .{ .name = "zig_ast", .module = mod },
            },
        }),
    });

    b.installArtifact(gen);

    const mod_tests = b.addTest(.{
        .root_module = mod,
    });

    const run_mod_tests = b.addRunArtifact(mod_tests);

    const test_step = b.step("test", "Run tests");
    test_step.dependOn(&run_mod_tests.step);
}
