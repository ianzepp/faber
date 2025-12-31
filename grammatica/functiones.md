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
parameter := ('de' | 'in' | 'ex')? (typeAnnotation IDENTIFIER | IDENTIFIER)
```

> Type-first syntax: "textus name" or "de textus source"
> Prepositional prefixes indicate semantic roles:
> de = from/concerning (borrowed, read-only),
> in = in/into (mutable borrow),
> ex = from/out of (source)

---

*Generated from `fons/parser/index.ts` — do not edit directly.*