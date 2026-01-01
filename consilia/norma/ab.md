---
status: partial
targets: [ts, py, zig, cpp, rs]
note: Boolean property shorthand and transforms implemented for TS. Ubi conditions require explicit property access.
updated: 2024-12
---

# Collection Filtering DSL with `ab`

## Overview

The `ab` preposition ("away from, by") introduces a collection filtering DSL. All filtering operations use `ab` — the `ex` preposition remains unchanged for imports, iteration, destructuring, and borrow sources.

Filtering is fundamentally about separation: selecting items "away from" a collection based on criteria. Include vs exclude is just a `non` flag.

---

## Basic Syntax

### Iteration with Filter

```fab
# Keep matching
ab users ubi activus pro user {
    scribe(user.nomen)
}

# Exclude matching
ab users non ubi banned pro user {
    scribe(user.nomen)
}
```

### Expression Form

```fab
fixum active = ab users ubi activus
fixum clean = ab users non ubi banned
```

---

## Boolean Property Shorthand

When filtering on a boolean property, `ubi` is optional:

```fab
# Full form
fixum active = ab users ubi activus

# Shorthand
fixum active = ab users activus
fixum clean = ab users non banned
```

Reads as Latin:

- `ab users activus` → "from users, the active ones"
- `ab users non banned` → "from users, not the banned ones"

---

## Condition Expressions

For non-boolean conditions, use `ubi` with an expression:

```fab
fixum adults = ab users ubi aetas >= 18
fixum teens = ab users ubi aetas >= 13 et aetas < 18

# Complex conditions
fixum eligible = ab users ubi activus et verificatus
fixum excluded = ab users non ubi activus et verificatus
```

---

## Negation Scoping

The placement of `non` determines what gets negated:

### Single Condition — Equivalent

```fab
# These produce the same result
ab users non ubi banned pro user { }
ab users ubi non banned pro user { }
```

Both keep users where `banned` is false.

### Multiple Conditions — Different

```fab
# Outer non: negate entire expression (De Morgan applies)
ab users non ubi banned et suspended pro user { }
# Result: !(banned && suspended) → !banned || !suspended
# Keeps: users who are not banned, OR not suspended, OR neither

# Inner non: negate individual terms
ab users ubi non banned et non suspended pro user { }
# Result: !banned && !suspended
# Keeps: users who are neither banned nor suspended
```

### Summary

| Pattern              | Equivalent To      | Keeps                     |
| -------------------- | ------------------ | ------------------------- |
| `non ubi A`          | `ubi non A`        | where A is false          |
| `non ubi A et B`     | `ubi non (A et B)` | where NOT(A and B)        |
| `ubi non A et non B` | —                  | where A false AND B false |

Use outer `non` to negate compound conditions as a whole. Use inner `non` for individual term negation.

---

## Chaining Transforms

Transforms chain with commas:

```fab
fixum result = ab users activus, ordina per nomen, prima 10
fixum top5 = ab products ubi inStock, ordina per pretium descendens, prima 5
```

### Available Transforms

| Transform | Meaning  | Example               |
| --------- | -------- | --------------------- |
| `ordina`  | sort     | `ordina per nomen`    |
| `prima`   | first n  | `prima 10`            |
| `ultima`  | last n   | `ultima 5`            |
| `collige` | pluck    | `collige nomen`       |
| `grupa`   | group by | `grupa per categoria` |

Sort direction:

```fab
fixum sorted = ab items, ordina per pretium descendens
fixum sorted = ab items, ordina per nomen  # ascendens default
```

---

## Aggregation

Aggregation terminates the pipeline:

```fab
fixum total = ab orders ubi completum, summa pretium
fixum highest = ab scores, maximum
fixum count = ab users activus, numera
```

| Aggregation | Meaning | Returns   |
| ----------- | ------- | --------- |
| `summa`     | sum     | `numerus` |
| `maximum`   | max     | `T?`      |
| `minimum`   | min     | `T?`      |
| `medium`    | average | `fractus` |
| `numera`    | count   | `numerus` |

---

## Comparison with `ex`

The `ex` preposition is **not** affected by this DSL:

| `ex` (unchanged)                 | Purpose               |
| -------------------------------- | --------------------- |
| `ex norma importa scribe`        | Import                |
| `ex items pro item { }`          | Iteration (all items) |
| `ex response fixum status, data` | Destructuring         |
| `-> de textus ex param`          | Borrow source         |

| `ab` (new DSL)                        | Purpose            |
| ------------------------------------- | ------------------ |
| `ab items ubi condition pro item { }` | Filtered iteration |
| `ab items activus`                    | Filter expression  |
| `ab items, ordina per x, prima 5`     | Filter + transform |

---

## Target Mappings

### TypeScript

```fab
fixum active = ab users activus
```

```typescript
const active = users.filter(u => u.activus);
```

```fab
fixum clean = ab users non banned
```

```typescript
const clean = users.filter(u => !u.banned);
```

### Python

```fab
fixum active = ab users activus
```

```python
active = [u for u in users if u.activus]
```

### Zig

```fab
fixum active = ab users activus
```

```zig
var active = std.ArrayList(User).init(alloc);
for (users.items) |u| {
    if (u.activus) try active.append(u);
}
```

---

## Grammar

```ebnf
ab_expr     = "ab" source [filter] {"," transform} [aggregation]
filter      = ["non"] ("ubi" condition | identifier)
condition   = expression
transform   = "ordina" "per" property [direction]
            | "prima" number
            | "ultima" number
            | "collige" property
            | "grupa" "per" property
aggregation = "summa" [property] | "maximum" | "minimum" | "medium" | "numera"
direction   = "ascendens" | "descendens"
```

---

## Implementation Status

| Feature                                         | TS        | Py  | Zig | C++ | Rs  |
| ----------------------------------------------- | --------- | --- | --- | --- | --- |
| `ab source property` (boolean shorthand)        | Done      | -   | -   | -   | -   |
| `ab source non property`                        | Done      | -   | -   | -   | -   |
| Transform chaining (`prima`, `ultima`, `summa`) | Done      | -   | -   | -   | -   |
| `ab source ubi condition`                       | Partial\* | -   | -   | -   | -   |
| Aggregation (`ordina`, `grupa`, etc.)           | Not done  | -   | -   | -   | -   |
| Iteration form (`ab...pro`)                     | Not done  | -   | -   | -   | -   |

\*`ubi` conditions currently require explicit property access in the condition expression. Implicit property rewriting (where bare identifiers become `_x.property`) is not yet implemented.

---

## Design Rationale

### Why contain DSL in `ab`?

1. **Avoids overloading `ex`**: The `ex` preposition already has four distinct uses (import, iteration, destructuring, borrow source). Adding DSL filtering would make it six.

2. **DSL is fundamentally filtering**: Collection pipelines are almost always about filtering first, then transforming. The `ab` preposition ("away from") naturally implies selection/separation.

3. **Include/exclude is symmetric**: `ab users activus` vs `ab users non activus` — the `non` keyword handles negation cleanly.

4. **Clear mental model**: `ex` = draw all items; `ab` = draw filtered items.

### Why `non` instead of separate `ex`/`ab`?

Earlier design considered `ex` for include, `ab` for exclude. But:

- Requires learning two DSL entry points
- Complex conditions mix include/exclude logic anyway
- `non` is already a Faber keyword
- Simpler grammar with one entry point
