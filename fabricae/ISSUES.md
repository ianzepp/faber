# Fabricae Issues

Known issues and limitations discovered while building the demo projects.

---

## TypeScript (ts/hono-demo)

### Working

- [x] Async lambdas with `fiet`
- [x] Sync lambdas with `pro`/`fit`
- [x] Imports from external packages
- [x] Object literals
- [x] Collection methods (inveni, adde, inveniIndicem)
- [x] Null checks (`=== nihil`)

### Issues

- [ ] **No export syntax** — Faber has no `exporta` for default exports. Workaround: append export manually in build script.

- [ ] **`.toString()` not recognized** — `nextId.toString()` compiles to `nextId.undefined()`. Workaround: use string concatenation `"" + nextId`.

- [ ] **`.filtra()` generates ugly IIFE** — In-place filter generates:

    ```ts
    (() => {
        for (let i = arr.length - 1; i >= 0; i--) {
            if (!pred(arr[i])) arr.splice(i, 1);
        }
    })();
    ```

    Should use `arr = arr.filter(pred)` or native splice loop without IIFE.

- [ ] **Type inference for empty arrays** — `varia users = []` fails. Must use `[] qua T[]`.

---

## Zig (zig/http-demo)

### Working

- [x] Pointer params via `in T` → `*T`
- [x] Const pointer params via `de T` → `*const T`
- [x] Struct definitions with `init()` pattern
- [x] Typed lambdas with `-> vacuum`
- [x] String comparison via `std.mem.eql`
- [x] Object literals → `.{ .field = value }`

### Issues

- [ ] **Duplicate `const std = @import("std")`** — When using `ex "std" importa *`, std is imported twice. First from always-included preamble, second from explicit import.

- [ ] **Mutable globals scoping bug** — `varia numerus nextId = 1` at module level ends up inside `main()`, but functions reference it at module scope. Causes undefined variable error.

    ```zig
    fn generateId() i64 {
        const id = nextId;  // Error: nextId not in scope
        ...
    }
    pub fn main() void {
        var nextId: i64 = 1;  // Declared here, too late
    }
    ```

- [ ] **Method mutability** — Struct methods always get `self: *const Self`, even when they mutate `self`. Should detect mutations and use `*Self`.

- [ ] **Lambda params use `anytype`** — Typed lambdas generate `fn call(x: anytype, y: anytype)` instead of concrete types. Works but less type-safe.

- [ ] **Server init pattern mismatch** — `novum Server { port: 3000 }` generates `Server.init(.{ .port = 3000 })`, but httpz needs `Server(*App).init(allocator, config, &app)`.

- [ ] **Error union returns not expressible** — No syntax for `!void` or `!T` return types. Functions that can fail need manual annotation.

- [ ] **No allocator passing syntax** — Zig idiom requires explicit allocator params. Faber auto-generates arena in `main()` preamble, but can't pass to functions.

---

## C++ (cpp/http-demo)

### Working

- [x] Reference params via `in T` → `T&`
- [x] Const reference params via `de T` → `const T&`
- [x] Struct definitions with C++20 concepts
- [x] Lambda capture with `[&]`
- [x] Object literals → designated initializers `.field = value`
- [x] `std::print` for output

### Issues

- [ ] **Crow API mismatch** — Generated code uses `server.route("/path").get(...)` but Crow uses `CROW_ROUTE(app, "/path")` macro or different method signatures.

- [ ] **`.delete()` is reserved** — Generated `server.route(...).delete(...)` fails because `delete` is a C++ keyword. Crow uses `.methods("DELETE"_method)`.

- [ ] **Object literal return type** — `userToJson()` returns `std::any` but should return `crow::json::wvalue` or similar.

- [ ] **No exception handling codegen** — `tempta`/`cape` not tested in C++ target. May need `try`/`catch` with proper exception types.

- [ ] **Missing includes** — Generated code includes `<crow>` but should be `<crow.h>` or `"crow_all.h"` depending on installation.

- [ ] **Designated initializers require aggregate** — Object literals compile to `{.field = value}` which requires the target type to be an aggregate. May fail with Crow's JSON types.

---

## Cross-Target Issues

### Prepositions

- [ ] **`de`/`in` ignored in TS/Py** — Prepositions only affect Zig and C++. TypeScript and Python ignore them entirely (no reference semantics).

### Imports

- [ ] **Imported symbols not in semantic scope** — `ex "module" importa foo` doesn't add `foo` to semantic checker's scope. Causes "undefined variable" errors when using imported names.

### Collections

- [ ] **Method names vary by target** — `lista.adde()` maps to different methods:
    - TS: `.push()`
    - Zig: `.append(alloc, x)`
    - C++: `.push_back()`

    Some methods may not have equivalents across all targets.

### Async

- [ ] **`fiet` lambdas only work in TS** — Zig and C++ don't have async lambdas in the same sense. `fiet` generates `async () =>` in TS but has no effect in systems targets.

---

## Priority

**High** (blocks real usage):

1. Mutable globals scoping in Zig
2. Imported symbols not in semantic scope
3. Export syntax for TS

**Medium** (workarounds exist): 4. Method mutability detection in Zig 5. Crow API adjustments for C++ 6. Duplicate std import in Zig

**Low** (cosmetic): 7. IIFE for filtra 8. Lambda param types in Zig
