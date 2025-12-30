const std = @import("std");

pub fn build(b: *std.Build) void {
    _ = b.addModule("faber", .{
        .root_source_file = b.path("mod.zig"),
    });
}
