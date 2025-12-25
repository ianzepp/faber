# Functiones

Function declarations: basic functions, typed parameters, async, generators, and lambdas.

## Exempla

- `exempla/functiones/`

---

## Syntax

### Function Declaration

```ebnf
funcDecl := ('futura' | 'cursor')* 'functio' IDENTIFIER '(' paramList ')' returnClause? blockStmt
returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
```

> Arrow syntax for return types: "functio greet(textus name) -> textus"
> 'futura' prefix marks async functions (future/promise-based).
> 'cursor' prefix marks generator functions (yield-based).
> Verb forms encode semantics: fit (sync), fiet (async), fiunt (generator), fient (async generator).

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

> Latin 'pro' (for) + 'redde' (return) creates lambda syntax.
> The ':' shorthand mirrors object literal syntax (x: value = "x is defined as value").
> Zero-param expr: pro redde 42, pro: 42 -> () => 42
> Single param expr: pro x redde x * 2, pro x: x * 2 -> (x) => x * 2
> Multi param expr: pro x, y redde x + y, pro x, y: x + y -> (x, y) => x + y
> Block form: pro x { redde x * 2 } -> (x) => { return x * 2; }
> Zero-param block: pro { scribe "hi" } -> () => { console.log("hi"); }

---

*Generated from `fons/parser/index.ts` — do not edit directly.*