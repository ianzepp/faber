---
status: partial
targets: [cpp]
note: Core language and collection stdlib working; `de`/`in` prepositions and async pending
updated: 2024-12
---

# C++23 Target Notes

C++23 is a natural systems target for Faber, bridging the gap between the high-level TypeScript/Python targets and the low-level Zig/Rust targets. Modern C++ (C++23) has converged toward Rust-like patterns (`std::expected`, concepts, ranges) making it well-aligned with Faber's existing systems target designs.

C++23 shares ownership concerns with Zig and Rust. Faber uses a **unified approach** for all systems targets: Latin prepositions (`de`, `in`) for borrowing semantics. See `praepositiones.md` for the unified preposition system. Unlike Zig/Rust, C++23 has exceptions and RAII, giving more flexibility in error handling and memory management.

## Why C++23

1. **Convergence with Rust** - `std::expected<T,E>` mirrors `Result<T,E>`, same mental model
2. **Mature ecosystem** - Vast library ecosystem, tooling, IDE support
3. **Exceptions work** - Unlike Zig/Rust, `tempta`/`cape`/`demum` maps directly to try/catch/finally semantics via RAII
4. **RAII simplifies memory** - Arena allocation is optional, not required
5. **Modern features** - Concepts, ranges, coroutines, `std::print`

## What C++23 Adds Over C++20

| Feature              | Benefit for Faber                    |
| -------------------- | ------------------------------------ |
| `std::expected<T,E>` | Rust-like error handling for `iace`  |
| `std::print`         | Clean output, no iostream complexity |
| `std::views::zip`    | Iteration patterns                   |
| `std::ranges::to<>`  | Collection transformations           |
| `std::unreachable()` | Optimization for impossible branches |

## Direct Mappings

| Faber                   | C++23                                      | Notes                                           |
| ----------------------- | ------------------------------------------ | ----------------------------------------------- |
| `varia`                 | `auto` (mutable)                           | Or explicit type                                |
| `fixum`                 | `const auto`                               | Immutable binding                               |
| `functio`               | `auto fn() ->`                             | Trailing return type                            |
| `futura`                | coroutine                                  | Returns `std::future` or custom awaitable       |
| `cede`                  | `co_await`/`co_yield`                      | Context-dependent                               |
| `genus`                 | `struct`/`class`                           | With auto-merge constructor                     |
| `pactum`                | `concept`                                  | Compile-time interface                          |
| `implet`                | `requires`                                 | Concept satisfaction                            |
| `ego`                   | `this->` or `self`                         | Via reference capture                           |
| `novum`                 | direct construction                        | No `new` for stack, `std::make_unique` for heap |
| `novum X {}`            | Designated initializers                    | C++20 designated init                           |
| `si`/`secus`           | `if`/`else`                                | Direct                                          |
| `dum`                   | `while`                                    | Direct                                          |
| `ex...pro`              | range-for                                  | `for (auto x : items)`                          |
| `elige`                 | `switch` or if-chain                       | Limited pattern matching                        |
| `tempta`/`cape`/`demum` | try/catch + RAII                           | `demum` via destructor                          |
| `iace`                  | `return std::unexpected`                   | Or `throw`                                      |
| `mori`                  | `std::unreachable()` or `std::terminate()` | Fatal                                           |
| `adfirma`               | `assert()`                                 | Or `static_assert`                              |
| `redde`                 | `return`                                   | Direct                                          |
| `scribe`                | `std::print`                               | C++23                                           |

## Type Mappings

| Faber          | C++23                     | Notes                                   |
| -------------- | ------------------------- | --------------------------------------- |
| `textus`       | `std::string`             | Owned string                            |
| `de textus`    | `std::string_view`        | Borrowed, read-only                     |
| `in textus`    | `std::string&`            | Mutable reference                       |
| `numerus`      | `int64_t`                 | 64-bit integer                          |
| `fractus`      | `double`                  | 64-bit float                            |
| `decimus`      | —                         | Requires library (Boost.Multiprecision) |
| `bivalens`     | `bool`                    | Boolean                                 |
| `nihil`        | `std::nullopt`            | In optional context                     |
| `vacuum`       | `void`                    | Void return                             |
| `T[]`          | `std::vector<T>`          | Dynamic array                           |
| `de T[]`       | `std::span<const T>`      | Borrowed view                           |
| `in T[]`       | `std::vector<T>&`         | Mutable reference                       |
| `tabula<K,V>`  | `std::unordered_map<K,V>` | Hash map                                |
| `copia<T>`     | `std::unordered_set<T>`   | Hash set                                |
| `T?`           | `std::optional<T>`        | Nullable                                |
| `T \| U`       | `std::variant<T,U>`       | Union type                              |
| `promissum<T>` | `std::future<T>`          | Async result                            |
| `erratum`      | `std::exception`          | Error base                              |
| `cursor<T>`    | Generator coroutine       | `std::generator<T>` (C++23)             |

## Ownership Design: Latin Prepositions

> **Note:** The `de` and `in` keywords are **systems-target-specific**. See rust.md and zig.md for the full rationale. This design is shared across all systems targets.

| Preposition | Meaning             | C++23 Output                     |
| ----------- | ------------------- | -------------------------------- |
| (none)      | Owned, may move     | `T` (by value, moved)            |
| `de`        | Borrowed, read-only | `const T&` or `std::string_view` |
| `in`        | Mutable borrow      | `T&`                             |

### Examples

```
// Faber
functio greet(de textus name) -> textus {
    redde "Hello, " + name + "!"
}

functio append(in textus[] items, textus value) {
    items.adde(value)
}

functio consume(textus data) -> Result {
    // data is moved in
}
```

**C++23 output:**

```cpp
auto greet(std::string_view name) -> std::string {
    return std::format("Hello, {}!", name);
}

auto append(std::vector<std::string>& items, std::string value) -> void {
    items.push_back(std::move(value));
}

auto consume(std::string data) -> Result {
    // data is owned, moved in
}
```

### `de` on Return Type

When returning a borrowed value, use `de` on return type to indicate the reference is tied to input lifetime:

```
// Faber
functio first(de textus[] items) -> de textus {
    redde items.primus()
}
```

**C++23 output:**

```cpp
auto first(std::span<const std::string> items) -> std::string_view {
    return items.front();
}
```

Lifetime is implicit - the returned view is valid as long as the input span is valid.

## Error Handling Design

C++23 supports two error handling models. Faber can target either.

### Model A: `std::expected` (Rust-like)

Preferred for consistency with Zig/Rust targets.

```
// Faber
functio fetch(textus url) -> textus {
    si timeout { iace "timeout" }
    redde data
}

fac {
    fixum result = fetch(url)
} cape err {
    handleError(err)
}
```

**C++23 output:**

```cpp
#include <expected>
#include <string>

using FaberError = std::string;

template<typename T>
using Result = std::expected<T, FaberError>;

auto fetch(std::string url) -> Result<std::string> {
    if (timeout) {
        return std::unexpected("timeout");  // iace
    }
    return data;  // implicit success
}

// fac/cape
auto result = fetch(url);
if (!result) {
    handleError(result.error());
} else {
    auto value = *result;
}

// Or with monadic operations
fetch(url)
    .transform([](auto data) { /* success */ })
    .or_else([](auto err) { handleError(err); return Result<std::string>{""}; });
```

### Model B: Exceptions (TypeScript/Python-like)

For code that prefers traditional exception handling:

```cpp
auto fetch(std::string url) -> std::string {
    if (timeout) {
        throw std::runtime_error("timeout");  // iace
    }
    return data;
}

// tempta/cape/demum maps directly
try {
    auto result = fetch(url);
} catch (const std::exception& err) {
    handleError(err);
}
// demum: RAII destructors handle cleanup
```

### `mori` - Fatal Errors

`mori` indicates unrecoverable errors:

```cpp
if (impossible_condition) {
    std::unreachable();  // C++23, optimizer hint
    // or
    std::terminate();    // Actually halt
}
```

### Recommendation

Use `std::expected` (Model A) as the default for consistency with Zig/Rust. The codegen could offer a flag to switch to exceptions for projects that prefer them.

## `demum` via RAII

Unlike Zig/Rust where `demum` (finally) has no direct equivalent, C++23 supports it via RAII scope guards:

```cpp
#include <scope>  // C++23 <experimental/scope> or custom

auto process() -> void {
    auto cleanup = std::scope_exit([]{
        // demum code runs on scope exit
        closeConnection();
    });

    riskyOperation();  // tempta block
}
```

Or use a simple scope guard:

```cpp
template<typename F>
struct ScopeExit {
    F fn;
    ~ScopeExit() { fn(); }
};

#define DEMUM auto _demum = ScopeExit{[&]{
#define DEMUM_END }};
```

## Lambda Syntax

Faber's `pro` lambdas map to C++ lambdas:

### Expression Lambda

```
// Faber
pro x redde x * 2

// C++23
[](auto x) { return x * 2; }
```

### Block Lambda

```
// Faber
pro user {
    si user.aetas < 18 { redde falsum }
    redde user.activus
}

// C++23
[](auto user) {
    if (user.aetas < 18) { return false; }
    return user.activus;
}
```

### Capturing Lambda

```
// Faber
fixum multiplier = 2
pro x redde x * multiplier

// C++23 - captures by value by default
[multiplier](auto x) { return x * multiplier; }
// or capture by reference
[&multiplier](auto x) { return x * multiplier; }
```

C++ lambdas are more flexible than Zig's (which require explicit context structs) and similar to Rust's closures.

## Auto-Merge Constructor

Faber's `novum X { overrides }` uses C++20 designated initializers combined with a merge pattern:

```cpp
struct Persona {
    std::string nomen = "anonymous";
    int aetas = 0;

    // Default constructor
    Persona() = default;

    // Auto-merge constructor using C++20 concepts
    template<typename Overrides>
        requires std::is_aggregate_v<Overrides>
    Persona(const Overrides& o) {
        if constexpr (requires { o.nomen; }) nomen = o.nomen;
        if constexpr (requires { o.aetas; }) aetas = o.aetas;
        _creo();
    }

    void _creo() { /* user initialization */ }
};

// Usage: novum Persona { nomen: "Marcus" }
auto p = Persona{{.nomen = "Marcus"}};
```

The `if constexpr (requires { ... })` pattern gives the same compile-time field checking as Zig's `@hasField`.

## Pactum as Concepts

Faber's `pactum` (interface) maps to C++20 concepts:

```
// Faber
pactum Nominatus {
    functio nomen() -> textus
}

genus Persona implet Nominatus {
    functio nomen() -> textus { redde ego.nomen }
}
```

**C++23 output:**

```cpp
template<typename T>
concept Nominatus = requires(T t) {
    { t.nomen() } -> std::convertible_to<std::string>;
};

struct Persona {
    std::string _nomen;

    auto nomen() -> std::string { return _nomen; }
};

static_assert(Nominatus<Persona>);  // Compile-time verification
```

Unlike runtime interfaces (virtual functions), concepts are compile-time and zero-cost.

## Lista Methods via Ranges

C++23 ranges provide functional-style list operations:

| Latin       | C++23                     | Example                              |
| ----------- | ------------------------- | ------------------------------------ |
| `mappata`   | `std::views::transform`   | `items \| std::views::transform(fn)` |
| `filtrata`  | `std::views::filter`      | `items \| std::views::filter(pred)`  |
| `primus`    | `.front()`                | `items.front()`                      |
| `ultimus`   | `.back()`                 | `items.back()`                       |
| `longitudo` | `.size()`                 | `items.size()`                       |
| `adde`      | `.push_back()`            | `items.push_back(x)`                 |
| `inversa`   | `std::views::reverse`     | `items \| std::views::reverse`       |
| `prima`     | `std::views::take`        | `items \| std::views::take(n)`       |
| `omitte`    | `std::views::drop`        | `items \| std::views::drop(n)`       |
| `coniunge`  | —                         | Custom or `fmt::join`                |
| `reducta`   | `std::accumulate` or fold | `std::ranges::fold_left`             |
| `omnes`     | `std::ranges::all_of`     | `std::ranges::all_of(items, pred)`   |
| `aliquis`   | `std::ranges::any_of`     | `std::ranges::any_of(items, pred)`   |

### Ranges Pipeline Example

```
// Faber
fixum result = items.filtrata(pro x redde x > 0).mappata(pro x redde x * 2).prima(5)
```

**C++23:**

```cpp
auto result = items
    | std::views::filter([](auto x) { return x > 0; })
    | std::views::transform([](auto x) { return x * 2; })
    | std::views::take(5)
    | std::ranges::to<std::vector>();
```

## Async/Coroutines

C++20 coroutines map to `futura`/`cede`:

```
// Faber
futura functio fetch(textus url) fiet textus {
    fixum response = cede httpGet(url)
    redde response.body
}
```

**C++23:**

```cpp
#include <coroutine>
#include <future>

auto fetch(std::string url) -> std::future<std::string> {
    auto response = co_await httpGet(url);
    co_return response.body;
}
```

Note: C++ coroutines require a coroutine framework (custom promise types or a library like cppcoro). This is more complex than the Faber syntax suggests.

## Memory Management

Unlike Zig/Rust, C++23 offers flexibility:

### Option 1: Stack/RAII (Preferred)

Most values live on the stack, RAII handles cleanup:

```cpp
auto process() -> void {
    std::vector<std::string> items;  // Stack, RAII cleanup
    items.push_back("hello");
}  // items destroyed here
```

### Option 2: Arena (For Zig/Rust Consistency)

For performance-critical code or consistency with other systems targets:

```cpp
#include <memory_resource>

auto process() -> void {
    std::pmr::monotonic_buffer_resource arena;
    std::pmr::vector<std::string> items{&arena};

    items.push_back("hello");
}  // Arena freed, all allocations gone
```

### Option 3: Smart Pointers

For heap allocation:

```cpp
auto p = std::make_unique<Persona>();  // novum Persona
auto shared = std::make_shared<Persona>();  // Shared ownership
```

**Recommendation:** Default to stack/RAII. Generate arena code only when explicitly requested or for very hot paths.

## Implementation Status

**Partially implemented.** Core language features and collection stdlib are working.

| Feature                | Status          | Notes                                     |
| ---------------------- | --------------- | ----------------------------------------- |
| Variables              | Implemented     | `auto` / `const auto`                     |
| Functions              | Implemented     | Trailing return type                      |
| Control flow           | Implemented     | `si`/`secus`, `dum`, `ex...pro`, `elige` |
| `genus`                | Implemented     | struct with fields, methods               |
| `pactum`               | Implemented     | C++20 concepts                            |
| Error handling         | **Implemented** | `tempta`/`cape`/`demum` via RAII          |
| `de`/`in` prepositions | Designed        | Not yet implemented                       |
| Lambdas                | Implemented     | `pro` and arrow syntax                    |
| Lista methods          | **Implemented** | 53 methods via C++23 ranges               |
| Tabula methods         | **Implemented** | 17 methods via `std::unordered_map`       |
| Copia methods          | **Implemented** | 15 methods via `std::unordered_set`       |
| `mori` (panic)         | **Implemented** | `std::abort()` with stderr message        |
| `creo` auto-merge      | **Implemented** | `if constexpr (requires {...})` pattern   |
| `praefixum`            | **Implemented** | Expression and block forms                |
| Async                  | Pending         | Coroutines, needs framework decision      |
| Arena                  | Optional        | `std::pmr` when needed                    |

## Build System

Generated projects should include CMakeLists.txt:

```cmake
cmake_minimum_required(VERSION 3.28)
project(faber_output CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(main main.cpp)
```

## Compiler Requirements

| Compiler | Minimum Version | Notes               |
| -------- | --------------- | ------------------- |
| GCC      | 13+             | Full C++23 support  |
| Clang    | 17+             | Full C++23 support  |
| MSVC     | 19.37+          | Most C++23 features |

## Comparison with Other Systems Targets

| Aspect        | Zig            | Rust               | C++23                              |
| ------------- | -------------- | ------------------ | ---------------------------------- |
| Error model   | Error unions   | `Result<T,E>`      | `std::expected<T,E>` or exceptions |
| Memory        | Arena required | Arena or ownership | RAII, optional arena               |
| `demum`       | Not supported  | Not supported      | RAII scope guards                  |
| Generics      | Comptime       | Monomorphized      | Templates                          |
| Interfaces    | Duck typing    | Traits             | Concepts                           |
| Async         | Frame-based    | Future + runtime   | Coroutines                         |
| String concat | Complex        | `format!`          | `std::format` / `+`                |
| Lambdas       | Context struct | Native closures    | Native lambdas                     |
| Build system  | `build.zig`    | Cargo              | CMake                              |

C++23 is arguably the **easiest** systems target because:

- Exceptions enable `tempta`/`demum` directly
- `std::string` just works
- RAII handles most cleanup
- Mature tooling

## Future Considerations

1. **Coroutine framework** - Need to decide on promise types or library dependency
2. **Module support** - C++20 modules could eliminate headers, but tooling is still maturing
3. **`std::generator`** - C++23 adds `std::generator<T>` for `cursor` functions
4. **Error type** - Define `FaberError` or use `std::error_code`
5. **Header generation** - Decide on header-only vs `.hpp`/`.cpp` split
6. **Format library** - `std::format` vs `fmt` library for broader compatibility
