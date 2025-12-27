---
status: implemented
targets: [ts, py, zig, cpp]
updated: 2024-12
---

# Union Types

Faber has two union constructs with distinct semantics.

## Implementation Status

### Implemented

- `unio<A, B>` generic type — TypeScript, Python, Zig (falls back to `anytype`)
- `discretio` declaration — TypeScript, Python, Zig, C++
- Variant syntax with named fields — All targets
- Generic type parameters (`discretio Option<T>`) — All targets
- Pattern matching (`discerne`/`si Variant pro bindings`) — TypeScript, Python, Zig
- Semantic analysis: type registration and binding introduction

### Partial / In Progress

- C++ pattern matching — generates TODO placeholder (needs `std::visit`)
- Exhaustiveness checking — not yet enforced by semantic analyzer
- Variant construction validation — `Type.Variant { ... }` not yet validated

### Not Yet Implemented

- Rust codegen
- Tuple-style variant shorthand (`Some(T)`)
- Methods on discretio types
- Nested pattern matching
- Guard clauses in patterns

---

## unio — Simple Type Union

A value that could be one of several types, without a discriminator tag. Requires runtime type checking to determine which.

**Etymology:** `unio` — "union, oneness"

### Syntax

```
typus Id = unio<numerus, textus>
typus Json = unio<textus, numerus, bivalens, nihil, Json[], tabula<textus, Json>>
```

### Usage

```
fixum id: unio<numerus, textus> = 42

// Must check type at runtime
si id est numerus {
    scribe "numeric id: " + id
}
si id est textus {
    scribe "string id: " + id
}
```

### Target Mappings

| Faber        | TypeScript | Python   | Zig | Rust |
| ------------ | ---------- | -------- | --- | ---- |
| `unio<A, B>` | `A \| B`   | `A \| B` | —   | —    |

**Note:** Zig and Rust don't have untagged unions in this sense. For systems targets, prefer `discretio` which maps to native tagged unions.

---

## discretio — Tagged Union

A discriminated union where each variant has a tag and optional payload. The compiler tracks which variant is active, enabling exhaustive pattern matching.

**Etymology:** `discretio` — "distinction, discrimination" — the tag discriminates between variants.

### Syntax

```
discretio Event {
    Click { numerus x, numerus y }
    Keypress { textus key }
    Quit
}

discretio Option<T> {
    Some { T value }
    None
}

discretio Result<T, E> {
    Ok { T value }
    Err { E error }
}
```

### Variant Forms

Variants can have:

1. **No payload** — just the tag

    ```
    Quit
    None
    ```

2. **Named fields** — struct-like payload (type-first, like `genus` fields)

    ```
    Click { numerus x, numerus y }
    ```

3. **Single value** — tuple-like payload (shorthand)
    ```
    Some { T value }
    // or potentially: Some(T) — not yet decided
    ```

### Construction

Construction uses the type name, dot, variant name, then an object literal for fields:

```
fixum event = Event.Click { x: 10, y: 20 }
fixum key = Event.Keypress { key: "Enter" }
fixum quit = Event.Quit

fixum result: Result<numerus, textus> = Result.Ok { value: 42 }
```

Note: The object literal uses name-colon-value syntax (like all Faber object literals), not type-first. The type-first syntax is only for the _declaration_ of variant fields.

### Pattern Matching

Use `discerne` for exhaustive matching with `si`/`pro` syntax:

```fab
discerne event {
    si Click pro x, y { scribe "clicked at " + x + ", " + y }
    si Keypress pro key { scribe "pressed " + key }
    si Quit { mori "goodbye" }
}
```

**Etymology:** `discerne` (imperative of `discerno`) means "distinguish!" — paired with `discretio` (distinction), the type it matches.

The syntax is `si VariantName pro bindings { body }`:

- `si` = "if this variant" (conditional match)
- `pro` = "binding these names"
- For unit variants (no payload), omit `pro`: `si Quit { ... }`

The compiler enforces exhaustiveness — all variants must be handled.

#### With Binding

```fab
discerne result {
    si Ok pro value { redde value }
    si Err pro error { iace error }
}
```

Only the fields you name are bound. Binding order matches declaration order.

#### Value Matching with `elige`

For simple value matching (not variant extraction), use `elige`:

```fab
elige status {
    si "pending" { ... }      // value match (string)
    si "active" { ... }
    aliter { ... }
}
```

`elige` = "choose" — for selecting among values.
`discerne` = "distinguish" — for matching `discretio` variants.

### Target Mappings

| Faber                     | TypeScript          | Python                | Zig           | Rust                 |
| ------------------------- | ------------------- | --------------------- | ------------- | -------------------- |
| `discretio Event { ... }` | Discriminated union | `@dataclass` variants | `union(enum)` | `enum Event { ... }` |

#### TypeScript Output

```fab
// Faber source:
discerne event {
    si Click pro x, y { scribe "clicked at " + x + ", " + y }
    si Keypress pro key { scribe "pressed " + key }
    si Quit { mori "goodbye" }
}
```

```typescript
// Generated TypeScript:
type Event = { tag: 'Click'; x: number; y: number } | { tag: 'Keypress'; key: string } | { tag: 'Quit' };

const event: Event = { tag: 'Click', x: 10, y: 20 };

switch (event.tag) {
    case 'Click': {
        const { x, y } = event;
        console.log(`clicked at ${x}, ${y}`);
        break;
    }
    case 'Keypress': {
        const { key } = event;
        console.log(`pressed ${key}`);
        break;
    }
    case 'Quit':
        throw new Error('goodbye');
}
```

#### Zig Output

```fab
// Faber source:
discerne event {
    si Click pro x, y { scribe "clicked at " + x + ", " + y }
    si Keypress pro key { scribe "pressed " + key }
    si Quit { mori "goodbye" }
}
```

```zig
// Generated Zig:
const Event = union(enum) {
    click: struct { x: i64, y: i64 },
    keypress: struct { key: []const u8 },
    quit,
};

const event = Event{ .click = .{ .x = 10, .y = 20 } };

switch (event) {
    .click => |c| {
        const x = c.x;
        const y = c.y;
        std.debug.print("clicked at {}, {}\n", .{ x, y });
    },
    .keypress => |k| {
        const key = k.key;
        std.debug.print("pressed {s}\n", .{key});
    },
    .quit => @panic("goodbye"),
}
```

#### Rust Output

```fab
// Faber source:
discerne event {
    si Click pro x, y { scribe "clicked at " + x + ", " + y }
    si Keypress pro key { scribe "pressed " + key }
    si Quit { mori "goodbye" }
}
```

```rust
// Generated Rust:
enum Event {
    Click { x: i64, y: i64 },
    Keypress { key: String },
    Quit,
}

let event = Event::Click { x: 10, y: 20 };

match event {
    Event::Click { x, y } => println!("clicked at {}, {}", x, y),
    Event::Keypress { key } => println!("pressed {}", key),
    Event::Quit => panic!("goodbye"),
}
```

---

## Comparison

| Aspect          | `unio<A, B>`      | `discretio Name { ... }`     |
| --------------- | ----------------- | ---------------------------- |
| Tag             | None (untagged)   | Compiler-managed             |
| Type check      | Runtime (`est`)   | Compile-time (pattern match) |
| Exhaustive      | No                | Yes                          |
| Systems targets | Not supported     | Native support               |
| Use case        | Flexible "one of" | Structured variants          |

### When to Use Which

**Use `unio<A, B>` when:**

- Targeting TypeScript/Python only
- Types are simple primitives
- Runtime type checking is acceptable

**Use `discretio` when:**

- Targeting Zig/Rust (required)
- Variants have associated data
- Exhaustive matching is desired
- Building compiler/interpreter data structures (AST nodes, tokens, etc.)

---

## Open Questions

1. **Tuple-style variants:** Should `Some(T)` be allowed as shorthand for `Some { T value }`?
2. **Methods on discretio:** Can variants have associated methods?
3. **Nested patterns:** How deep can pattern matching go?
4. **Guard clauses:** Should `si Click pro x, y ubi x > 0 { ... }` be supported?

---

## Implementation Notes

### Grammar

```ebnf
discretioDecl := 'discretio' IDENTIFIER typeParams? '{' variant (',' variant)* ','? '}'
variant := IDENTIFIER ('{' variantFields '}')?
variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
```

### Pattern Matching Grammar Extension

```ebnf
// elige for value matching
eligeStmt := 'elige' expression '{' eligeCase* defaultCase? '}'
eligeCase := 'si' expression (blockStmt | 'ergo' expression)

// discerne for variant matching (discretio types)
discerneStmt := 'discerne' expression '{' discerneCase* '}'
discerneCase := 'si' IDENTIFIER ('pro' IDENTIFIER (',' IDENTIFIER)*)? blockStmt
```

### AST Nodes (Implemented)

- `DiscretioDeclaration` — the type declaration (`fons/parser/ast.ts`)
- `VariantDeclaration` — each variant within discretio
- `VariantField` — type-first field declaration within a variant
- `VariantCase` — `si Variant pro bindings { }` in discerne

### Compiler Pipeline

1. **Lexicon** (`fons/lexicon/keywords.ts`): `discretio` keyword
2. **Parser** (`fons/parser/index.ts`):
    - `parseDiscretioDeclaration()` — parses the full declaration
    - `parseVariantDeclaration()` — parses each variant
    - `parseDiscerneStatement()` — parses variant matching with `si`/`pro` syntax
3. **Semantic** (`fons/semantic/index.ts`):
    - `analyzeDiscretioDeclaration()` — registers type in scope
    - `analyzeDiscerneStatement()` — introduces variant bindings in each case scope
4. **Codegen**:
    - TypeScript: Discriminated union with `tag` property
    - Python: `@dataclass` per variant + union type alias
    - Zig: Native `union(enum)` with struct payloads
    - C++: `std::variant<...>` with structs per variant
