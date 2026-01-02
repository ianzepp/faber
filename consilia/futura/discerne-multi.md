# Multi-Discriminant Pattern Matching

## Summary

Extend `discerne` to match multiple values simultaneously, reducing nesting when comparing two or more typed values.

## Motivation

Comparing two types (or any two discriminated unions) currently requires nested `discerne` blocks:

```fab
functio typiAequales(Typus a, Typus b) -> bivalens {
    discerne a {
        casu Primitivum ut pa {
            discerne b {
                casu Primitivum ut pb {
                    redde pa.nomen == pb.nomen
                }
            }
            redde falsum
        }

        casu Genericum ut ga {
            discerne b {
                casu Genericum ut gb {
                    si ga.nomen != gb.nomen { redde falsum }
                    # ... more checks ...
                    redde verum
                }
            }
            redde falsum
        }

        # ... 6 more variants, each with nested discerne ...
    }
}
```

Problems:

- Deep nesting (4+ levels)
- Repeated `redde falsum` after each inner `discerne`
- Hard to scan — the "both are Primitivum" case is buried two levels deep

## Proposed Syntax

Comma-separated discriminants and patterns:

```fab
functio typiAequales(Typus a, Typus b) -> bivalens {
    discerne a, b {
        casu Primitivum ut pa, Primitivum ut pb {
            redde pa.nomen == pb.nomen
        }

        casu Genericum ut ga, Genericum ut gb {
            si ga.nomen != gb.nomen ergo redde falsum
            si ga.parametri.longitudo() != gb.parametri.longitudo() ergo redde falsum

            varia i = 0
            ex ga.parametri pro t {
                si non typiAequales(t, gb.parametri[i]) ergo redde falsum
                i = i + 1
            }
            redde verum
        }

        casu Functio ut fa, Functio ut fb {
            # ... checks ...
            redde verum
        }

        # ... more same-type pairs ...

        casu _, _ {
            redde falsum
        }
    }
}
```

## Grammar

```ebnf
discerneStatement := 'discerne' discriminants '{' discerneCase* '}'
discriminants := expression (',' expression)*
discerneCase := 'casu' patterns '{' statement* '}'
patterns := pattern (',' pattern)*
pattern := '_' | (TYPENAME patternBind?)
patternBind := ('ut' IDENTIFIER) | ('pro' IDENTIFIER (',' IDENTIFIER)*)
```

Note: Binding is per-pattern: `casu X ut a, Y pro y`.

## Semantics

### `ut` vs `pro`

- `casu X ut a` binds `a` to the entire variant value (payload accessed via `a.field`).
- `casu X pro x` extracts and binds payload fields from inside the variant.
- `casu X pro x, y` extracts multiple payload fields (positional).

1. **Arity must match:** `discerne a, b` requires each `casu` to have exactly two patterns
2. **Per-pattern binding:** Each pattern can bind with `ut` or extract with `pro`
3. **Wildcard `_`:** Matches any variant, binds nothing
4. **No default arm:** catch-all combinations are expressed with `_` patterns

## Examples

### Matching same types

```fab
discerne left, right {
    casu Primitivum ut l, Primitivum ut r {
        redde l.nomen == r.nomen
    }
    casu _, _ {
        redde falsum
    }
}
```

### Mixed matching with wildcards

```fab
discerne left, right {
    casu Ignotum, _ { redde verum }       # left unknown, any right
    casu _, Ignotum { redde verum }       # any left, right unknown
    casu Primitivum ut l, Primitivum ut r { ... }
    casu _, _ { redde falsum }
}
```

### Three-way matching

```fab
discerne a, b, c {
    casu X ut x, Y ut y, Z ut z { ... }
    casu _, _, _ { ... }
}
```

## Benefits

| Metric                     | Before          | After              |
| -------------------------- | --------------- | ------------------ |
| Nesting levels             | 4+              | 2                  |
| `redde falsum` repetitions | 9 (per variant) | 1 (in `casu _, _`) |
| Lines (typiAequales)       | ~120            | ~80                |

## LLM Readability

Multi-discriminant matching is highly scannable:

- Each `casu` line declares exactly what combination it handles
- No hunting through nested blocks to find the "both are X" case
- `casu _, _` clearly handles "everything else"

Combined with `custodi` blocks for preconditions:

```fab
casu Genericum ut ga, Genericum ut gb {
    custodi {
        si ga.nomen != gb.nomen ergo redde falsum
        si ga.parametri.longitudo() != gb.parametri.longitudo() ergo redde falsum
    }

    # Past custodi: same name, same arity guaranteed
    varia i = 0
    ex ga.parametri pro t {
        si non typiAequales(t, gb.parametri[i]) ergo redde falsum
        i = i + 1
    }
    redde verum
}
```

The `custodi` block signals "preconditions checked here" — once past it, the LLM knows invariants are established.

## Codegen

| Target     | Output                                    |
| ---------- | ----------------------------------------- |
| TypeScript | Nested if/else with type narrowing        |
| Python     | match with tuple patterns (3.10+)         |
| Rust       | match with tuple patterns                 |
| C++        | if/else chain with std::holds_alternative |
| Zig        | if/else chain with tagged union checks    |

## Related

- `casu`/`ceterum` keywords: see `consilia/futura/casu-ceterum.md`
- `custodi` with `ergo`: allowing `si ... ergo` inside `custodi` blocks (nice-to-have)
