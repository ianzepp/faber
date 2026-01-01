# Functiones

Design decisions for function declarations, parameters, and call syntax. For ownership prepositions (`de`, `in`), see `praepositiones.md`.

## Status

| Feature                | Status | Notes                               |
| ---------------------- | ------ | ----------------------------------- |
| Basic functions        | Done   | `functio name() { }`                |
| Typed parameters       | Done   | `functio f(textus name) { }`        |
| Ownership prepositions | Done   | `de`/`in` for borrow semantics      |
| Dual parameter naming  | Done   | Swift-style external/internal names |
| Default values (`vel`) | Done   | `textus name vel "default"`         |

## Dual Parameter Naming

Swift allows separate external (callsite) and internal (body) names for parameters:

```swift
func greet(at location: String) { print(location) }
greet(at: "Rome")  // callsite uses "at"
```

Faber adopts this with `ut` (as) for aliasing the internal name:

```fab
functio greet(textus location ut loc) {
    scribe loc  # internal name
}

greet(location: "Roma")  # external name at callsite
```

### Full Syntax

```ebnf
parameter := preposition? typeAnnotation externalName ('ut' internalName)?
preposition := 'ad' | 'de' | 'in' | 'ex'
```

See `praepositiones.md` for the complete preposition system.

### With Ownership Prepositions

Ownership prepositions combine naturally:

```fab
functio processPoints(de Point[] points ut p1, in Point[] targets ut p2) {
    # p1 is borrowed (read-only)
    # p2 is mutably borrowed
    ex p1 pro point {
        p2.adde(point)
    }
}
```

### Why `ut`?

Latin `ut` means "as" — the external name is known internally _as_ the internal name. This unifies aliasing across the language:

- **Imports:** `ex norma importa scribe ut s`
- **Destructuring:** `ex persona fixum nomen ut n`
- **Parameters:** `textus location ut loc`

All three express the same concept: "X, known locally as Y."

> **Historical note:** Earlier designs used `pro` for parameter aliasing because `ut` was reserved for type casting. When `qua` took over casting (`x qua numerus`), `ut` became available for aliasing, enabling this unified syntax.

### Callsite Syntax

Named arguments at callsites use the external name:

```fab
functio move(de Point[] from ut source, in Point[] to ut dest) {
    # ...
}

move(from: points, to: targets)
```

### When Internal Name is Omitted

If `ut internalName` is omitted, the external name serves both roles (current behavior):

```fab
functio greet(textus name) {
    scribe name  # same name internally
}
```

### Target Codegen

| Target     | Mapping                                       |
| ---------- | --------------------------------------------- |
| TypeScript | External name in JSDoc, internal in body      |
| Python     | Both names in signature (Python 3.8+ posargs) |
| Zig        | Internal name only (no named args)            |
| Rust       | Internal name only (no named args)            |
| C++        | Internal name only (no named args)            |

For targets without named arguments, only the internal name appears in generated code. The external name is documentation-only.

## Default Values

Default values use `vel` (Latin "or"), consistent with the nullish coalescing operator:

```fab
functio greet(textus name vel "Mundus") {
    scribe "Salve, " + name
}

greet()          # "Salve, Mundus"
greet("Marcus")  # "Salve, Marcus"
```

### Why `vel`?

1. **Consistency** — `vel` already means "or if nil" in expressions: `value vel "default"`
2. **Readability** — "textus name or Mundus" reads naturally
3. **Avoids `:` confusion** — Colon looks like type annotation in other languages

### With Dual Naming

Default comes after the internal name binding:

```fab
functio greet(textus location ut loc vel "Roma") {
    scribe loc
}
```

### Full Syntax

```ebnf
parameter := preposition? typeAnnotation externalName ('ut' internalName)? ('vel' defaultValue)?
```

### Ownership Restriction

Default values only make sense for **owned** parameters. Borrowed (`de`) and mutable (`in`) parameters require the caller to provide a value:

```fab
# Valid - owned with default
functio greet(textus name vel "Mundus")

# Invalid - borrowed can't have default (no owned storage for default)
functio greet(de textus displayName ut name vel "Mundus")  # error

# Invalid - mutable borrow can't have default (nothing to mutate)
functio process(in numerus[] items vel [])  # error
```

**Rationale:** A borrowed parameter means "the caller passes a reference to something they own." A default would require the function to own that default value, contradicting the borrow semantics.

## Open Questions

1. Named argument syntax at callsite: `name: value` or `name = value`?
