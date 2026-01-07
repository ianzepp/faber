# Lexer: Nested Generics with >>

## Problem

The lexer tokenizes `>>` as a single RIGHT_SHIFT operator token, breaking nested generic syntax.

```fab
# This fails to parse:
functio fragmenta(numerus n) -> lista<lista<T>>

# Error: Expected '>', got '>>'
```

## Solution

Remove `<<` and `>>` as tokens entirely. Replace bit shift operations with Latin postfix operators:

```fab
x dextratum 3      # x >> 3 (shift right)
x sinistratum 3    # x << 3 (shift left)
```

This eliminates the ambiguity completely — `>` is always a single token.

## Rationale

1. **Bit shifting is rare** — most application code never uses it
2. **Nested generics are common** — `lista<lista<T>>`, `tabula<K, lista<V>>`
3. **Consistent vocabulary** — `-atum` suffix matches `numeratum`, `fractatum`, `textatum`
4. **No context tracking** — lexer stays simple, no parser token splitting needed

## Syntax

```
shiftExpr := expression ('dextratum' | 'sinistratum') expression
```

Examples:
```fab
flags dextratum 4           # flags >> 4
1 sinistratum n             # 1 << n
(x dextratum 8) et 0xff     # (x >> 8) & 0xff
```

## Implementation

1. Remove `<<` (`LEFT_SHIFT`) and `>>` (`RIGHT_SHIFT`) from lexer
2. Add `dextratum` and `sinistratum` as operator keywords
3. Parse as binary postfix operators (like `numeratum` with `vel`)
4. Codegen emits `(expr >> amount)` or `(expr << amount)`

## Result

Nested generics work without any special handling:

```fab
lista<lista<T>>
tabula<textus, lista<numerus>>
lista<lista<lista<T>>>
functio fragmenta(numerus n) -> lista<lista<T>>
```

## Related

- parser-function-types.md - both needed for full lista.fab type signatures
