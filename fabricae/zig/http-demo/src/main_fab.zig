const std = @import("std");

const _httpz = @import("httpz");
const Server = _httpz.Server;
const Request = _httpz.Request;
const Response = _httpz.Response;
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
const App = struct {
    nextId: i64 = 1,

    const Self = @This();

    pub fn init(overrides: anytype) Self {
        const self = Self{
            .nextId = if (@hasField(@TypeOf(overrides), "nextId")) overrides.nextId else 1,
        };
        return self;
    }
};
fn isValidId(id: i64) bool {
    return (id > 0);
}
fn validateUser(user: *const User) bool {
    if (!isValidId(user.id)) {
        return false;
    }
    if (std.mem.eql(u8, user.nomen, "")) {
        return false;
    }
    return true;
}
fn handleIndex(req: *Request, res: *Response) void {
    res.body = "Salve! Zig HTTP Demo";
}
fn handleHealth(req: *Request, res: *Response) void {
    res.json(.{ .status = "ok" });
}
fn handleUsers(app: *App, req: *Request, res: *Response) void {
    res.json(.{ .message = "User list" });
}
fn handleGetUser(app: *App, req: *Request, res: *Response) void {
    const id = req.param("id");
    if ((id == null)) {
        res.status = 400;
        res.body = "Missing id";
        return;
    }
    res.json(.{ .id = id });
}
fn handleCreateUser(app: *App, req: *Request, res: *Response) void {
    const id = app.nextId;
    app.nextId += 1;
    res.status = 201;
    res.json(.{ .id = id, .message = "Created" });
}
fn handleDeleteUser(app: *App, req: *Request, res: *Response) void {
    res.status = 204;
}

pub fn main() void {
    std.debug.print("{s}\n", .{ "Initializing HTTP server..." });
    var app: App = App.init(.{});
    const server = Server.init(.{ .port = 3000 });
    server.get("/", handleIndex);
    server.get("/health", handleHealth);
    server.get("/users", struct { fn call(req: anytype, res: anytype) void {
        handleUsers(app, req, res);
    } }.call);
    server.get("/users/:id", struct { fn call(req: anytype, res: anytype) void {
        handleGetUser(app, req, res);
    } }.call);
    server.post("/users", struct { fn call(req: anytype, res: anytype) void {
        handleCreateUser(app, req, res);
    } }.call);
    server.delete("/users/:id", struct { fn call(req: anytype, res: anytype) void {
        handleDeleteUser(app, req, res);
    } }.call);
    std.debug.print("{s}\n", .{ "Server running on http://localhost:3000" });
    server.listen();
}