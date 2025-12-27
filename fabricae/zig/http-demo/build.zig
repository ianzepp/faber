const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Main executable
    const exe = b.addExecutable(.{
        .name = "http-demo",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Add httpz dependency
    const httpz = b.dependency("httpz", .{
        .target = target,
        .optimize = optimize,
    });
    exe.root_module.addImport("httpz", httpz.module("httpz"));

    b.installArtifact(exe);

    // Run step
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    const run_step = b.step("run", "Run the HTTP server");
    run_step.dependOn(&run_cmd.step);

    // Logic-only executable (for testing Faber output)
    const logic_exe = b.addExecutable(.{
        .name = "logic",
        .root_source_file = b.path("src/logic.zig"),
        .target = target,
        .optimize = optimize,
    });

    b.installArtifact(logic_exe);

    const logic_run = b.addRunArtifact(logic_exe);
    const logic_step = b.step("logic", "Run logic tests only");
    logic_step.dependOn(&logic_run.step);
}
