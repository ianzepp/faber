# Codegen Target Checklist

Status key: `[x]` implemented, `[~]` partial, `[ ]` not implemented

## Type System

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `textus` (string) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `numerus` (number) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `bivalens` (boolean) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `nihil` (null) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `vacuum` (void) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `lista<T>` (array) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `tabula<K,V>` (map) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `copia<T>` (set) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `promissum<T>` (promise) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `erratum` (error) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `cursor<T>` (iterator) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Nullable types (`T?`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Union types (`T | U`) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Generic type params | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Type aliases (`typus`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Variable Declarations

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `varia` (mutable) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `fixum` (immutable) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Type annotations | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Object destructuring | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Initializer expressions | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Function Declarations

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| Basic functions (`functio`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Parameters | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Parameter type annotations | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Return type annotation | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `futura` (async) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `cursor` (generator) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Async generator | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Arrow functions | [x] | [~] | [x] | [ ] | [ ] | [ ] |

## Control Flow Statements

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `si` (if) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `aliter` (else) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `aliter si` (else if) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `dum` (while) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...pro` (for-of) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `in...pro` (for-in) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...fit` (for-of verb form) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...fiet` (async for) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Range expressions (`0..10`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Range with step (`per`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `cum` (with/context) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `elige` (switch) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Switch cases (`si`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Switch default (`aliter`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `custodi` (guard) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Catch on control flow | [x] | [ ] | [x] | [ ] | [ ] | [ ] |

## Return/Exit Statements

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `redde` (return) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `redde` with value | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `redde` void | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Exception Handling

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `tempta` (try) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `cape` (catch) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `demum` (finally) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `iace` (throw) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `adfirma` (assert) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Assert with message | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Output/Debug

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `scribe` statement | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Multiple args | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Expressions

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| Identifiers | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ego` (this/self) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Boolean literals (`verum`/`falsum`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `nihil` literal | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| String literals | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Number literals | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Template literals | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| Array literals | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Object literals | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Binary operators | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Comparison operators | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Logical operators | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Unary operators | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `nulla` (is empty) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `nonnulla` (has content) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `negativum` (is negative) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `positivum` (is positive) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Member access (`.`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Computed access (`[]`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Function calls | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Method calls | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Assignment | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Conditional (ternary) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `cede` (await/yield) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `novum` (new) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `novum...cum` (new with props) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |

## OOP Features (genus/pactum)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `genus` declaration | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Field declarations | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Field defaults | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Computed fields (getters) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Static fields (`generis`) | [x] | [ ] | [~] | [ ] | [ ] | [ ] |
| Public/private visibility | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `creo` constructor | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Auto-merge constructor | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Methods | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Async methods | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Generator methods | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `implet` (implements) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Generic classes | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `pactum` declaration | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Interface methods | [x] | [ ] | [x] | [ ] | [ ] | [ ] |

## Import/Export

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `ex...importa` (named imports) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...importa *` (wildcard) | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Standard Library Intrinsics

| Feature | TypeScript | Zig | Python | WASM | Rust | C++ |
|---------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `_scribe` (print) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `_vide` (debug) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_mone` (warn) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_lege` (read input) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_fortuitus` (random) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_pavimentum` (floor) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_tectum` (ceiling) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_radix` (sqrt) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `_potentia` (pow) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |

## Lista (Array) Methods

| Latin | TypeScript | Zig | Python | WASM | Rust | C++ |
|-------|:----------:|:---:|:------:|:----:|:----:|:---:|
| `adde` (push) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `addita` (push copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `praepone` (unshift) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `praeposita` (unshift copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `remove` (pop) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `remota` (pop copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `decapita` (shift) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `decapitata` (shift copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `purga` (clear) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `primus` (first) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `ultimus` (last) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `accipe` (at index) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `longitudo` (length) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `vacua` (is empty) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `continet` (includes) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `indiceDe` (indexOf) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `inveni` (find) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `inveniIndicem` (findIndex) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `filtrata` (filter) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `mappata` (map) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `reducta` (reduce) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `explanata` (flatMap) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `plana` (flat) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `inversa` (reverse copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `ordinata` (sort copy) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `sectio` (slice) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `prima` (take first n) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `ultima` (take last n) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `omitte` (skip first n) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `omnes` (every) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `aliquis` (some) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `coniunge` (join) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `perambula` (forEach) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |

## Target-Specific Considerations

### Python Target Notes

1. **No block braces** - Python uses indentation; codegen must track indent level
2. **No explicit types at runtime** - Type hints are optional annotations only
3. **No `new` keyword** - Classes are called directly: `MyClass()`
4. **Different async model** - Uses `asyncio`, needs event loop
5. **Different iterator model** - `__iter__`/`__next__` protocol
6. **Property decorators** - `@property` for getters
7. **Protocol classes** - `typing.Protocol` for interfaces (3.8+)
8. **Pattern matching** - `match`/`case` only in 3.10+
9. **Union types** - `X | Y` syntax only in 3.10+, else `Union[X, Y]`
10. **Type aliases** - `type X = Y` only in 3.12+, else `X = Y` or `TypeAlias`

**Minimum version**: Python 3.10+ (for `match`/`case` and `|` union syntax)

### WASM Target Notes

1. **No native strings** - Strings require memory management, often via linear memory
2. **No GC** - Manual memory management or use reference types proposal
3. **Limited types** - Only i32, i64, f32, f64 natively; structs via memory layout
4. **No exceptions** - Error handling via return values or trap
5. **No closures** - Function references exist but no captures
6. **Module imports** - Host functions for I/O (`console.log`, etc.)
7. **Table-based dispatch** - For dynamic calls and interfaces
8. **Component Model** - Future: richer types via WASM Component Model

**Approach**: Consider targeting WASM via Zig backend or using WAT text format directly

### Rust Target Notes

1. **Ownership system** - Every value has exactly one owner; moved on assignment
2. **Borrowing** - References via `&` (immutable) and `&mut` (mutable)
3. **No null** - Uses `Option<T>` for optional values, `Result<T, E>` for errors
4. **Lifetimes** - Explicit `'a` annotations when compiler can't infer reference validity
5. **Traits** - Like interfaces but with default implementations; no inheritance
6. **Pattern matching** - `match` is exhaustive; compiler enforces all cases covered
7. **Macros** - `println!`, `vec!`, `format!` - metaprogramming via macro_rules!
8. **Error handling** - No exceptions; uses `Result<T, E>` and `?` operator for propagation
9. **Iterators** - Lazy, composable, zero-cost via `.iter()`, `.map()`, `.filter()`
10. **Memory safety** - No data races, no dangling pointers, enforced at compile time

**Minimum version**: Rust 2021 edition (stable)

### C++ Target Notes

1. **Header/source split** - May need `.hpp`/`.cpp` file pairs
2. **Manual memory** - RAII, smart pointers (`unique_ptr`, `shared_ptr`)
3. **Templates** - For generics, but complex instantiation rules
4. **No reflection** - Type info limited without RTTI
5. **Exceptions** - `try`/`catch`/`throw`, but often avoided in perf-critical code
6. **STL containers** - `std::vector`, `std::map`, `std::set`, `std::string`
7. **Lambdas** - `[captures](params) { body }` syntax
8. **Async** - `std::async`, `std::future`, or coroutines (C++20)
9. **Modules** - C++20 modules vs traditional `#include`
10. **constexpr** - Compile-time evaluation similar to Zig's comptime

**Minimum version**: C++17 (structured bindings, `if constexpr`) or C++20 (coroutines, concepts)

## Implementation Priority Suggestion

| Target | Priority | Rationale |
|--------|----------|-----------|
| TypeScript | Done | Primary target, web-first |
| Zig | In Progress | Systems programming, educational |
| Python | High | Popular, good for teaching Latin syntax |
| Rust | Medium | Memory safety, systems programming |
| WASM | Medium | Browser runtime, but complex |
| C++ | Low | Complex, better served by Zig path |
