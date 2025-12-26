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
> RETURN TYPE VERBS: Latin verb forms encode async/generator semantics directly:
> '->'    neutral arrow (semantics from prefix only)
> 'fit'   "it becomes" - sync, returns single value
> 'fiet'  "it will become" - async, returns Promise<T>
> 'fiunt' "they become" - sync generator, yields multiple values
> 'fient' "they will become" - async generator, yields Promise values
> When using verb forms, the futura/cursor prefix is NOT required - the verb
> itself carries the semantic information. The prefix becomes redundant:
> functio compute() -> numerus { ... }    // arrow: sync by default
> functio compute() fit numerus { ... }   // verb: explicitly sync
> functio fetch() fiet textus { ... }     // verb implies async (no futura needed)
> functio items() fiunt numerus { ... }   // verb implies generator (no cursor needed)
> functio stream() fient datum { ... }    // verb implies async generator
> Prefix is still allowed for emphasis, but verb/prefix conflicts are errors.
> NOT SUPPORTED (will produce parser errors):
> - TS-style param annotation: functio f(x: textus) (use: functio f(textus x))
> - TS-style return type: functio f(): textus (use: functio f() -> textus)
> - Trailing comma in params: functio f(a, b,)

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
> The ':' and 'redde' forms are INTERCHANGEABLE - use whichever reads better:
> pro x: x * 2        ≡  pro x redde x * 2      -> (x) => x * 2
> pro: 42             ≡  pro redde 42           -> () => 42
> pro x, y: x + y     ≡  pro x, y redde x + y   -> (x, y) => x + y
> Block form (for multi-statement bodies):
> pro x { redde x * 2 }     -> (x) => { return x * 2; }
> pro { scribe "hi" }       -> () => { console.log("hi"); }
> Style guidance: Use ':' for short expressions, 'redde' for clarity in complex cases.

---

*Generated from `fons/parser/index.ts` — do not edit directly.*