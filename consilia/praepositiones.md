---
status: partial
targets: [ts, py, zig, cpp, rs]
note: Core prepositions implemented; collection DSL and some ownership features pending
updated: 2024-12
---

# Praepositiones - Latin Prepositions

## Overview

Faber uses Latin prepositions as syntactic markers throughout the language. Unlike English keywords that have fixed meanings, Latin prepositions derive their specific role from **position** — just as Latin uses noun declensions to indicate grammatical role regardless of word order.

This is not overloading. It's positional grammar.

---

## Banned: `cum`

The Latin preposition `cum` ("with") is **permanently banned** from Faber. Its English homograph makes it unsuitable for a programming language. Use alternative constructs for "with" semantics — `cura` for resource management, composition for companion values.

---

## The Six Prepositions

| Preposition | Latin Meaning       | Core Semantic         |
| ----------- | ------------------- | --------------------- |
| `ex`        | "from, out of"      | Source/origin         |
| `de`        | "from, concerning"  | Read-only reference   |
| `in`        | "in, into"          | Mutable target        |
| `ad`        | "to, toward"        | Destination/recipient |
| `pro`       | "for, on behalf of" | Binding/naming        |
| `qua`       | "as, by which"      | Type/name binding     |

---

## `ex` — Source/Origin

**Latin:** "from, out of"

`ex` introduces a source from which something is drawn. What happens next depends on context.

### Import

```fab
ex norma importa scribe, lege
ex "hono" importa Hono
```

Draw bindings **from** a module.

### Iteration

```fab
ex items pro item {
    scribe(item)
}
```

Draw each element **from** the collection.

### Iteration with Transforms

```fab
ex items filtra ubi active pro item {
    scribe(item)
}

ex items filtra ubi active, ordina per nomen pro item {
    scribe(item)
}
```

Draw elements **from** the collection, after applying transforms. The `ex ... pro` frame stays constant; the middle is an optional pipeline.

### Collection Expression

```fab
fixum active = ex users filtra ubi active
fixum sorted = ex users filtra ubi active, ordina per nomen
fixum total = ex prices summa
```

Same as iteration transforms, but assigned instead of iterated. Context (assignment vs `pro` block) determines behavior.

### Destructuring

```fab
ex response fixum { status, data }
ex fetchData() figendum { result }   // await + destructure
```

Extract fields **from** an object/expression.

### Summary

| Pattern                            | Meaning                         |
| ---------------------------------- | ------------------------------- |
| `ex module importa ...`            | Import from module              |
| `ex source pro var { }`            | Iterate from source             |
| `ex source transforms pro var { }` | Iterate from transformed source |
| `ex source transforms` (assigned)  | Collection expression           |
| `ex source fixum { pattern }`      | Destructure from source         |

---

## `de` — Read-Only Reference

**Latin:** "from, concerning"

`de` indicates a read-only relationship. You're reading from or concerning something, not modifying it.

### Key Iteration

```fab
de tabula pro clavis {
    scribe(clavis, tabula[clavis])
}
```

Iterate over keys **concerning** an object. Contrast with `ex`, which iterates values.

### Borrowed Parameter (Systems Targets)

```fab
functio process(de textus input) {
    // input is borrowed, read-only
    scribe(input)
}
```

The parameter is borrowed **from** the caller. Read-only access.

**Target mappings:**

- Zig: `input: []const u8`
- Rust: `input: &str`
- C++: `const std::string&`
- TS/Py: ignored (GC handles memory)

### Novum Source

```fab
fixum props = { nomen: "Marcus", aetas: 30 }
fixum person = novum Persona de props
```

Create new instance, taking initial values **from** an expression.

### Summary

| Pattern                 | Meaning                      |
| ----------------------- | ---------------------------- |
| `de object pro key { }` | Iterate keys from object     |
| `de Type param`         | Borrowed/read-only parameter |
| `novum Type de expr`    | Initialize from expression   |

---

## `in` — Mutable Target

**Latin:** "in, into"

`in` indicates reaching into something to modify it.

### Mutation Block

```fab
in user {
    nomen = "Marcus"
    aetas = 31
}
```

Reach **into** the object to modify its fields.

### Mutable Parameter (Systems Targets)

```fab
functio append(in lista<textus> items, textus value) {
    items.adde(value)
}
```

The parameter is mutably borrowed. The function modifies what was passed **in**.

**Target mappings:**

- Zig: `items: *std.ArrayList([]const u8)`
- Rust: `items: &mut Vec<String>`
- C++: `std::vector<std::string>&`
- TS/Py: ignored (reference semantics by default)

### Summary

| Pattern                   | Meaning                    |
| ------------------------- | -------------------------- |
| `in object { mutations }` | Mutate object's fields     |
| `in Type param`           | Mutably borrowed parameter |

---

## `ad` — Destination/Recipient

**Latin:** "to, toward"

`ad` indicates a destination or recipient.

### Ad Dispatch (HTTP/Messaging)

```fab
ad "https://api.example.com/users" ("GET") fiet Response qua resp {
    scribe(resp.status)
}
```

Send a request **to** a destination. See `ad.md` for full documentation.

### Future Uses

`ad` may be used for:

- Channel sends: `ad channel mitte(message)`
- Event dispatch: `ad handler mitte(event)`

### Summary

| Pattern                                 | Meaning                      |
| --------------------------------------- | ---------------------------- |
| `ad url (method) fiet Type qua var { }` | HTTP/dispatch to destination |

---

## `pro` — Binding/Naming

**Latin:** "for, on behalf of"

`pro` introduces named bindings. It says "for this name, bind the following."

### Iteration Binding

```fab
ex items pro item {
    scribe(item)
}
```

**For** each element, call it `item`.

### Lambda Parameter

```fab
fixum double = pro x: x * 2
fixum add = pro a, b: a + b
```

**For** parameter `x`, compute the body.

### Variant Field Binding

```fab
discerne event {
    si Click pro x, y { scribe(x, y) }
}
```

**For** fields `x` and `y`, bind them from the matched variant. Note: variant matching uses `discerne`/`si`, not `ex`. See below.

### Internal Parameter Name

```fab
functio move(de Point[] from pro source, in Point[] to pro dest) {
    // 'source' and 'dest' are internal names
    // 'from' and 'to' are external (callsite) names
}
```

**For** external name `from`, use internal name `source`.

### Summary

| Pattern                      | Meaning                  |
| ---------------------------- | ------------------------ |
| `ex source pro var { }`      | Bind each element as var |
| `pro params: expr`           | Lambda with params       |
| `si Variant pro fields { }`  | Bind variant fields      |
| `Type external pro internal` | Internal parameter name  |

---

## `qua` — Type/Name Binding

**Latin:** "as, in the capacity of, by which"

`qua` binds a value as a specific name or type. It says "treat this as."

### Response Binding

```fab
ad "https://api.example.com/users" ("GET") fiet Response qua resp {
    scribe(resp.status)
}
```

Bind the response **as** `resp`.

### Type Assertion

```fab
fixum value = getData() qua textus
```

Treat the result **as** `textus`. (Type assertion, not conversion.)

### Summary

| Pattern              | Meaning                    |
| -------------------- | -------------------------- |
| `fiet Type qua name` | Bind typed result as name  |
| `expr qua Type`      | Assert expression as type  |

---

## Positional Grammar

The same preposition means different things based on position:

```fab
// 'ex' at statement start = iteration/import
ex items pro item { }

// 'pro' after 'ex' = iteration binding
ex items pro item { }

// 'pro' at expression start = lambda
pro x: x + 1

// 'pro' after param type = internal name
functio f(textus name pro n) { }

// 'pro' after 'si' in discerne = variant field binding
discerne event { si Click pro x, y { } }
```

This mirrors Latin, where word order is flexible because declensions carry grammatical role. Faber uses position instead of declensions, but the principle is the same: **context determines role**.

---

## Preposition Combinations

Prepositions compose naturally:

```fab
// Iterate from source, binding as name
ex items pro item { }

// Borrow from caller, name internally
functio f(de textus external pro internal) { }

// Extract from source into bindings
ex response fixum { data }

// Send to destination, bind response
ad url ("GET") fiet Response qua resp { }
```

---

## Implementation Status

| Preposition | Context                      | Status     |
| ----------- | ---------------------------- | ---------- |
| `ex`        | Import                       | Done       |
| `ex`        | Iteration (`ex...pro`)       | Done       |
| `ex`        | Destructuring                | Done       |
| `ex`        | Collection DSL               | Not done   |
| `de`        | Key iteration (`de...pro`)   | Done       |
| `de`        | Borrowed parameter           | Done (Zig) |
| `de`        | Novum source                 | Done       |
| `in`        | Mutation block               | Not done   |
| `in`        | Mutable parameter            | Done (Zig) |
| `ad`        | HTTP dispatch                | Partial    |
| `pro`       | Iteration binding            | Done       |
| `pro`       | Lambda parameter             | Done       |
| `pro`       | Variant binding (`discerne`) | Not done   |
| `pro`       | Internal param name          | Not done   |
| `qua`       | Response binding (`ad`)      | Partial    |
| `qua`       | Type assertion               | Done       |

---

## Target Behavior

### GC Targets (TypeScript, Python)

Ownership prepositions (`de`, `in` on parameters) are ignored. The language has reference semantics by default.

### Systems Targets (Zig, Rust, C++)

Ownership prepositions map to borrow semantics:

| Faber  | Zig                     | Rust     | C++        |
| ------ | ----------------------- | -------- | ---------- |
| (none) | owned/copied            | owned    | value      |
| `de`   | `[]const u8`, const ptr | `&T`     | `const T&` |
| `in`   | `*T` (mutable ptr)      | `&mut T` | `T&`       |

See `codegen/zig.md`, `codegen/rust.md`, `codegen/cpp.md` for details.
