# Operatores

Operators: arithmetic, logical, comparison, ternary, nullish coalescing, and ranges.

## Exempla

- `exempla/operatores/`

---

## Syntax

### Assignment

```ebnf
assignment := ternary (('=' | '+=' | '-=' | '*=' | '/=' | '&=' | '|=') assignment)?
```

### Ternary

```ebnf
ternary := or (('?' expression ':' | 'sic' expression 'secus') ternary)?
```

> Supports both symbolic (? :) and Latin (sic secus) syntax.
> The two forms cannot be mixed: use either ? : or sic secus.

**Examples:**

```fab
verum ? 1 : 0              // symbolic style
verum sic 1 secus 0        // Latin style
a ? b ? c : d : e          // nested (right-associative)
```

### Or

```ebnf
or := and (('||' | 'aut') and)* | and ('vel' and)*
```

> Latin 'aut' (or) for logical OR, 'vel' (or) for nullish coalescing.
> Mixing aut/|| with vel without parentheses is a syntax error
> (same as JavaScript's ?? and || restriction).

### And

```ebnf
and := equality ('&&' equality | 'et' equality)*
```

> Latin 'et' (and) supported alongside '&&'.

### Equality

```ebnf
equality := comparison (('==' | '!=' | '===' | '!==' | 'est' | 'non' 'est') comparison)*
```

> 'est' always means type check (instanceof/typeof).
> Use '===' or '!==' for value equality.
> Use 'nihil x' or 'nonnihil x' for null checks.

### Comparison

```ebnf
comparison := bitwiseOr (('<' | '>' | '<=' | '>=' | 'intra' | 'inter') bitwiseOr)*
```

> intra/inter at comparison level - same precedence as relational operators

### Range

```ebnf
range := additive (('..' | 'ante' | 'usque') additive ('per' additive)?)?
```

> Range expressions provide concise numeric iteration.
> Three operators with different end semantics:
> - '..' and 'ante': exclusive (0..10 / 0 ante 10 = 0-9)
> - 'usque': inclusive (0 usque 10 = 0-10)
> Optional step via 'per' keyword.

**Examples:**

```fab
0..10           -> RangeExpression(0, 10, inclusive=false)
0 ante 10       -> RangeExpression(0, 10, inclusive=false)
0 usque 10      -> RangeExpression(0, 10, inclusive=true)
0..10 per 2     -> RangeExpression(0, 10, 2, inclusive=false)
```

### Additive

```ebnf
additive := multiplicative (('+' | '-') multiplicative)*
```

### Multiplicative

```ebnf
multiplicative := unary (('*' | '/' | '%') unary)*
```

### Unary

```ebnf
unary := ('!' | '-' | 'non' | 'nulla' | 'nonnulla' | 'nihil' | 'nonnihil' | 'negativum' | 'positivum' | 'cede' | 'novum' | 'finge') unary | cast
```

> Latin 'non' (not), 'nulla' (none/empty), 'nonnulla' (some/non-empty),
> 'nihil' (is null), 'nonnihil' (is not null),
> 'negativum' (< 0), 'positivum' (> 0),
> 'cede' (await), 'novum' (new), 'finge' (form variant).

---

*Generated from `fons/faber/parser/index.ts` — do not edit directly.*