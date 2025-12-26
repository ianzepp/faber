# Regimen

Control flow: conditionals, loops, guards, assertions, and program structure.

## Exempla

- `exempla/regimen/`

---

## Syntax

### Program

```ebnf
program := statement*
```

### Statement

```ebnf
statement := importDecl | varDecl | funcDecl | typeAliasDecl | ifStmt | whileStmt | forStmt
| returnStmt | throwStmt | tryStmt | blockStmt | exprStmt
```

> Uses lookahead to determine statement type via keyword inspection.

### If Statement

```ebnf
ifStmt := 'si' expression (blockStmt | 'ergo' statement) ('cape' IDENTIFIER blockStmt)? (elseClause | 'sin' ifStmt)?
elseClause := ('aliter' | 'secus') (ifStmt | blockStmt | statement)
```

> 'cape' (catch/seize) clause allows error handling within conditionals.
> 'ergo' (therefore) for one-liner consequents.
> TWO STYLE OPTIONS (both supported, can be mixed within the same chain):
> Literal style: si / aliter si / aliter
> si x > 0 { positive() }
> aliter si x < 0 { negative() }
> aliter { zero() }
> Poetic style: si / sin / secus
> si x > 0 { positive() }
> sin x < 0 { negative() }    // "sin" = "but if" (classical Latin)
> secus { zero() }            // "secus" = "otherwise"
> Keywords are interchangeable at each branch point:
> - 'aliter si' ≡ 'sin' (else-if)
> - 'aliter' ≡ 'secus' (else)
> - Mixed: si ... sin ... aliter { } is valid

**Examples:**

```fab
si x > 5 ergo scribe("big")
si x > 5 { scribe("big") } aliter scribe("small")
si x < 0 { ... } sin x == 0 { ... } secus { ... }
```

### While Statement

```ebnf
whileStmt := 'dum' expression (blockStmt | 'ergo' statement) ('cape' IDENTIFIER blockStmt)?
```

> 'dum' (while/until) for while loops.

**Examples:**

```fab
dum x > 0 { x = x - 1 }
dum x > 0 ergo x = x - 1
```

### Ex Statement

```ebnf
exStmt := 'ex' expression (forBinding | destructBinding)
forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement) catchClause?
destructBinding := ('fixum' | 'varia' | 'figendum' | 'variandum') objectPattern
```

> 'ex' (from/out of) introduces both iteration and extraction:
> - Iteration: ex items pro item { ... } (for each item from items)
> - Destructuring: ex response fixum { data } (extract data from response)
> - Async destructuring: ex promise figendum { result } (await + extract)
> The binding keywords encode mutability and async semantics:
> - fixum: immutable binding (const)
> - varia: mutable binding (let)
> - figendum: immutable + await (const with await)
> - variandum: mutable + await (let with await)

**Examples:**

```fab
ex numeri pro n { ... }              // for-loop (sync)
ex numeri fiet n { ... }             // for-await-of loop (async)
ex response fixum { status, data }   // destructuring (sync)
ex fetchData() figendum { result }   // destructuring (async, awaits first)
```

### De Statement

```ebnf
deStmt := 'de' expression ('pro' | 'fit' | 'fiet') IDENTIFIER
(blockStmt | 'ergo' statement) catchClause?
```

> 'de' (from/concerning) for extracting keys from an object.
> Semantically read-only - contrasts with 'in' for mutation.

**Examples:**

```fab
de tabula pro clavis { ... }  // from table, for each key
de object pro k ergo scribe k // one-liner form
```

### In Statement

```ebnf
inStmt := 'in' expression blockStmt
```

> 'in' (into) for reaching into an object to modify it.
> Semantically mutable - contrasts with 'de' for read-only iteration.

**Examples:**

```fab
in user { nomen = "Marcus" }  // mutation block
```

### Switch Statement

```ebnf
switchStmt := 'elige' expression '{' switchCase* defaultCase? '}' catchClause?
switchCase := 'si' expression (blockStmt | 'ergo' expression)
defaultCase := ('aliter' | 'secus') (blockStmt | statement)
```

> 'elige' (choose) for switch, 'si' (if) for cases, 'ergo' (therefore) for one-liners.
> 'aliter'/'secus' (otherwise) doesn't need 'ergo' - it's already the consequence.

**Examples:**

```fab
elige status {
si "pending" ergo scribe("waiting")
si "active" { processActive() }
aliter iace "Unknown status"
}
```

### Guard Statement

```ebnf
guardStmt := 'custodi' '{' guardClause+ '}'
guardClause := 'si' expression blockStmt
```

> 'custodi' (guard!) groups early-exit conditions.

**Examples:**

```fab
custodi {
si user == nihil { redde nihil }
si useri age < 0 { iace "Invalid age" }
}
```

### Assert Statement

```ebnf
assertStmt := 'adfirma' expression (',' expression)?
```

> 'adfirma' (affirm/assert) for runtime invariant checks.

**Examples:**

```fab
adfirma x > 0
adfirma x > 0, "x must be positive"
```

### Return Statement

```ebnf
returnStmt := 'redde' expression?
```

> 'redde' (give back/return) for return statements.

### Break Statement

```ebnf
breakStmt := 'rumpe'
```

> 'rumpe' (break!) exits the innermost loop.

### Continue Statement

```ebnf
continueStmt := 'perge'
```

> 'perge' (continue/proceed!) skips to the next loop iteration.

### Scribe Statement

```ebnf
outputStmt := ('scribe' | 'vide' | 'mone') expression (',' expression)*
```

> Latin output keywords as statement forms:
> scribe (write!) → console.log
> vide (see!)     → console.debug
> mone (warn!)    → console.warn

**Examples:**

```fab
scribe "hello"
vide "debugging:", value
mone "warning:", message
```

### Probandum Statement

```ebnf
probandumDecl := 'probandum' STRING '{' probandumBody '}'
probandumBody := (curaBlock | probandumDecl | probaStmt)*
```

> Latin "probandum" (gerundive of probare) = "that which must be tested".
> Analogous to describe() in Jest/Vitest.

**Examples:**

```fab
probandum "Tokenizer" {
cura ante { lexer = init() }
proba "parses numbers" { ... }
}
```

### Proba Statement

```ebnf
probaStmt := 'proba' probaModifier? STRING blockStmt
probaModifier := 'omitte' STRING | 'futurum' STRING
```

> Latin "proba" (imperative of probare) = "test!" / "prove!".
> Analogous to test() or it() in Jest/Vitest.

**Examples:**

```fab
proba "parses integers" { adfirma parse("42") est 42 }
proba omitte "blocked by #42" { ... }
proba futurum "needs async support" { ... }
```

### Cura Block

```ebnf
curaBlock := 'cura' ('ante' | 'post') 'omnia'? blockStmt
```

> Latin "cura" (care, concern) for resource management.
> In test context:
> cura ante { } = beforeEach (care before each test)
> cura ante omnia { } = beforeAll (care before all tests)
> cura post { } = afterEach (care after each test)
> cura post omnia { } = afterAll (care after all tests)

**Examples:**

```fab
cura ante { lexer = init() }
cura ante omnia { db = connect() }
cura post { cleanup() }
cura post omnia { db.close() }
```

### Block Statement

```ebnf
blockStmt := '{' statement* '}'
```

### Expression Statement

```ebnf
exprStmt := expression
```

### Expression

```ebnf
expression := assignment
```

> Top-level expression delegates to assignment (lowest precedence).

### Cast

```ebnf
castExpr := call ('ut' typeAnnotation)*
```

> Latin 'ut' (as, in the capacity of) for type assertions.
> Compile-time only — no runtime checking. Maps to:
> - TypeScript: x as T
> - Python: x (ignored, dynamic typing)
> - Zig: @as(T, x)
> - Rust: x as T
> - C++: static_cast<T>(x)

---

*Generated from `fons/parser/index.ts` — do not edit directly.*