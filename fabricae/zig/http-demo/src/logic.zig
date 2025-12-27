const std = @import("std");

const User = struct {
    id: i64 = 0,
    nomen: []const u8 = "",
    email: []const u8 = "",
    active: bool = true,

    const Self = @This();

    pub fn init(overrides: anytype) Self {
        const self = Self{
            .id = if (@hasField(@TypeOf(overrides), "id")) overrides.id else 0,
            .nomen = if (@hasField(@TypeOf(overrides), "nomen")) overrides.nomen else "",
            .email = if (@hasField(@TypeOf(overrides), "email")) overrides.email else "",
            .active = if (@hasField(@TypeOf(overrides), "active")) overrides.active else true,
        };
        return self;
    }
};
const ApiResponse = struct {
    success: bool = true,
    status: i64 = 200,

    const Self = @This();

    pub fn init(overrides: anytype) Self {
        const self = Self{
            .success = if (@hasField(@TypeOf(overrides), "success")) overrides.success else true,
            .status = if (@hasField(@TypeOf(overrides), "status")) overrides.status else 200,
        };
        return self;
    }
};
fn isValidId(id: i64) bool {
    return (id > 0);
}
fn isValidEmail(email: []const u8) bool {
    return true;
}
fn validateUser(user: User) bool {
    if (!isValidId(user.id)) {
        return false;
    }
    if (std.mem.eql(u8, user.nomen, "")) {
        return false;
    }
    return true;
}
fn clamp(value: i64, min: i64, max: i64) i64 {
    if ((value < min)) {
        return min;
    }
    if ((value > max)) {
        return max;
    }
    return value;
}
fn isSuccess(status: i64) bool {
    return ((status >= 200) and (status < 300));
}
fn isClientError(status: i64) bool {
    return ((status >= 400) and (status < 500));
}
fn isServerError(status: i64) bool {
    return ((status >= 500) and (status < 600));
}

pub fn main() void {
    std.debug.print("{s}\n", .{ "Logic module loaded" });
    const testUser = User.init(.{ .id = 1, .nomen = "Marcus", .email = "marcus@roma.it" });
    if (validateUser(testUser)) {
        std.debug.print("{s}\n", .{ "User is valid" });
    } else {
        std.debug.print("{s}\n", .{ "User is invalid" });
    }
    std.debug.print("{s} {}\n", .{ "Status 200 is success:", isSuccess(200) });
    std.debug.print("{s} {d}\n", .{ "Clamped:", clamp(150, 0, 100) });
}