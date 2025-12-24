# Rust Target Notes

Rust presents unique challenges as a compilation target due to its ownership system. While many Faber constructs have natural Rust equivalents, the fundamental difference is that Rust requires explicit ownership reasoning. Faber addresses this through Latin prepositions (`de`, `in`) that map naturally to Rust's borrow semantics.

Rust and Zig share similar memory management concerns. Faber uses a **unified approach** for both targets: Latin prepositions (`de`, `in`) for borrowing semantics, and arena allocation for memory management. See zig.md for Zig-specific details.

## What Makes Rust Easier

### 1. `varia`/`fixum` → `let mut`/`let`

Direct mapping. Rust defaults to immutable (`let`), Faber's `fixum` maps cleanly. `varia` → `let mut`.

### 2. No Inheritance

Faber's `genus` doesn't support inheritance - only `implet`. Rust also has no inheritance, only traits. Perfect alignment.

### 3. `pactum` → `trait`

Rust has a direct equivalent for interfaces: traits. `pactum` becomes a `trait` definition, and `implet` becomes `impl Trait for Struct`.

**Interface enforcement:** Like all targets, interface conformance is checked in Faber's semantic analyzer before codegen. This is analogous to TypeScript → JavaScript: types are enforced at compile time, then erased. For Rust, the trait info is preserved in the output (unlike Zig where it becomes a comment), but Faber has already validated correctness.

### 4. Pattern Matching (`elige`)

Faber's `elige`/`si` switch maps well to Rust's `match`. Rust's match is more powerful (exhaustive, can destructure), but the basic mapping works.

### 5. Error Propagation

Faber's `cede` in async contexts → Rust's `?` operator. Both are "propagate or unwrap" semantics. The `?` operator is actually closer to what Faber wants than Zig's `try`.

### 6. `ego` → `self`

Same as Zig - Rust requires explicit `self` in methods. Clean transformation.

### 7. Iterators

Rust's iterator combinators (`.iter().map().filter().collect()`) are similar to Faber's lista methods (`mappata`, `filtrata`). The functional style aligns well.

### 8. `nihil` Handling

Faber's nullable types (`textus?`) map to Rust's `Option<String>`. Rust has first-class optional handling, unlike Zig where it's more manual.

## What Makes Rust Harder

### 1. Ownership (Solved via Prepositions)

Rust requires explicit ownership reasoning:
- Who owns this value?
- Should this be `&T`, `&mut T`, or `T`?
- When does this get dropped?

**Solution:** Latin prepositions annotate ownership on parameters. See "Ownership Design" section below. Without annotations, fallback to clone-all semantics.

### 2. Lifetimes (Solved via `de` on Return)

Rust needs lifetime annotations when returning borrowed values.

**Solution:** Use `de` on return type to indicate borrowed return tied to input lifetimes. Mirrors Rust's elision rules. See "Lifetimes via `de` on Return Type" section below.

### 3. String Types (Solved via Prepositions)

Rust has `String` (owned) vs `&str` (borrowed slice). Faber's `textus` maps to either based on preposition:
- `textus` → `String` (owned)
- `de textus` → `&str` (borrowed)

### 4. Generators

Rust has unstable generator syntax, but stable Rust requires manual `Iterator` impl with state structs - same problem as Zig.

### 5. Async Model

Rust's async is also different from JS. It's lazy (futures don't run until polled), needs a runtime (tokio, async-std), and has `Pin` complexity for self-referential futures. Faber's simple `futura`/`cede` hides all this.

### 6. `novum X cum { overrides }`

Rust doesn't have Zig's comptime `@hasField`. The auto-merge pattern would need:
- A builder pattern, or
- Default trait + struct update syntax: `Struct { field: value, ..Default::default() }`

The struct update syntax is close but requires `Default` impl.

## Rust vs Zig Comparison

| Aspect | Zig | Rust | Easier Target |
|--------|-----|------|---------------|
| Ownership/borrowing | `de`/`in` prepositions | `de`/`in` prepositions | Same (unified) |
| Memory management | Arena (`std.heap.ArenaAllocator`) | Arena (`bumpalo`) | Same (unified) |
| Interfaces | Duck typing (lossy) | Real traits | Rust |
| Error handling | Error unions | Result<T, E> | Similar |
| Generics | Comptime | Monomorphized | Similar |
| String handling | `de textus` → `[]const u8` | `de textus` → `&str` | Same (unified) |
| Auto-merge constructor | `@hasField` comptime | Needs builder/Default | Zig |
| Async | Frame-based | Future + runtime | Both hard |
| Ecosystem/tooling | Newer | Mature (cargo) | Rust |

## Ownership Design: Latin Prepositions

> **Note:** The `de` and `in` keywords are **systems-target-specific**. They are only valid in Faber projects configured to compile to Rust or Zig. TypeScript or Python targets would reject these as syntax errors. See thesis.md on target-specific features; see zig.md for Zig-specific mappings.

A key insight: **Rust's ownership model maps naturally to Latin grammatical cases.** Latin uses declensions (noun endings) to indicate relationships - subject, object, possession, recipient. These concepts align with Rust's owned, moved, borrowed, and mutably borrowed.

| Latin Case | Grammatical Role | Rust Ownership |
|------------|------------------|----------------|
| Nominative | Subject, the "doer" | Owned value (`T`) |
| Accusative | Direct object, acted upon | Moved/consumed (`T`) |
| Genitive | Possession, "of X" | Borrowed reference (`&T`) |
| Dative | Indirect object, "to/for X" | Mutable borrow (`&mut T`) |
| Ablative | "by/with/from X" | Source, transformed from |

### Primary Mechanism: Prepositions

Since variable names are often not Latin (e.g., `queryResult`, `app`, `userId`), we can't rely on declining them. Instead, use **Latin prepositions** to annotate ownership on parameters:

```
// No preposition = owned, will be moved/consumed (default)
functio consume(textus data) -> Result

// "de" (from/concerning) = borrowed, read-only
functio read(de textus source) -> numerus

// "in" (into) = mutable borrow, will be modified
functio append(in textus target, textus suffix)
```

This reads naturally as Latin:
- `de textus source` = "concerning the string source" (borrowed)
- `in textus target` = "into the string target" (mutable)

**Rust output:**
```rust
fn consume(data: String) -> Result          // owned
fn read(source: &str) -> i64                // borrowed
fn append(target: &mut String, suffix: String)  // mut borrow + owned
```

### Future Enhancement: Declensions

For those who want pure Latin, type declensions can work **interchangeably** with prepositions:

```
// Preposition style (works with any variable name)
functio read(de textus queryResult) -> numerus

// Declension style (pure Latin)
functio read(texti nomen) -> numerus

// Both generate the same Rust:
fn read(query_result: &str) -> i64
fn read(nomen: &str) -> i64
```

The compiler already has morphological analysis via the lexicon. It knows `texti` is genitive of `textus` - no type explosion needed, just lookup.

| Type (Nominative) | Genitive | Dative | Rust Mapping |
|-------------------|----------|--------|--------------|
| `textus` | `texti` | `texto` | `String` / `&str` / `&mut String` |
| `numerus` | `numeri` | `numero` | `i64` / `&i64` / `&mut i64` |
| `lista` | `listae` | `listae` | `Vec<T>` / `&[T]` / `&mut Vec<T>` |

### Lifetimes via `de` on Return Type

When a function returns a borrowed value, use `de` on the return type to tie its lifetime to the borrowed input(s):

```
functio first(de lista<textus> items) fit de textus
//            ^^ borrowed input          ^^ borrowed return, tied to input

functio longest(de textus x, de textus y) fit de textus
// All three share the same implicit lifetime
```

**Rust output:**
```rust
fn first(items: &[String]) -> &str
// Compiler infers: fn first<'a>(items: &'a [String]) -> &'a str

fn longest(x: &str, y: &str) -> &str
// Compiler infers: fn longest<'a>(x: &'a str, y: &'a str) -> &'a str
```

This mirrors Rust's lifetime elision rules - no explicit lifetime names needed for common patterns.

### Async/Generator Restriction: No Borrowed Returns

The return verb distinguishes sync from async:
- `fit` (present: "becomes") = sync return
- `fiet` (future: "will become") = async return

**Borrowed returns are only allowed with `fit` (sync).**

Async functions and generators cannot return borrowed values because the `Future`/iterator may outlive the borrowed data, creating dangling references. This is a fundamental Rust constraint.

```
// ALLOWED - sync can return borrowed
functio first(de lista<textus> items) fit de textus

// ERROR - async cannot return borrowed
futura functio first(de lista<textus> items) fiet de textus
```

**Error message:**
```
error: futura functio cannot return borrowed (de) value
  --> file.fab:10:50
   |
10 | futura functio first(de lista<textus> items) fiet de textus
   |                                                   ^^ borrowed return not allowed with fiet
   |
note: async futures may outlive borrowed data, causing dangling references
help: return owned value instead
   |
10 | futura functio first(de lista<textus> items) fiet textus
```

The same restriction applies to generators (`cursor` functions) yielding borrowed values.

### Design Principles

1. **No preposition = owned** (matches Rust's default move semantics)
2. **Prepositions are the accessible path** (work with any variable name)
3. **Declensions are the elegant path** (for Latin purists)
4. **`de` on return = borrowed, lifetime tied to inputs**
5. **Both syntaxes are equivalent** (same semantics, different aesthetics)

## Memory Management: Arena Allocator

Faber uses arena allocation as the default memory strategy for systems targets. This provides GC-like ergonomics without sacrificing Rust's safety guarantees.

### Why Arena?

1. **Simple mental model** - Allocate freely, everything freed at scope exit
2. **No lifetime complexity** - Arena owns everything, no borrowing headaches
3. **Zero memory leaks** - Arena drop handles cleanup
4. **Battle-tested** - Uses `bumpalo` crate (widely used in production)
5. **Unified approach** - Same strategy works for Zig (`std.heap.ArenaAllocator`)

### Generated Code Pattern

```rust
use bumpalo::Bump;

fn main() {
    // Arena wraps all allocations
    let arena = Bump::new();

    // User code - allocations use arena
    let greeting = greet(&arena, "World");
    println!("{}", greeting);

    // Arena dropped on function exit - no manual cleanup
}

fn greet<'a>(arena: &'a Bump, name: &str) -> &'a str {
    arena.alloc_str(&format!("Hello, {}!", name))
}
```

### Allocator Threading

Functions that may allocate receive `arena: &Bump` as a hidden first parameter. The codegen inserts this automatically for:
- Functions returning owned values (no `de` on return type)
- Functions that concatenate strings
- Functions that build collections

Functions with only borrowed parameters (`de`) and borrowed returns don't need arena.

### Fallback Approaches

If arena allocation is disabled or insufficient:

**Fallback 1: Clone Everything**

Generate owned values everywhere, clone on every pass. Safe but inefficient.

```rust
fn process(items: Vec<String>) -> Vec<String> {
    items.iter().map(|x| x.clone()).collect()
}
```

**Fallback 2: Inference Heuristics**

When no annotation is provided:
- Function params: borrow by default (`&T`)
- Return values: owned by default (`T`)
- Local variables: owned (`T`)
- Method receivers: `&self` for reads, `&mut self` for mutations

## Lambda Syntax

Faber uses `pro` for lambdas/closures, aligning with iteration syntax (`ex items pro x { }`).

### Syntax

```
pro <params> redde <expr>   // expression lambda
pro <params> { <body> }     // block lambda
```

### Examples

```
// Expression lambdas
fixum double = pro x redde x * 2
fixum add = pro x, y redde x + y

// Block lambdas
lista.filtra(pro user {
    si user.aetas < 18 { redde falsum }
    redde user.activus
})

// Zero-param
button.onClick(pro { scribe "clicked" })

// With higher-order functions
lista.mappa(pro item redde item.nomen)
lista.reducta(pro acc, x redde acc + x, 0)
lista.filtra(pro x redde x > 0)
```

Reads as: "for x, return x times 2" — same `pro` as iteration.

### Rust Output

Rust has native closures with `|params| body` syntax.

**Expression lambda:**
```
// Faber
pro x redde x * 2

// Rust
|x| x * 2
```

**Block lambda:**
```
// Faber
pro user {
    si user.aetas < 18 { redde falsum }
    redde user.activus
}

// Rust
|user| {
    if user.aetas < 18 { return false; }
    user.activus
}
```

**Capturing lambda:**
```
// Faber
fixum multiplier = 2
pro x redde x * multiplier

// Rust - captures by reference by default
let multiplier = 2;
|x| x * multiplier
```

### Capture Semantics

The compiler identifies captures (variables referenced but not in param list). For Rust:

- Default: capture by reference (`Fn`)
- With `in` preposition: capture by mutable reference (`FnMut`)
- Move semantics inferred when closure outlives scope (`FnOnce`)

```
// Faber - mutable capture
varia counter = 0
fixum increment = pro x {
    counter = counter + 1
    redde x
}

// Rust
let mut counter = 0;
|x| { counter += 1; x }  // FnMut
```

### Async Lambdas

Async is inferred from `cede` usage inside the closure:

```
// Faber
pro url {
    figendum data = fetchData(url)
    redde data
}

// Rust
|url| async move { fetch_data(url).await }
```

Async lambdas generate `async move` blocks to avoid lifetime issues.

## Error Handling Design

Faber uses a simplified error model that maps cleanly to Rust's `Result` type. This design is shared with the Zig target.

### Keywords

| Keyword | Meaning | Rust Output |
|---------|---------|-------------|
| `iace` | Expected failure, recoverable | `return Err(...)` |
| `mori` | Fatal, unrecoverable | `panic!("msg")` |
| `fac` | Block scope (like `{}`) | scoped block |
| `cape` | Catch errors on block | `match` / `if let Err` |

**Target-specific keywords:**
- `tempta`/`demum` - Available for TS/Python (maps to try/finally), but **not valid for Rust** because `demum` has no equivalent (Rust uses RAII/Drop). Use `fac`/`cape` instead.

### `fac` vs `tempta`

`fac` (do!) is a simple block scope. `tempta` (try) implies the full try/catch/finally pattern.

```
// Systems targets (Zig/Rust): use fac
fac {
    riskyCall()
} cape err {
    handleError(err)
}

// TS/Python: tempta with demum works
tempta {
    riskyCall()
} cape err {
    handleError(err)
} demum {
    cleanup()  // always runs
}
```

For Rust, `tempta` is rejected because it implies `demum` which cannot be directly implemented. Use `fac` for explicit block scoping with error handling.

### `iace` - Recoverable Errors

`iace` (throw) becomes an error return. The compiler automatically:
1. Marks functions containing `iace` as failable
2. Changes return type from `T` to `Result<T, FaberError>`
3. Inserts `?` at call sites of failable functions
4. Wraps successful returns in `Ok(...)`

```
// Faber
functio fetch(url: textus) -> textus {
    si timeout { iace "timeout" }
    redde data
}

fixum result = fetch(url)
```

**Rust output:**
```rust
fn fetch(arena: &Bump, url: &str) -> Result<String, FaberError> {
    if timeout { return Err(FaberError::new("timeout")); }
    Ok(data)
}

let result = fetch(&arena, url)?;
```

### `mori` - Fatal Errors

`mori` (to die) indicates an unrecoverable error - a bug or impossible state. Maps directly to `panic!`.

```
// Faber
si index < 0 { mori "negative index" }

// Rust
if index < 0 { panic!("negative index"); }
```

### `cape` - Error Boundaries

`cape` attaches to `fac` blocks or control flow statements to catch errors.

```
// Faber - fac block with cape
fac {
    riskyCall()
} cape err {
    handleError(err)
}

// cape on control flow
si needsData {
    fetchData()
} cape err {
    useFallback()
}
```

**Rust output:**
```rust
match risky_call() {
    Ok(_) => { /* success path */ },
    Err(err) => { handle_error(err); }
}

if needs_data {
    match fetch_data() {
        Ok(_) => { /* success */ },
        Err(err) => { use_fallback(); }
    }
}
```

### Error Type

By default, Faber uses a simple `FaberError` type wrapping a string message:

```rust
#[derive(Debug)]
pub struct FaberError {
    message: String,
}
```

Future enhancement: infer error enums from `iace` usage patterns.

## Type Mappings

| Faber | Rust | Notes |
|-------|------|-------|
| `textus` | `String` | Owned string |
| `numerus` | `i64` | Integer |
| `fractus` | `f64` | Floating point |
| `decimus` | `BigDecimal` | Arbitrary precision decimal |
| `bivalens` | `bool` | Boolean |
| `nihil` | `()` | Unit type |
| `textus?` | `Option<String>` | Optional |
| `lista<T>` | `Vec<T>` | Vector |
| `tabula<K,V>` | `HashMap<K,V>` | Hash map |
| `copia<T>` | `HashSet<T>` | Hash set |

## Current Implementation Status

**Not yet implemented.** The `rs/` codegen directory contains a skeleton only.

## Future Considerations

1. ~~**Ownership strategy**~~ - Solved: prepositions (`de`, `in`) with clone-all as fallback
2. ~~**Lifetime inference**~~ - Solved: `de` on return type mirrors Rust elision
3. ~~**Memory management**~~ - Solved: arena allocator (`bumpalo`) as default
4. ~~**Error handling**~~ - Solved: `iace`/`mori` split, `fac`/`cape` for blocks
5. ~~**Lambda syntax**~~ - Solved: `pro x redde expr` / `pro x { }` maps to native closures
6. **Async runtime** - Tokio vs async-std vs runtime-agnostic
7. **Derive macros** - Auto-generate `Default`, `Clone`, `Debug`
8. **Cargo integration** - Generate `Cargo.toml` for projects (include `bumpalo` dep)

## Open Questions

1. ~~Should Faber grow ownership annotations to support Rust properly?~~ **Yes - via Latin prepositions (`de`, `in`) with future declension support.**
2. ~~Is "clone everything" acceptable as fallback for unannotated code?~~ **Arena is primary, clone is fallback.**
3. Could we detect pure functions and optimize ownership automatically?
4. Should `genus` generate `#[derive(Clone, Default)]` automatically?
5. ~~What preposition maps to `Box<T>` (heap allocation)?~~ **Arena handles this - explicit Box not needed.**
6. ~~How do we handle lifetime annotations when borrowing across scopes?~~ **Use `de` on return type - mirrors Rust's elision rules.**
7. Should `de de textus` (double borrow) mean `&&str`? Probably not needed.
8. What about complex lifetime relationships (multiple distinct lifetimes)? Rare in practice - defer.
9. ~~Scope-based arenas: should `cum arena { }` create nested arenas for bounded lifetimes?~~ **Use `fac arena { }` - extends existing block syntax without overloading `cum`.**
