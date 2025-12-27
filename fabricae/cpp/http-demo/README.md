# C++ HTTP Demo

A REST API written in Faber, compiled to C++23, using Crow framework.

## Architecture

```
fons/main.fab      Full HTTP server in Faber
       |
       v (bun run compile -t cpp)
src/main.cpp       Compiled C++23 (structs, handlers, routes)
       |
       v (cmake && make)
build/http-demo    Native executable
```

## Quick Start

```bash
# Compile Faber to C++
bun run ../../../fons/cli.ts compile fons/main.fab -t cpp -o src/main.cpp

# Build with CMake
mkdir -p build && cd build
cmake ..
make

# Run server
./http-demo
```

## API Endpoints

| Method | Path           | Description     |
| ------ | -------------- | --------------- |
| GET    | `/`            | Welcome message |
| GET    | `/health`      | Health check    |
| GET    | `/users`       | List all users  |
| GET    | `/users/<int>` | Get user by ID  |
| POST   | `/users`       | Create user     |
| DELETE | `/users/<int>` | Delete user     |

## Faber Features Demonstrated

**Reference semantics via prepositions:**

```fab
// de = const reference (read-only)
functio validateUser(de User user) -> bivalens

// in = mutable reference (will modify)
functio handleIndex(de request req, in response res)
```

Compiles to:

```cpp
bool validateUser(const User& user)
void handleIndex(const request& req, response& res)
```

**Struct definitions with C++23 features:**

```fab
genus User {
    numerus id: 0
    textus nomen: ""
}
```

Compiles to:

```cpp
struct User {
    int64_t id = 0;
    std::string nomen = std::string("");

    User() = default;

    template<typename Overrides>
        requires std::is_aggregate_v<Overrides>
    User(const Overrides& o) {
        if constexpr (requires { o.id; }) id = o.id;
        if constexpr (requires { o.nomen; }) nomen = o.nomen;
    }
};
```

**Lambda route handlers:**

```fab
server.route("/users").get(pro req, res { handleGetUsers(req, res) })
```

Compiles to:

```cpp
server.route("/users").get([&](auto req, auto res) {
    handleGetUsers(req, res);
});
```

## Preposition Semantics

| Faber    | C++                   | Use Case                                   |
| -------- | --------------------- | ------------------------------------------ |
| `de T x` | `const T& x`          | Read-only reference                        |
| `in T x` | `T& x`                | Mutable reference                          |
| `T x`    | `T x` or `const T& x` | Value (auto const-ref for strings/vectors) |

## Test with curl

```bash
# Health check
curl http://localhost:3000/health

# Create user
curl -X POST http://localhost:3000/users

# Get user
curl http://localhost:3000/users/1

# Delete user
curl -X DELETE http://localhost:3000/users/1
```

## Requirements

- C++23 compiler (GCC 13+, Clang 16+)
- CMake 3.20+
- Crow is fetched automatically via FetchContent
