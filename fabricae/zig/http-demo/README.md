# Zig HTTP Demo

A REST API written entirely in Faber, compiled to Zig.

## Architecture

```
fons/main.fab      Full HTTP server in Faber
       |
       v (bun run compile -t zig)
src/main_fab.zig   Compiled Zig (server, routes, handlers)
```

## Key Faber Features for Zig

**Pointer semantics via prepositions:**

- `in T param` → `*T` (mutable pointer)
- `de T param` → `*const T` (read-only pointer)

**Typed lambdas for Zig:**

- `pro x, y -> vacuum { ... }` → anonymous struct pattern

**Example:**

```fab
// Mutable pointer - httpz passes *Request, *Response
functio handleIndex(in Request req, in Response res) {
    res.body = "Hello"
}

// Read-only pointer for validation
functio validateUser(de User user) -> bivalens {
    redde user.id > 0
}

// Typed lambda for route with captured state
server.get("/users", pro req, res -> vacuum { handleUsers(app, req, res) })
```

## Quick Start

```bash
# Compile Faber to Zig
bun run ../../../fons/cli.ts compile fons/main.fab -t zig -o src/main_fab.zig

# Build and run (requires Zig 0.13+)
zig build run
```

## API Endpoints

| Method | Path         | Description     |
| ------ | ------------ | --------------- |
| GET    | `/`          | Welcome message |
| GET    | `/health`    | Health check    |
| GET    | `/users`     | List all users  |
| GET    | `/users/:id` | Get user by ID  |
| POST   | `/users`     | Create user     |
| DELETE | `/users/:id` | Delete user     |

## What Faber Generates

From `fons/main.fab`:

```fab
functio handleIndex(in Request req, in Response res) {
    res.body = "Salve! Zig HTTP Demo"
}

functio handleUsers(in App app, in Request req, in Response res) {
    res.json({ message: "User list" })
}

server.get("/users", pro req, res -> vacuum { handleUsers(app, req, res) })
```

Compiles to `src/main_fab.zig`:

```zig
fn handleIndex(req: *Request, res: *Response) void {
    res.body = "Salve! Zig HTTP Demo";
}

fn handleUsers(app: *App, req: *Request, res: *Response) void {
    res.json(.{ .message = "User list" });
}

server.get("/users", struct { fn call(req: anytype, res: anytype) void {
    handleUsers(app, req, res);
} }.call);
```

## Preposition Semantics

| Faber    | Zig        | Use Case                          |
| -------- | ---------- | --------------------------------- |
| `in T x` | `*T`       | Mutable pointer (httpz handlers)  |
| `de T x` | `*const T` | Read-only pointer (validation)    |
| `T x`    | `T`        | Value (primitives, small structs) |

## Test with curl

```bash
# Health check
curl http://localhost:3000/health

# Create user
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"nomen": "Marcus", "email": "marcus@roma.it"}'

# List users
curl http://localhost:3000/users

# Get user
curl http://localhost:3000/users/1

# Delete user
curl -X DELETE http://localhost:3000/users/1
```

## Faber Features Demonstrated

- **Pointer params** via `in`/`de` prepositions
- **Typed lambdas** with `-> vacuum` for Zig closures
- **Struct definitions** (`genus`) with field defaults
- **Object literals** compile to Zig `.{ .field = value }`
- **String comparison** compiles to `std.mem.eql`
- **Control flow** (`si`/`aliter`, `non`)

## Known Limitations

- Server init doesn't match httpz's actual API (needs allocator)
- Error unions (`!void`) not yet expressible
- Lambda captures use `anytype` (works but less type-safe)

The generated code is structurally correct but may need manual tweaks for httpz specifics.
