---
status: completed
updated: 2026-01-06
note: Overview of preposition system. See individual files for detailed documentation.
---

# Praepositiones - Latin Prepositions

## Overview

Faber uses Latin prepositions as syntactic markers throughout the language. Unlike English keywords that have fixed meanings, Latin prepositions derive their specific role from **position** — just as Latin uses noun declensions to indicate grammatical role regardless of word order.

This is not overloading. It's positional grammar.

---

## Banned: `cum`

The Latin preposition `cum` ("with") is **permanently banned** from Faber. Its English homograph makes it unsuitable for a programming language. Use alternative constructs for "with" semantics — `cura` for resource management, composition for companion values.

---

## The Seven Prepositions

| Preposition | Latin Meaning        | Core Semantic         | Documentation      |
| ----------- | -------------------- | --------------------- | ------------------ |
| `ex`        | "from, out of"       | Source/origin         | `ex.md`            |
| `de`        | "from, concerning"   | Read-only reference   | `de.md`            |
| `in`        | "in, into"           | Mutable target        | `in.md`            |
| `ad`        | "to, toward"         | Destination/recipient | `futura/ad.md`     |
| `pro`       | "for, on behalf of"  | Iteration binding     | `pro.md`           |
| `ut`        | "as, like"           | Aliasing/renaming     | `ut.md`            |
| `qua`       | "as, in capacity of" | Type assertion        | `qua.md`           |

---

## Positional Grammar

The same preposition means different things based on position:

```fab
# 'ex' at statement start = iteration/import
ex items pro item { }

# 'pro' after 'ex' = iteration binding
ex items pro item { }

# 'pro' at expression start = lambda
pro x: x + 1

# 'pro' after 'casu' in discerne = variant field binding
discerne event { casu Click pro x, y { } }
```

This mirrors Latin, where word order is flexible because declensions carry grammatical role. Faber uses position instead of declensions, but the principle is the same: **context determines role**.

---

## Preposition Combinations

Prepositions compose naturally:

```fab
# Iterate from source, binding as name
ex items pro item { }

# Borrow from caller, rename internally
functio f(de textus external ut internal) { }

# Import and rename
ex norma importa scribe ut s { }

# Extract from source into bindings (brace-less)
ex response fixum status, data

# Send to destination, bind response (future)
ad url ("GET") fiet Response pro resp { }
```

---

## Implementation Status Summary

| Preposition | Primary Use Cases               | Status    |
| ----------- | ------------------------------- | --------- |
| `ex`        | Import, iteration, destructure  | ✅ Mostly |
| `de`        | Keys, borrowed params, novum    | ✅ Mostly |
| `in`        | Mutation, mutable params        | ✅ Done   |
| `ad`        | Syscall dispatch                | ❌ Future |
| `pro`       | Binding in loops/lambdas        | ✅ Done   |
| `ut`        | Import/destructure aliases      | ✅ Mostly |
| `qua`       | Type casting                    | ✅ Done   |

See individual files for detailed implementation status and examples.

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

See target-specific codegen docs for details.
