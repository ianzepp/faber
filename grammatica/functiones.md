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

### Pro Expression

```ebnf
lambdaExpr := 'pro' params? ((':' | 'redde') expression | blockStmt)
params := IDENTIFIER (',' IDENTIFIER)*
```

> Latin 'pro' (for) creates lambda syntax with two equivalent forms:
> - 'pro x redde expr' - explicit return keyword
> - 'pro x: expr' - colon shorthand (mirrors object literal syntax)
> 
> The ':' and 'redde' forms are INTERCHANGEABLE - use whichever reads better:
> pro x: x * 2        ≡  pro x redde x * 2      -> (x) => x * 2
> pro: 42             ≡  pro redde 42           -> () => 42
> pro x, y: x + y     ≡  pro x, y redde x + y   -> (x, y) => x + y
> 
> Block form (for multi-statement bodies):
> pro x { redde x * 2 }     -> (x) => { return x * 2; }
> pro { scribe "hi" }       -> () => { console.log("hi"); }
> 
> Style guidance: Use ':' for short expressions, 'redde' for clarity in complex cases.

---

*Generated from `fons/parser/index.ts` — do not edit directly.*