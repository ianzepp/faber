# Bootstrap: Zig as Target for Self-Hosted Faber Compiler

This document defines the minimum Zig codegen requirements to compile a Faber-rewritten version of the Faber compiler itself.

## Rationale: Why Zig over Rust

1. **No ownership inference required** - Faber has no borrow checker semantics. Emitting valid Rust requires either conservative cloning (slow) or complex ownership inference (hard). Zig's manual memory model maps directly.

2. **Existing infrastructure** - `subsidia/zig/` contains working Zig stdlib bindings. The Zig codegen has 61% test coverage (385 Zig vs 632 TS expectations).

3. **Faster compile times** - Self-hosted compiler edit-compile-test cycles benefit from Zig's speed.

4. **Simpler FFI** - If we need C library interop (e.g., tree-sitter), Zig's seamless C ABI is trivial.

## Compiler Architecture Overview

The current compiler (`fons/`) has these modules:

| Module       | Purpose                      | Key Data Structures         |
| ------------ | ---------------------------- | --------------------------- |
| `lexicon/`   | Keyword/type definitions     | String constants, enums     |
| `tokenizer/` | Source text -> tokens        | Token structs, position     |
| `parser/`    | Tokens -> AST                | ~50 AST node types          |
| `semantic/`  | Type checking, scope binding | Symbol tables, scope stacks |
| `codegen/`   | AST -> target source         | String building, recursion  |
| `prettier/`  | AST -> formatted Faber       | Same as codegen             |
| `cli.ts`     | Orchestration                | File I/O, arg parsing       |

## Feature Requirements by Priority

### P0: Absolutely Required

These features are non-negotiable for a working compiler.

| Feature                    | Current Zig Status | Notes                                 |
| -------------------------- | :----------------: | ------------------------------------- |
| `textus` (string)          |        [x]         | Heavy use for source code, output     |
| `numerus` (integer)        |        [x]         | Line/column numbers, indices          |
| `bivalens` (boolean)       |        [x]         | Flags, conditionals                   |
| `nihil` / nullable `T?`    |        [x]         | Optional AST children                 |
| `genus` (struct)           |        [x]         | AST nodes, tokens, errors             |
| `ordo` (enum)              |        [x]         | Token types, AST node types           |
| `discretio` (tagged union) |        [x]         | AST node variants                     |
| `functio` (functions)      |        [x]         | Every compiler pass                   |
| Methods on `genus`         |        [x]         | Parser methods, generator methods     |
| `si`/`aliter` (if/else)    |        [x]         | Conditional logic everywhere          |
| `dum` (while)              |        [x]         | Token scanning loops                  |
| `ex...pro` (for-of)        |        [x]         | Iterating AST children, token streams |
| `elige` (switch)           |        [x]         | Node type dispatch (critical)         |
| `discerne` (match)         |        [x]         | Pattern matching on tagged unions     |
| `redde` (return)           |        [x]         | Function returns                      |
| `rumpe`/`perge`            |        [x]         | Loop control                          |
| `lista<T>` (arrays)        |        [x]         | Token arrays, AST child lists         |
| `tabula<K,V>` (maps)       |        [x]         | Symbol tables, keyword lookups        |
| String concatenation       |        [~]         | Output building (needs allocator)     |
| `scriptum()` formatting    |        [x]         | Error messages, code output           |
| I/O streams                |        [x]         | stdin/stdout/stderr all work          |
| Basic error handling       |        [~]         | `iace`/`mori` work, `tempta` doesn't  |

### P1: Highly Desirable

Would make the bootstrap significantly cleaner.

| Feature                   | Current Zig Status | Notes                               |
| ------------------------- | :----------------: | ----------------------------------- |
| Object destructuring      |        [x]         | Cleaner AST unpacking               |
| Array destructuring       |        [x]         | Multi-value returns                 |
| `copia<T>` (sets)         |        [~]         | Keyword sets, visited node tracking |
| `cura...fit` (resources)  |        [x]         | Allocator scoping                   |
| `custodi` (guard clauses) |        [x]         | Early returns in parser             |
| `fac...cape` (scoped try) |        [x]         | Local error handling                |
| `adfirma` (assert)        |        [x]         | Invariant checks                    |
| Default parameter values  |        [ ]         | Parser options (can work around)    |
| `prae` (comptime params)  |        [x]         | Generic type functions              |
| `typus` (type aliases)    |        [x]         | Cleaner type names                  |

### P2: Nice to Have

Can be worked around but would improve code quality.

| Feature                 | Current Zig Status | Notes                          |
| ----------------------- | :----------------: | ------------------------------ |
| Lambdas / closures      |        [x]         | Works with type inference      |
| `sub` (inheritance)     |        [ ]         | Not needed - use composition   |
| `implet` (implements)   |        [x]         | Interface contracts            |
| Regex (`sed`)           |        [ ]         | Tokenizer - use manual parsing |
| `nexum` (reactive)      |        [ ]         | Not needed for compiler        |
| Async (`futura`/`fiet`) |        [ ]         | Compiler is synchronous        |
| Generators (`fiunt`)    |        [-]         | Zig has no generators          |

### Not Required

Features explicitly not needed for bootstrap.

| Feature               | Reason                      |
| --------------------- | --------------------------- |
| `decimus` (decimal)   | No decimal math in compiler |
| `magnus` (bigint)     | Token positions fit in i64  |
| `promissum` (promise) | Compiler is synchronous     |
| `cursor` (iterator)   | Use `ex...pro` loops        |
| HTTP/Network          | Compiler is offline         |
| Crypto                | No cryptographic operations |
| Compression           | No compressed I/O           |
| Database              | No persistence              |

## Critical Gaps to Address

### 1. String Operations (SOLVED)

**Problem:** String concatenation at runtime requires allocator threading in Zig.

**Current state:** Zig codegen **throws an error** on `textus a + textus b`, directing users to `scriptum()`. This is correct behavior.

**Required for:** Building output code strings, error messages.

**Solution:** Already implemented correctly. Use `scriptum()` exclusively for all string building:

```faber
// Instead of: fixum msg = "Error at " + line + ":" + col
fixum msg = scriptum("Error at {}:{}", line, col)
```

The bootstrap compiler must be written using `scriptum()` for all string construction. This is the Zig-idiomatic approach and already enforced by codegen.

### 2. Collection Methods (SOLVED)

**Problem:** Functional methods exist but had integration issues.

**Current state:** All major blockers resolved.

- `subsidia/zig/lista.zig` runtime **exists** with `filtrata`, `mappata`, `reducta`, etc.
- Codegen emits calls like `nums.filtrata(alloc, predicate)`
- **FIXED:** Array literals with `lista<T>` type emit `Lista(T).fromItems(alloc, &.{...})`
- **FIXED:** Lambda return types inferred from semantic analysis when not annotated

**Example of current output:**

```faber
varia lista<numerus> nums = [1, 2, 3, 4, 5]
fixum filtered = nums.filtrata(pro x: x > 2)
```

```zig
var nums = Lista(i64).fromItems(alloc, &.{ 1, 2, 3, 4, 5 });
const filtered = nums.filtrata(alloc, struct { fn call(x: anytype) bool { return (x > 2); } }.call);
```

Lambda syntax uses colon (`pro x: expr`), not arrow. Return type is inferred from the expression via semantic analysis.

**Imperative alternative** (also works, sometimes clearer):

```faber
varia numerus[] filtered = [] qua numerus[]
ex items pro x {
    si x > 0 { filtered.adde(x) }
}
```

**Note:** Empty array literals still require `qua` cast: `[] qua numerus[]`

````

### 3. Hash Maps (SOLVED)

**Problem:** `tabula<textus, T>` works but complex operations don't.

**Current state:** All methods implemented:

- `pone`, `accipe`, `accipeAut`, `habet`, `dele` — core ops
- `longitudo`, `vacua`, `purga` — size/state
- `claves()`, `valores()`, `paria()` — iteration
- `confla()` — merge another map
- `inLista()` — convert to array of tuples

**Required for:** Symbol tables, keyword lookups, scope management. All needs covered.

### 4. I/O Streams (SOLVED)

**Problem:** Compiler needs to read input and write output.

**Solution:** Use stdin/stdout instead of file I/O. Simpler and more Unix-idiomatic.

| Stream | Faber    | Zig Output                              | Status |
| ------ | -------- | --------------------------------------- | :----: |
| stdout | `scribe` | `stdout.print(...)`                     |  [x]   |
| stderr | `mone`   | `stderr.print("[WARN] ...")`            |  [x]   |
| stdin  | `lege()` | `stdin.readAllAlloc(alloc, max)`        |  [x]   |

**Usage:**

```faber
// Read all input from stdin
fixum source = lege()

// Write output to stdout
scribe output

// Warnings to stderr
mone "Parse error at line 42"
```

**Invocation:**

```bash
cat input.fab | faber compile -t zig > output.zig
```

File I/O (`solum` module) is deferred — not needed for bootstrap.

### 5. Error Collection (MEDIUM)

**Problem:** `tempta...cape` not implemented for Zig (emits `[-]`).

**Current state:** `iace` and `mori` work. No structured error recovery.

**Required for:** Parser error recovery, collecting multiple errors.

**Solution for bootstrap:** Use Zig's native error handling:

- Functions return `!T` (error union)
- Collect errors in a `lista<Error>` passed as parameter
- Use `catch` at call sites to accumulate

This matches Zig idiom better than exception-style `tempta/cape`.

### 6. Unused Parameters (LOW)

**Problem:** Zig errors on unused function parameters.

**Current state:** ~8 test failures due to this.

**Required for:** Clean compilation.

**Solution:** Codegen should either:

- Prefix unused params with `_`
- Add `_ = param;` statements
- Requires tracking which params are used in body

## Bootstrap Subset Definition

The "bootstrap subset" of Faber is:

```
Types:       textus, numerus, bivalens, nihil, T?, vacuum
             genus, ordo, discretio
             lista<T>, tabula<textus, T>

Statements:  varia, fixum
             functio (sync only, no async/generators)
             incipit (entry point, required for Zig)
             si/aliter, dum, ex...pro, de...pro
             elige, discerne
             redde, rumpe, perge
             iace, mori, adfirma
             cura...fit (allocator scoping, REQUIRED for Zig allocations)
             scribe

Expressions: Literals, identifiers, ego
             Binary ops (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||)
             Member access (., ?., !.)
             Function calls
             Array/object literals
             scriptum() format strings
             novum...de construction
             qua (type cast)
             Ternary (sic/secus)
             Lambdas (pro x: expr)
```

**Zig-specific requirements:**

- Use `incipit ergo cura arena { ... }` as entry point (provides allocator context)
- All allocating operations (`lista`, `tabula`, `scriptum`) must be inside a `cura` block
- Lambda syntax is `pro x: expr` (colon, not arrow)

Explicitly excluded from bootstrap:

- Async (`futura`, `fiet`, `cede`)
- Generators (`fiunt`, `fient`)
- Closures capturing variables
- Inheritance (`sub`)
- Regex literals
- `nexum` reactive fields
- Default parameter values (use overloads or optional params)

## Implementation Roadmap

### Phase 1: Foundation (Current)

- [x] Basic types and control flow
- [x] `genus`/`ordo`/`discretio`
- [x] `elige`/`discerne` pattern matching
- [x] Allocator management (`cura`, `curator`)

### Phase 2: Collections (Complete)

- [x] Reliable `lista<T>` with imperative methods
- [x] Lambda type inference for functional methods
- [x] `tabula<textus, T>` with iteration (`claves()`, `valores()`, `paria()` work)
- [ ] Fix unused parameter warnings

### Phase 3: I/O (Complete)

- [x] `scribe` → stdout
- [x] `mone` → stderr
- [x] `lege()` → stdin

### Phase 4: Error Handling

- [ ] Error collection pattern (lista-based)
- [ ] Clean `!T` return type inference
- [ ] `fac...cape` scoped error handling

### Phase 5: Bootstrap

- [ ] Rewrite `fons/` in Faber using bootstrap subset
- [ ] Compile Faber compiler to Zig
- [ ] Self-host: compiled Zig compiler compiles itself

## Metrics for Readiness

**Current state:** 474 Zig tests pass (0 failures). Test coverage is 61% (385 Zig expectations vs 632 TypeScript expectations). Tests without Zig expectations are skipped, not failed.

Bootstrap is ready when:

1. **P0 features have Zig test coverage** - Add `zig:` expectations to remaining tests
2. **I/O works** - stdin/stdout/stderr functional
3. **Error collection works** - Can report multiple parse errors
4. **Self-compile succeeds** - `faber compile fons/*.fab -t zig` produces valid Zig
5. **Output compiles** - `zig build` succeeds on generated code
6. **Round-trip works** - Zig-compiled Faber produces identical output to TS version

## Alternative: Minimal Runtime

If full codegen proves too difficult, consider a minimal Zig runtime:

```zig
// runtime/faber.zig - copied alongside generated code
pub const Lista = @import("lista.zig").Lista;
pub const Tabula = @import("tabula.zig").Tabula;
pub const solum = @import("solum.zig");
```

This moves complexity from inline codegen to a testable Zig library. See `consilia/futura/zig-norma.md` for this approach.

## Decision Log

| Date       | Decision                         | Rationale                              |
| ---------- | -------------------------------- | -------------------------------------- |
| 2025-12    | Target Zig over Rust             | No ownership inference needed          |
| 2025-12    | Use `scriptum()` for strings     | Avoids runtime concat issues           |
| 2025-12    | Imperative-first bootstrap       | Functional methods are sugar           |
| 2025-12    | Error lists over exceptions      | Matches Zig idiom                      |
| 2025-12-28 | Add `incipit` entry point        | Required for Zig allocator context     |
| 2025-12-28 | Require explicit `cura` blocks   | Zig needs allocator for all heap ops   |
| 2025-12-30 | Remove arrow syntax from lambdas | Simplify to `pro x: expr` only         |
| 2025-12-30 | Lambda type inference            | Semantic analysis provides return type |
| 2025-12-30 | stdin/stdout over file I/O       | Simpler, more Unix-idiomatic           |
````
