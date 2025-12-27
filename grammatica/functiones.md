# Functiones

Function declarations: basic functions, typed parameters, async, generators, and lambdas.

## Exempla

- `exempla/functiones/`

---

## Syntax

### Parameter List

```ebnf
paramList := (parameter (',' parameter)*)?
```

### Parameter

```ebnf
parameter := ('ad' | 'de' | 'in' | 'ex')? (typeAnnotation IDENTIFIER | IDENTIFIER)
```

> Type-first syntax: "textus name" or "ad textus recipientem"
> Prepositional prefixes indicate semantic roles:
> ad = toward/to, de = from/concerning (borrowed),
> in = in/into (mutable), ex = from/out of

### Arrow Function

```ebnf
arrowFunction := '(' paramList ')' '=>' (expression | blockStmt)
```

> Called after detecting '() =>' pattern in parsePrimary.

---

*Generated from `fons/parser/index.ts` — do not edit directly.*