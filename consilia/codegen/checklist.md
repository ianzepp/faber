# Codegen Target Checklist

Status key: `[x]` implemented, `[~]` partial, `[ ]` not implemented, `[-]` not applicable to target

## Type System

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
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

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `varia` (mutable) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `fixum` (immutable) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `figendum` (async immutable) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `variandum` (async mutable) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `nexum` (reactive binding) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Type annotations | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Object destructuring | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Initializer expressions | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Enum Declarations

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `ordo` (enum) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Enum variants | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Enum with values | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Function Declarations

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| Basic functions (`functio`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Parameters | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Parameter type annotations | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Return type annotation | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `futura` (async) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `cursor` (generator) | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| Async generator | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| Arrow functions | [x] | [~] | [x] | [ ] | [ ] | [ ] |

## Control Flow Statements

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `si` (if) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `aliter` (else) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `aliter si` (else if) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `sin` (else if, poetic) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `dum` (while) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...pro` (for-of) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `in...pro` (for-in) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...fit` (for-of verb form) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...fiet` (async for) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| Range expressions (`0..10`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Range with step (`per`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `cum` (with/context) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `elige` (→ if/else) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `quando` (switch case) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Switch cases (`si`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Switch default (`aliter`) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `secus` (else/ternary alt) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `fac` (do/block) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `ergo` (then, one-liner) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `rumpe` (break) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `perge` (continue) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `custodi` (guard) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Catch on control flow | [x] | [ ] | [x] | [ ] | [ ] | [ ] |

## Return/Exit Statements

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `redde` (return) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `redde` with value | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `redde` void | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Exception Handling

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `tempta` (try) | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| `cape` (catch) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `demum` (finally) | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| `iace` (throw) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `adfirma` (assert) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Assert with message | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `mori` (panic/fatal) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Output/Debug/Events

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `scribe` statement | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `vide` (debug) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `mone` (warn) | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `emitte` (emit event) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `ausculta` (event stream) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Multiple args | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Expressions

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| Identifiers | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ego` (this/self) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
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
| `sic`/`secus` ternary syntax | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `cede` (await/yield) | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| `novum` (new) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `novum...cum` (new with props) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `est` (strict equality) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `aut` (logical or) | [x] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Arrow/Lambda Operators

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `fit` (-> sync single) | [~] | [~] | [~] | [ ] | [ ] | [ ] |
| `fiet` (async ->) | [~] | [ ] | [~] | [ ] | [ ] | [ ] |
| `fiunt` (yields ->, generator) | [~] | [-] | [~] | [ ] | [ ] | [ ] |
| `fient` (async yields ->) | [~] | [-] | [~] | [ ] | [ ] | [ ] |

## OOP Features (genus/pactum)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `genus` declaration | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Field declarations | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Field defaults | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Computed fields (getters) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Static fields (`generis`) | [x] | [ ] | [~] | [ ] | [ ] | [ ] |
| Public/private visibility | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `creo` (constructor hook) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `deleo` (destructor) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `pingo` (render method) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Auto-merge constructor | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Methods | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Async methods | [x] | [~] | [x] | [ ] | [ ] | [ ] |
| Generator methods | [x] | [-] | [x] | [ ] | [ ] | [ ] |
| `implet` (implements) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Generic classes | [x] | [ ] | [x] | [ ] | [ ] | [ ] |
| `pactum` declaration | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| Interface methods | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Import/Export

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `ex...importa` (named imports) | [x] | [x] | [x] | [ ] | [ ] | [ ] |
| `ex...importa *` (wildcard) | [x] | [x] | [x] | [ ] | [ ] | [ ] |

## Preamble / Prologue

Feature-dependent setup code (imports, includes, class definitions). See `consilia/codegen/preamble.md`.

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| Preamble infrastructure | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Panic class/import | [ ] | [-] | [ ] | [ ] | [-] | [ ] |
| Collection imports | [-] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Async imports | [-] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Arena allocator | [-] | [ ] | [-] | [ ] | [ ] | [ ] |

## Standard Library Intrinsics

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
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

| Latin | TypeScript | Zig | Python | WASM | Rust | C++23 |
|-------|:----------:|:---:|:------:|:----:|:----:|:-----:|
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

## Collection DSL (Future)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `ex items filtra ubi...` | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `cum property` shorthand | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `{ .property }` implicit | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `cum X descendens` sort | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Stdlib: File I/O (fasciculus)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `lege path` (read file) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `lege path ut format` (read+parse) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `inscribe path, data` (write) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `appone path, data` (append) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Stdlib: Filesystem (solum)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `exstat path` (exists) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `dele path` (delete) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `crea path` (mkdir) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `elenca path` (list dir) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `via.iunge` (path join) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Stdlib: Network (caelum)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `pete url` (HTTP GET) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `mitte url, body` (HTTP POST) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Response `.corpus()/.textus()/.json()` | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Stdlib: Time (tempus)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `nunc()` (current epoch) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `dormi ms` (sleep) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| Duration constants | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Stdlib: Crypto

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `digere data, algo` (hash) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `fortuita n` (random bytes) | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |

## Stdlib: Resource Management (cura)

| Feature | TypeScript | Zig | Python | WASM | Rust | C++23 |
|---------|:----------:|:---:|:------:|:----:|:----:|:-----:|
| `cura acquire fit binding {}` | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |
| `curator` interface | [ ] | [ ] | [ ] | [ ] | [ ] | [ ] |

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

### Zig Target Notes

1. **No classes** - Uses `struct` with methods; `genus` → `const Name = struct { ... };`
2. **No interfaces** - Zig uses duck typing, but Faber's semantic analyzer enforces `pactum`/`implet` before codegen (like TypeScript → JavaScript). Zig receives already-validated code.
3. **No constructors** - Uses `init()` pattern; auto-merge via comptime `@hasField`
4. **Self reference** - `const Self = @This();` for methods to reference own type
5. **No exceptions** - Error unions (`!T`) and `try` for propagation; `iace` → `@panic`
6. **No generators** - Would need custom iterator struct pattern
7. **No async/await** - Has frame-based async, but error unions more idiomatic
8. **Comptime generics** - Generic classes would need `fn(comptime T: type)` pattern
9. **No visibility modifiers** - All struct fields public; `pub` only for functions
10. **String handling** - `[]const u8` slices; no `+` concatenation at runtime

**Current gaps**:
- Static fields (`generis`) - would need module-level const
- Public/private - Zig doesn't have field visibility
- Generic classes - requires comptime type parameters

**Minimum version**: Zig 0.11+ (stable)

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

### C++23 Target Notes

1. **`std::expected<T,E>`** - Rust-like error handling; `iace` → `std::unexpected`
2. **`std::print`** - Clean output without iostream complexity
3. **Concepts** - `pactum` → C++20 concepts for compile-time interfaces
4. **`if constexpr (requires {})`** - Auto-merge constructor field checking
5. **Ranges** - `std::views::transform`, `filter` for lista methods
6. **Coroutines** - `futura`/`cede` → `co_await`/`co_yield`
7. **RAII** - `demum` works via scope guards (unlike Zig/Rust)
8. **`std::string`** - Just works, no ownership complexity
9. **`std::string_view`** - `de textus` → borrowed string view
10. **Templates** - Generics via templates with concepts constraints

**Alignment with Zig/Rust**:
- `de`/`in` prepositions map to `const T&`/`std::string_view` and `T&`
- `std::expected` mirrors Rust's `Result<T,E>` and Zig's error unions
- Arena optional via `std::pmr::monotonic_buffer_resource`

**Minimum version**: C++23 (GCC 13+, Clang 17+, MSVC 19.37+)

## Implementation Priority Suggestion

| Target | Priority | Rationale |
|--------|----------|-----------|
| TypeScript | Done | Primary target, web-first |
| Python | Done | Popular, good for teaching Latin syntax |
| Zig | In Progress | Systems programming, educational |
| C++23 | Medium | Bridges to C++ ecosystem, aligns with Rust/Zig patterns |
| Rust | Medium | Memory safety, systems programming |
| WASM | Low | Browser runtime, complex; consider Zig backend |
