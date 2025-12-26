# Union Types

Faber has two union constructs with distinct semantics.

## Implementation Status

### Implemented

- `unio<A, B>` generic type — TypeScript, Python, Zig (falls back to `anytype`)

### Not Yet Implemented

- `discretio` declaration and variant syntax
- Pattern matching on `discretio` variants
- Codegen for Rust, C++

---

## unio — Simple Type Union

A value that could be one of several types, without a discriminator tag. Requires runtime type checking to determine which.

**Etymology:** `unio` — "union, oneness"

### Syntax

```
typus Id = unio<numerus, textus>
typus Json = unio<textus, numerus, bivalens, nihil, lista<Json>, tabula<textus, Json>>
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
    Click { x: numerus, y: numerus }
    Keypress { key: textus }
    Quit
}

discretio Option<T> {
    Some { value: T }
    None
}

discretio Result<T, E> {
    Ok { value: T }
    Err { error: E }
}
```

### Variant Forms

Variants can have:

1. **No payload** — just the tag

    ```
    Quit
    None
    ```

2. **Named fields** — struct-like payload

    ```
    Click { x: numerus, y: numerus }
    ```

3. **Single value** — tuple-like payload (shorthand)
    ```
    Some { value: T }
    // or potentially: Some(T)
    ```

### Construction

```
fixum event = Event.Click { x: 10, y: 20 }
fixum key = Event.Keypress { key: "Enter" }
fixum quit = Event.Quit

fixum result: Result<numerus, textus> = Result.Ok { value: 42 }
```

### Pattern Matching

Use `elige` for exhaustive matching:

```
elige event {
    si Click { scribe "clicked at " + x + ", " + y }
    si Keypress { scribe "pressed " + key }
    si Quit { mori "goodbye" }
}
```

The compiler enforces exhaustiveness — all variants must be handled.

#### With Binding

```
elige result {
    si Ok { redde value }
    si Err { iace error }
}
```

Variant fields are bound as local variables within the branch.

### Target Mappings

| Faber                     | TypeScript          | Python                | Zig           | Rust                 |
| ------------------------- | ------------------- | --------------------- | ------------- | -------------------- |
| `discretio Event { ... }` | Discriminated union | `@dataclass` variants | `union(enum)` | `enum Event { ... }` |

#### TypeScript Output

```typescript
type Event = { tag: 'Click'; x: number; y: number } | { tag: 'Keypress'; key: string } | { tag: 'Quit' };

const event: Event = { tag: 'Click', x: 10, y: 20 };

switch (event.tag) {
    case 'Click':
        console.log(`clicked at ${event.x}, ${event.y}`);
        break;
    case 'Keypress':
        console.log(`pressed ${event.key}`);
        break;
    case 'Quit':
        throw new Error('goodbye');
}
```

#### Zig Output

```zig
const Event = union(enum) {
    click: struct { x: i64, y: i64 },
    keypress: struct { key: []const u8 },
    quit,
};

const event = Event{ .click = .{ .x = 10, .y = 20 } };

switch (event) {
    .click => |c| std.debug.print("clicked at {}, {}\n", .{c.x, c.y}),
    .keypress => |k| std.debug.print("pressed {s}\n", .{k.key}),
    .quit => @panic("goodbye"),
}
```

#### Rust Output

```rust
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

1. **Tuple-style variants:** Should `Some(T)` be allowed as shorthand for `Some { value: T }`?
2. **Methods on discretio:** Can variants have associated methods?
3. **Nested patterns:** How deep can pattern matching go?
4. **Guard clauses:** Should `si Click si x > 0 { ... }` be supported?
