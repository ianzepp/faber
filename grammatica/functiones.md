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
parameter := ('de' | 'in' | 'ex')? 'si'? 'ceteri'? (typeAnnotation IDENTIFIER | IDENTIFIER) ('ut' IDENTIFIER)? ('vel' expression)?
```

> Type-first syntax: "textus name" or "de textus source"
> Prepositional prefixes indicate semantic roles:
> de = from/concerning (borrowed, read-only),
> in = in/into (mutable borrow),
> ex = from/out of (source)
> 
> OPTIONAL PARAMETERS:
> 'si' marks a parameter as optional. Without 'vel', type becomes ignotum<T>.
> With 'vel', parameter has a default value and type stays T.
> Order: preposition, then si, then ceteri, then type, then name.

---

*Generated from `fons/faber/parser/index.ts` — do not edit directly.*