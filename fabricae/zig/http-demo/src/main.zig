// =============================================================================
// HTTP SERVER - Native Zig wrapper
// =============================================================================
//
// This is the HTTP layer written in native Zig. It imports business logic
// compiled from Faber (logic.zig) and exposes it via REST endpoints.
//
// WHY NATIVE ZIG:
//   - httpz requires specific patterns (allocators, error unions, handlers)
//   - These patterns don't map cleanly to Faber's abstractions yet
//   - Writing HTTP layer in Zig lets us use Faber for business logic
//
// BUILD:
//   zig build run
//
// =============================================================================

const std = @import("std");
const httpz = @import("httpz");

// Import Faber-generated logic
// WHY: This is compiled from fons/logic.fab
const logic = @import("logic.zig");

// Re-export types from logic for convenience
const User = logic.User;
const ApiResponse = logic.ApiResponse;

// =============================================================================
// APPLICATION STATE
// =============================================================================

const App = struct {
    allocator: std.mem.Allocator,
    users: std.ArrayList(User),
    next_id: i64,

    pub fn init(allocator: std.mem.Allocator) App {
        return .{
            .allocator = allocator,
            .users = std.ArrayList(User).init(allocator),
            .next_id = 1,
        };
    }

    pub fn deinit(self: *App) void {
        self.users.deinit();
    }

    pub fn generateId(self: *App) i64 {
        const id = self.next_id;
        self.next_id += 1;
        return id;
    }

    pub fn findUser(self: *App, id: i64) ?*User {
        for (self.users.items) |*user| {
            if (user.id == id) {
                return user;
            }
        }
        return null;
    }
};

// =============================================================================
// ROUTE HANDLERS
// =============================================================================

fn index(_: *httpz.Request, res: *httpz.Response) void {
    res.body = "Salve! Zig HTTP Demo";
}

fn health(_: *httpz.Request, res: *httpz.Response) void {
    res.json(.{ .status = "ok" }, .{}) catch {
        res.body = "error";
    };
}

fn listUsers(app: *App, _: *httpz.Request, res: *httpz.Response) void {
    // Return all users as JSON array
    res.json(app.users.items, .{}) catch {
        res.status = .internal_server_error;
        res.body = "Failed to serialize users";
    };
}

fn getUser(app: *App, req: *httpz.Request, res: *httpz.Response) void {
    const id_str = req.param("id") orelse {
        res.status = .bad_request;
        res.body = "Missing id parameter";
        return;
    };

    const id = std.fmt.parseInt(i64, id_str, 10) catch {
        res.status = .bad_request;
        res.body = "Invalid id format";
        return;
    };

    if (app.findUser(id)) |user| {
        // Use Faber-generated validation
        if (!logic.validateUser(user.*)) {
            res.status = .internal_server_error;
            res.body = "User data is invalid";
            return;
        }
        res.json(user.*, .{}) catch {
            res.status = .internal_server_error;
        };
    } else {
        res.status = .not_found;
        res.body = "User not found";
    }
}

fn createUser(app: *App, req: *httpz.Request, res: *httpz.Response) void {
    const body = req.json(struct {
        nomen: []const u8,
        email: []const u8,
    }) catch {
        res.status = .bad_request;
        res.body = "Invalid JSON body";
        return;
    } orelse {
        res.status = .bad_request;
        res.body = "Missing request body";
        return;
    };

    const user = User.init(.{
        .id = app.generateId(),
        .nomen = body.nomen,
        .email = body.email,
        .active = true,
    });

    // Use Faber-generated validation
    if (!logic.validateUser(user)) {
        res.status = .bad_request;
        res.body = "Invalid user data";
        return;
    }

    app.users.append(user) catch {
        res.status = .internal_server_error;
        res.body = "Failed to store user";
        return;
    };

    res.status = .created;
    res.json(user, .{}) catch {
        res.status = .internal_server_error;
    };
}

fn deleteUser(app: *App, req: *httpz.Request, res: *httpz.Response) void {
    const id_str = req.param("id") orelse {
        res.status = .bad_request;
        return;
    };

    const id = std.fmt.parseInt(i64, id_str, 10) catch {
        res.status = .bad_request;
        return;
    };

    // Find and remove user
    for (app.users.items, 0..) |user, i| {
        if (user.id == id) {
            _ = app.users.orderedRemove(i);
            res.status = .no_content;
            return;
        }
    }

    res.status = .not_found;
    res.body = "User not found";
}

// =============================================================================
// SERVER SETUP
// =============================================================================

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var app = App.init(allocator);
    defer app.deinit();

    // Initialize HTTP server with app state
    var server = try httpz.Server(*App).init(allocator, .{
        .port = 3000,
    }, &app);
    defer server.deinit();

    var router = server.router(.{});

    // Routes
    router.get("/", index);
    router.get("/health", health);
    router.get("/users", listUsers);
    router.get("/users/:id", getUser);
    router.post("/users", createUser);
    router.delete("/users/:id", deleteUser);

    std.debug.print("Server running on http://localhost:3000\n", .{});
    std.debug.print("Using Faber-generated logic from logic.zig\n", .{});

    // Run tests from Faber logic
    std.debug.print("\n--- Logic Module Tests ---\n", .{});
    std.debug.print("isSuccess(200) = {}\n", .{logic.isSuccess(200)});
    std.debug.print("isSuccess(404) = {}\n", .{logic.isSuccess(404)});
    std.debug.print("clamp(150, 0, 100) = {}\n", .{logic.clamp(150, 0, 100)});
    std.debug.print("--- End Tests ---\n\n", .{});

    try server.listen();
}
