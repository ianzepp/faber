# Operatores

Operators: arithmetic, logical, comparison, ternary, nullish coalescing, and ranges.

## Exempla

- `exempla/operatores/`

---

## Syntax

### Assignment

```ebnf
assignment := ternary ('=' assignment)?
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
equality := comparison (('==' | '!=' | 'est' | 'non' 'est') comparison)*
```

> 'est' (is) is the Latin copula for strict equality (===).
> 'non est' (is not) is strict inequality (!==).
> Allows natural syntax: si x est nihil { ... }

### Comparison

```ebnf
comparison := range (('<' | '>' | '<=' | '>=') range)*
```

### Range

```ebnf
range := additive ('..' additive ('per' additive)?)?
```

> Range expressions provide concise numeric iteration.
> End is inclusive: 0..10 includes 10.
> Optional step via 'per' keyword.

**Examples:**

```fab
0..10           -> RangeExpression(0, 10)
0..10 per 2     -> RangeExpression(0, 10, 2)
start..end      -> RangeExpression(start, end)
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
unary := ('!' | '-' | 'non' | 'nulla' | 'nonnulla' | 'negativum' | 'positivum' | 'cede' | 'novum') unary | call
```

> Latin 'non' (not), 'nulla' (none/empty), 'nonnulla' (some/non-empty),
> 'negativum' (< 0), 'positivum' (> 0), 'cede' (await), 'novum' (new).

---

*Generated from `fons/parser/index.ts` — do not edit directly.*