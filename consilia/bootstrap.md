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
| Basic error handling       |        [x]         | `iace`/`mori` work; see §5 for design |

### P1: Highly Desirable

Would make the bootstrap significantly cleaner.

| Feature                   | Current Zig Status | Notes                                 |
| ------------------------- | :----------------: | ------------------------------------- |
| Object destructuring      |        [x]         | Cleaner AST unpacking                 |
| Array destructuring       |        [x]         | Multi-value returns                   |
| `copia<T>` (sets)         |        [~]         | Keyword sets, visited node tracking   |
| `cura...fit` (resources)  |        [x]         | Allocator scoping                     |
| `custodi` (guard clauses) |        [x]         | Early returns in parser               |
| `fac...cape` (scoped try) |        [~]         | Emits comment; use Zig idiom instead  |
| `adfirma` (assert)        |        [x]         | Invariant checks                      |
| Default parameter values  |        [~]         | Faber has `vel`; Zig fix is ~20 lines |
| `prae` (comptime params)  |        [x]         | Generic type functions                |
| `typus` (type aliases)    |        [x]         | Cleaner type names                    |

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

## Implementation Notes

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

### 5. Error Handling (DESIGN DECISION)

**Problem:** `tempta...cape` maps poorly to Zig's error model.

**Current state:** Zig codegen emits comments for `tempta`/`fac...cape` blocks because Zig uses error unions (`!T`) and `catch` on expressions, not try/catch blocks. This is intentional, not a gap.

**Why this is fine for bootstrap:**

Zig's error model is fundamentally different:
- Functions return `!T` (value or error)
- Errors are caught at expression level: `riskyCall() catch |err| { ... }`
- No exception unwinding — errors are explicit values

The bootstrap compiler should use Zig-idiomatic patterns:

```faber
// Instead of tempta/cape, use error-returning functions
functio parseToken() fit Token! {
    custodi source[current] != nihil aliter iace ParseError.UnexpectedEnd
    // ... parsing logic
}

// Collect errors in a lista passed as parameter
functio parse(lista<ParseError> errors) fit Program {
    fixum token = parseToken() cape err {
        errors.adde(err)
        redde nihil
    }
}
```

The `!` suffix on return types and `cape` on expressions (not blocks) would need implementation, but the current `iace`/`mori` + `lista<Error>` pattern works.

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

### Phase 4: Error Handling (Deferred)

Zig error handling is fundamentally different from try/catch. The `tempta`/`fac...cape` constructs emit comments intentionally. For bootstrap, use:

- [x] `iace` / `mori` — work correctly
- [x] `lista<Error>` accumulation — works with existing collection support
- [ ] Expression-level `cape` — would need `expr cape err { }` syntax (optional)
- [ ] `!T` return type suffix — would enable Zig error unions (optional)

The current primitives are sufficient for bootstrap. Zig-idiomatic error handling can be added later.

### Phase 5: Bootstrap

See detailed analysis below.

- [x] **5a:** Core types — AST nodes, Token, SemanticType as `genus`/`discretio`
- [x] **5b:** Tokenizer — Refactor closures → struct (~1,200 lines)
- [ ] **5c:** Parser — Largest module (~5,000 lines), complex state
- [ ] **5d:** Semantic — Symbol tables, type checking (~2,000 lines)
- [ ] **5e:** Codegen (Zig only) — Handler functions (~3,500 lines)
- [ ] **5f:** CLI — Arg parsing, orchestration (~600 lines)
- [ ] **5g:** Integration — Compile, debug, iterate until self-hosting

## Phase 5 Detailed Analysis

### Source Code Metrics

| Module | Lines (non-test) | Files | Purpose |
|--------|------------------|-------|---------|
| `parser/` | 7,630 | 4 | AST types (2,318) + parser (4,905) |
| `semantic/` | 2,630 | 4 | Type checking, scope binding |
| `tokenizer/` | 1,223 | 3 | Lexical analysis |
| `lexicon/` | ~1,400 | 6 | Keywords, types, nouns |
| `codegen/zig/` | 3,528 | ~50 | Zig-only codegen |
| `prettier/` | 1,618 | 5 | Formatter (skip for bootstrap) |
| `cli.ts` | 583 | 1 | Entry point |
| **Bootstrap total** | **~17,000** | | Excluding prettier, other targets |

### AST Complexity

| Type | Count | Notes |
|------|-------|-------|
| Statement variants | 34 | Large `discretio` needed |
| Expression variants | 26 | Another large `discretio` |
| Supporting types | ~14 | Parameters, patterns, etc. |
| **Total AST nodes** | **~74** | All need `genus` definitions |

Parser has ~65 parsing functions. Semantic analyzer has ~30 analysis functions.

### Structural Refactoring Required

**1. Closure-Heavy Design → Struct Methods**

All three core modules use nested functions closing over mutable state:

```typescript
// Current TypeScript pattern
function tokenize(source: string) {
    let current = 0;
    let tokens: Token[] = [];
    function advance() { return source[current++]; }
    // ... 20+ nested functions
}
```

Must become explicit struct with methods:

```faber
genus Tokenizer {
    textus source
    numerus current
    lista<Token> tokens

    publicum functio advance() fit textus {
        fixum c = ego.source[ego.current]
        ego.current = ego.current + 1
        redde c
    }
}
```

This is mechanical but touches every function in tokenizer, parser, and semantic modules.

**2. Large Discriminated Unions**

TypeScript uses discriminated unions with `type` field:

```typescript
type Statement = ImportaDeclaration | VariaDeclaration | ... // 34 types
switch (node.type) {
    case 'ImportaDeclaration': ...
}
```

Faber equivalent using `ordo` + `discretio`:

```faber
ordo NodeKind {
    ImportaDeclaration
    VariaDeclaration
    // ... 34 members
}

discretio Statement {
    ImportaDeclaration { specifiers: lista<ImportSpecifier>, ... }
    VariaDeclaration { name: textus, init: Expression?, ... }
    // ... 34 variants
}
```

Works but verbose. Consider grouping into categories (declarations, control flow, etc.) for ergonomics.

**3. Property Optionality**

TypeScript: `returnType?: TypeAnnotation`

Faber: Use nullable `T?` types with explicit null checks:

```faber
genus FunctioDeclaration {
    TypeAnnotation? returnType  // nullable
}

// Usage requires explicit check
si node.returnType != nihil {
    fixum rt = node.returnType!  // force unwrap after check
}
```

**4. No Literal String Types**

TypeScript discriminants like `type: 'Identifier'` become enum values:

```faber
ordo NodeKind {
    Identifier
    BinaryExpression
    CallExpression
    // ... 60+ members
}
```

### What Ports Easily

| Feature | Notes |
|---------|-------|
| Tokenizer (no regex) | Already uses char-by-char, range checks |
| Switch dispatch | `elige` maps directly |
| Error accumulation | `lista<Error>` pattern works |
| Symbol tables | `tabula<textus, Symbol>` ready |
| Recursive tree walking | Standard recursion |
| stdin/stdout I/O | `lege()`/`scribe` implemented |

### Blocking Issues

| Issue | Severity | Fix |
|-------|----------|-----|
| Default parameters | Low | ~20 line codegen fix (see below) |
| Regex (1 use) | Low | Replace with character loop |
| String `.trim()` | Low | Implement as helper function |
| `tempta...cape` | N/A | Not a gap; use `iace`/`mori` + `lista<Error>` |

**Default parameters:** Faber supports `vel` for defaults:

```faber
functio greet(textus name vel "World") fit textus { ... }
```

TypeScript emits `name: string = "World"` correctly. **Zig codegen currently ignores `vel`** because Zig doesn't have default parameter syntax. However, this is a **~20 line codegen fix**:

```zig
// Faber: functio peek(numerus offset vel 0) fit Token
// Current Zig output (wrong): fn peek(offset: i64) Token { ... }
// Fixed Zig output:
fn peek(offset_param: ?i64) Token {
    const offset = offset_param orelse 0;
    // ... rest of body
}
```

The fix: in `genFunctioDeclaration`, detect params with `defaultValue`, emit them as optional (`?T`), and inject `const name = name_param orelse <default>;` at the start of the body. Not a blocker.

### Effort Estimate

| Phase | Scope | Days |
|-------|-------|------|
| 5a: Core Types | AST nodes, Token, types as `genus`/`discretio` | 2-3 |
| 5b: Tokenizer | Refactor closures → struct | 2-3 |
| 5c: Parser | Largest module, complex state | 5-7 |
| 5d: Semantic | Symbol tables, type checking | 3-4 |
| 5e: Codegen | Zig handlers only | 4-5 |
| 5f: CLI | Arg parsing, orchestration | 1 |
| 5g: Integration | Debug, iterate | 3-5 |
| **Total** | | **20-28 days** |

### Recommended Bootstrap Strategy

**Minimal viable compiler first:**

1. **Skip `prettier/`** — Not needed for compilation
2. **Skip other codegen targets** — Only emit Zig
3. **Defer semantic analysis** — Emit untyped Zig initially, let Zig compiler catch type errors
4. **Add semantics incrementally** — Once basic pipeline works

This reduces initial scope to ~12,000 lines (tokenizer + parser + zig codegen + cli).

**Incremental self-hosting:**

1. Write Faber source for one module (e.g., tokenizer)
2. Compile to Zig using TypeScript compiler
3. Verify Zig output compiles and runs correctly
4. Repeat for parser, then codegen
5. Once all modules work, compile the Faber compiler with itself

**Parallel development path:**

```
TypeScript compiler (existing)
        ↓ compiles
Faber source (new)
        ↓ produces
Zig source
        ↓ zig build
Native binary
        ↓ compiles (same Faber source)
Zig source (must match above)
```

Round-trip verification: both compilers produce identical Zig output.

## Metrics for Readiness

**Current state:** 476 Zig tests pass (0 failures). Test coverage is 61% (385 Zig expectations vs 632 TypeScript expectations). Tests without Zig expectations are skipped, not failed.

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
| 2025-12-30 | Phase 5 detailed analysis        | ~17k lines, 20-28 days estimated       |
| 2025-12-31 | Complete Phase 5a (AST types)    | 20 files in `fons-fab/ast/`            |
| 2025-12-31 | Complete Phase 5b (Tokenizer)    | `fons-fab/lexicon/` + `fons-fab/lexor/`|
| 2025-12-31 | `genus` self.alloc pattern       | Collections in structs get auto-alloc  |

## Key Implementation Patterns (from Phase 5a/5b)

These patterns emerged during bootstrap implementation and should guide remaining phases.

### Pattern 1: `curata` Function Modifier

Functions needing heap allocation use the `curata NAME` modifier after the parameter list. This declares that the function receives an allocator bound to NAME. Call sites must be inside a `cura` block:

```faber
functio lexare(textus fons) curata alloc -> LexorResultatum {
    // alloc is available here and passed to collection operations
    varia lista<Symbolum> symbola = [] qua lista<Symbolum>
    symbola.adde(symbolum)  // alloc auto-threaded
}

// Call site - alloc injected automatically from cura block
incipit ergo cura arena fit alloc {
    fixum result = lexare("source code")  // alloc from cura block
}
```

The modifier position (after params, before return type) matches `futura` and `cursor`:
- `functio fetch(textus url) futura -> Response` — async
- `functio range(numerus n) cursor -> numerus` — generator  
- `functio lexare(textus fons) curata alloc -> LexorResultatum` — managed

### Pattern 2: Struct `init()` with Allocator

For structs containing collections (`lista`, `tabula`, `copia`), use explicit `init()` that takes the allocator. The `init()` method uses `curata` to receive the allocator:

```faber
genus Lexor {
    textus fons
    lista<Symbolum> symbola
    
    publicum functio init(textus fons) curata alloc -> Lexor {
        redde Lexor.{
            fons: fons,
            symbola: [] qua lista<Symbolum>
        }
    }
}

// Usage - alloc comes from enclosing cura block
varia lexor = Lexor.init(fons)
```

The Zig codegen automatically:
- Adds `alloc: std.mem.Allocator` field to structs with collections
- `init()` stores the allocator as first operation
- Methods access `self.alloc` for collection operations

### Pattern 3: Closure → Genus Refactoring

TypeScript's closure-based modules become `genus` with methods. All state moves to struct fields:

```typescript
// TypeScript (before)
function tokenize(source: string) {
    let current = 0;
    function advance() { return source[current++]; }
    function peek() { return source[current]; }
    // ... many nested functions
}
```

```faber
// Faber (after)
genus Lexor {
    textus fons
    numerus index
    
    publicum functio procede() -> textus {
        fixum c = ego.fons[ego.index]
        ego.index = ego.index + 1
        redde c
    }
    
    publicum functio specta() -> textus {
        redde ego.fons[ego.index]
    }
}
```

### Pattern 4: Latin Naming Convention

All bootstrap code uses Latin identifiers, following the AST module convention:

| English | Latin | Usage |
|---------|-------|-------|
| `peek` | `specta` | Look without consuming |
| `advance` | `procede` | Move forward |
| `current` | `index` | Current position |
| `tokens` | `symbola` | Token list |
| `source` | `fons` | Source text |
| `error` | `error` | Error (Latin origin) |
| `new` | `novum` | Constructor (but reserved) |

**Note:** `novum` is a keyword in Faber - use `init` for constructor methods.

### Pattern 5: Result Types for Error Handling

Instead of exceptions, return result types containing either success or errors:

```faber
genus LexorResultatum {
    lista<Symbolum> symbola
    lista<LexorError> errores
    bivalens successum
}

functio lexare(curator alloc, textus fons) -> LexorResultatum {
    varia errores = [] qua lista<LexorError>
    varia symbola = [] qua lista<Symbolum>
    
    // ... tokenization logic, accumulate errors
    
    redde LexorResultatum.{
        symbola: symbola,
        errores: errores,
        successum: errores.longitudo == 0
    }
}
```

### Pattern 6: Switch via `elige` for Keyword Lookup

Large keyword tables use `elige` which compiles to efficient switch statements:

```faber
functio estVerbum(textus verbum) -> SymbolumGenus? {
    elige verbum {
        si "si" { redde SymbolumGenus.Si }
        si "aliter" { redde SymbolumGenus.Aliter }
        si "dum" { redde SymbolumGenus.Dum }
        // ... 80+ keywords
        secus { redde nihil }
    }
}
```

This avoids `tabula` allocation for static lookups.

## Current Bootstrap Structure

```
fons-fab/
├── ast/                    # Phase 5a - COMPLETE (20 files)
│   ├── positio.fab         # Locus, Tractus (position types)
│   ├── lexema.fab          # Symbolum, SymbolumGenus (token types)
│   ├── radix.fab           # Programma root node
│   ├── sententia/          # Statement AST nodes (12 files)
│   │   ├── importa.fab     # Import declarations
│   │   ├── varia.fab       # Variable declarations
│   │   ├── functio.fab     # Function declarations
│   │   └── ...
│   └── expressia/          # Expression AST nodes (6 files)
│       ├── litteralis.fab  # Literals
│       ├── binarius.fab    # Binary expressions
│       └── ...
├── lexicon/                # Phase 5b - COMPLETE
│   └── verba.fab           # estVerbum() keyword lookup (no heap)
└── lexor/                  # Phase 5b - COMPLETE
    ├── errores.fab         # LexorErrorCodice enum + helpers
    └── index.fab           # Lexor genus + lexare() entry point
```

## Codegen Fixes Applied

During Phase 5b, these Zig codegen issues were discovered and fixed:

### Fix 1: Default Curator in Methods

**Problem:** `getCurator()` threw errors when generating method bodies at module level (before any `cura` block established context).

**Solution:** Return `'alloc'` as default instead of throwing. Zig compiler catches missing allocators at actual call sites.

```typescript
// fons/codegen/zig/generator.ts
getCurator(): string {
    const curator = this.curatorStack[this.curatorStack.length - 1];
    return curator ?? 'alloc';  // Was: throw Error(...)
}
```

### Fix 2: Self-Allocator Pattern for Genus

**Problem:** Methods inside a `genus` with `lista<>` fields couldn't access an allocator for collection operations.

**Solution:** Structs with collection fields automatically:
1. Add `alloc: std.mem.Allocator` field
2. `init()` takes allocator as first param and stores it
3. Methods push `self.alloc` onto curatorStack during codegen

This is implemented in `fons/codegen/zig/statements/genus.ts`.
````
