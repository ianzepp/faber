# Lexer: Nested Generics with >>

## Problem

The lexer tokenizes `>>` as a single RIGHT_SHIFT operator token, breaking nested generic syntax.

```fab
# This fails to parse:
functio fragmenta(numerus n) -> lista<lista<T>>

# Error: Expected '>', got '>>'
```

## Current Behavior

The lexer greedily consumes `>>` as one token:
- `lista<lista<T>>` becomes tokens: `lista`, `<`, `lista`, `<`, `T`, `>>`
- Parser expects `>` but gets `>>`

## Required Behavior

Context-aware tokenization or parser-level splitting:

```fab
# Should work:
lista<lista<T>>
tabula<textus, lista<numerus>>
lista<lista<lista<T>>>
```

## Solutions

### Option A: Lexer Lookahead
When inside generic context, tokenize `>` individually even when followed by `>`.

### Option B: Parser Token Splitting
Parser splits `>>` into two `>` tokens when parsing generic type parameters.

### Option C: Whitespace Convention
Require space: `lista<lista<T> >` (ugly, not recommended)

## Recommendation

**Option B (Parser Token Splitting)** is cleanest:
- Lexer stays simple
- Parser already knows it's in generic context
- Common approach (C++11 solved this same problem)

## Implementation Notes

- Track generic depth in parser
- When expecting `>` and see `>>`, consume as single `>` and leave synthetic `>` for next iteration
- Same applies to `>>>` in triple-nested generics

## Use Cases

1. `lista<lista<T>>` - nested lists
2. `tabula<K, lista<V>>` - map of lists
3. `lista<tabula<K, V>>` - list of maps
4. Return types in lista.fab: `fragmenta() -> lista<lista<T>>`

## Related

- parser-function-types.md - both needed for full lista.fab type signatures
