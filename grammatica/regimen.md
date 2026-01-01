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

### Specifier

```ebnf
specifier := 'ceteri'? IDENTIFIER ('ut' IDENTIFIER)?
```

> Shared between imports and destructuring.
> 'ceteri' (rest) is only valid in destructuring contexts.
> 'ut' provides aliasing: nomen ut n

**Examples:**

```fab
scribe             -> imported=scribe, local=scribe
scribe ut s        -> imported=scribe, local=s
ceteri rest        -> imported=rest, local=rest, rest=true
```

### Importa Declaration

```ebnf
importDecl := 'ex' (STRING | IDENTIFIER) 'importa' (specifierList | '*')
specifierList := specifier (',' specifier)*
specifier := IDENTIFIER ('ut' IDENTIFIER)?
```

**Examples:**

```fab
ex norma importa scribe, lege
ex norma importa scribe ut s, lege ut l
ex "norma/tempus" importa nunc, dormi
ex norma importa *
```

### Varia Declaration

```ebnf
varDecl := ('varia' | 'fixum' | 'figendum' | 'variandum') typeAnnotation? IDENTIFIER ('=' expression)?
arrayDestruct := ('varia' | 'fixum' | 'figendum' | 'variandum') arrayPattern '=' expression
```

> Type-first syntax: "fixum textus nomen = value" or "fixum nomen = value"
> Latin 'varia' (let it be) for mutable, 'fixum' (fixed) for immutable.

### Array Pattern

```ebnf
arrayPattern := '[' arrayPatternElement (',' arrayPatternElement)* ']'
arrayPatternElement := '_' | 'ceteri'? IDENTIFIER
```

**Examples:**

```fab
[a, b, c]                 # extract first three elements
[first, ceteri rest]     # extract first, collect rest
[_, second, _]           # skip first and third, extract second

T SUPPORTED:
[...rest]                # JS spread syntax
[*rest]                  # Python unpack syntax
```

### Functio Declaration

```ebnf
funcDecl := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt
paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
typeParamDecl := 'prae' 'typus' IDENTIFIER
funcModifier := 'futura' | 'cursor' | 'curata' IDENTIFIER
returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
```

> All function declarations start with 'functio' for consistent parsing.
> Modifiers come after the parameter list, before the return clause.
> 'futura' marks async functions (future/promise-based).
> 'cursor' marks generator functions (yield-based).
> 'curata NAME' marks managed functions (receives allocator as NAME).
> 
> TYPE PARAMETERS: 'prae typus T' declares compile-time type parameters.
> functio max(prae typus T, T a, T b) -> T { ... }
> Maps to: <T> (TS/Rust), TypeVar (Py), comptime T: type (Zig)
> 
> RETURN TYPE VERBS: Latin verb forms encode async/generator semantics directly:
> '->'    neutral arrow (semantics from modifier only)
> 'fit'   "it becomes" - sync, returns single value
> 'fiet'  "it will become" - async, returns Promise<T>
> 'fiunt' "they become" - sync generator, yields multiple values
> 'fient' "they will become" - async generator, yields Promise values
> 
> When using verb forms, the futura/cursor modifier is NOT required - the verb
> itself carries the semantic information. The modifier becomes redundant:
> functio compute() -> numerus { ... }         // arrow: sync by default
> functio compute() fit numerus { ... }        // verb: explicitly sync
> functio fetch() futura -> textus { ... }     // modifier: async
> functio fetch() fiet textus { ... }          // verb implies async
> functio items() cursor -> numerus { ... }    // modifier: generator
> functio items() fiunt numerus { ... }        // verb implies generator
> functio stream() fient datum { ... }         // verb implies async generator
> functio alloc(textus s) curata a -> T { ... } // managed, allocator bound as 'a'
> 
> Modifier is still allowed for emphasis, but verb/modifier conflicts are errors.
> 
> NOT SUPPORTED (will produce parser errors):
> - TS-style param annotation: functio f(x: textus) (use: functio f(textus x))
> - TS-style return type: functio f(): textus (use: functio f() -> textus)
> - Trailing comma in params: functio f(a, b,)

### Type And Parameter List

```ebnf
paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
typeParamDecl := 'prae' 'typus' IDENTIFIER
```

> Type parameters (prae typus T) must come first, followed by regular params.
> This matches the conventions of TypeScript, Rust, and Zig.

**Examples:**

```fab
(prae typus T, T a, T b)     -> typeParams=[T], params=[a, b]
(prae typus T, prae typus U) -> typeParams=[T, U], params=[]
(numerus a, numerus b)       -> typeParams=[], params=[a, b]
```

### Ordo Declaration

```ebnf
enumDecl := 'ordo' IDENTIFIER '{' enumMember (',' enumMember)* ','? '}'
enumMember := IDENTIFIER ('=' ('-'? NUMBER | STRING))?
```

> Latin 'ordo' (order/rank) for enumerated constants.

**Examples:**

```fab
ordo color { rubrum, viridis, caeruleum }
ordo status { pendens = 0, actum = 1, finitum = 2 }
ordo offset { ante = -1, ad = 0, post = 1 }
```

### Discretio Declaration

```ebnf
discretioDecl := 'discretio' IDENTIFIER typeParams? '{' variant (',' variant)* ','? '}'
variant := IDENTIFIER ('{' variantFields '}')?
variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
```

> Latin 'discretio' (distinction) for tagged unions.
> Each variant has a compiler-managed tag for exhaustive pattern matching.

**Examples:**

```fab
discretio Event {
    Click { numerus x, numerus y }
    Keypress { textus key }
    Quit
}

discretio Option<T> {
    Some { T value }
    None
}
```

### Variant Declaration

```ebnf
variant := IDENTIFIER ('{' variantFields '}')?
variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
```

> Variant names are capitalized by convention (like type names).
> Fields use type-first syntax like genus fields.

**Examples:**

```fab
Click { numerus x, numerus y }  -> fields with payload
Quit                            -> unit variant (no payload)
```

### Si Statement

```ebnf
ifStmt := 'si' expression (blockStmt | 'ergo' statement) ('cape' IDENTIFIER blockStmt)? (elseClause | 'sin' ifStmt)?
elseClause := ('secus' | 'secus') (ifStmt | blockStmt | statement)
```

> 'cape' (catch/seize) clause allows error handling within conditionals.
> 'ergo' (therefore) for one-liner consequents.
> 
> TWO STYLE OPTIONS (both supported, can be mixed within the same chain):
> 
> Literal style: si / sin / secus
> si x > 0 { positive() }
> sin x < 0 { negative() }
> secus { zero() }
> 
> Poetic style: si / sin / secus
> si x > 0 { positive() }
> sin x < 0 { negative() }    // "sin" = "but if" (classical Latin)
> secus { zero() }            // "secus" = "otherwise"
> 
> Keywords are interchangeable at each branch point:
> - 'sin' ≡ 'sin' (else-if)
> - 'secus' ≡ 'secus' (else)
> - Mixed: si ... sin ... secus { } is valid

**Examples:**

```fab
si x > 5 ergo scribe("big")
si x > 5 { scribe("big") } secus scribe("small")
si x < 0 { ... } sin x == 0 { ... } secus { ... }
```

### Dum Statement

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
exStmt := 'ex' expression (forBinding | destructBinding | arrayDestructBinding)
forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement) catchClause?
destructBinding := ('fixum' | 'varia' | 'figendum' | 'variandum') specifierList
arrayDestructBinding := ('fixum' | 'varia' | 'figendum' | 'variandum') arrayPattern
specifierList := specifier (',' specifier)*
specifier := 'ceteri'? IDENTIFIER ('ut' IDENTIFIER)?
```

> 'ex' (from/out of) introduces both iteration and extraction:
> - Iteration: ex items pro item { ... } (for each item from items)
> - Object destructuring: ex persona fixum nomen, aetas (extract properties)
> - Array destructuring: ex coords fixum [x, y, z] (extract by position)
> - Async destructuring: ex promise figendum result (await + extract)
> 
> The binding keywords encode mutability and async semantics:
> - fixum: immutable binding (const)
> - varia: mutable binding (let)
> - figendum: immutable + await (const with await)
> - variandum: mutable + await (let with await)

**Examples:**

```fab
ex numeri pro n { ... }              # for-loop (sync)
ex numeri fiet n { ... }             # for-await-of loop (async)
ex persona fixum nomen, aetas        # object destructuring
ex persona fixum nomen ut n          # object destructuring with alias
ex persona fixum nomen, ceteri rest  # object destructuring with rest
ex coords fixum [x, y, z]            # array destructuring
ex fetchData() figendum result       # async destructuring

llection DSL forms:
ex items prima 5 pro item { }        # iteration with transforms
ex items prima 5, ultima 2 pro x {}  # multiple transforms
```

### D S L Transforms

```ebnf
dslTransforms := dslTransform (',' dslTransform)*
dslTransform := dslVerb expression?
dslVerb := 'prima' | 'ultima' | 'summa'
```

> DSL provides concise syntax for common collection operations.
> Transforms chain with commas: prima 5, ultima 3

**Examples:**

```fab
prima 5           -> first 5 elements
ultima 3          -> last 3 elements
summa             -> sum (no argument)
prima 5, ultima 2 -> first 5, then last 2 of those
```

### Collection D S L Expression

```ebnf
dslExpr := 'ex' expression dslTransform (',' dslTransform)*
```

> When 'ex' appears in expression context with DSL verbs (not pro/fit/fiet),
> it creates a collection pipeline expression that can be assigned.

**Examples:**

```fab
fixum top5 = ex items prima 5
fixum total = ex prices summa
fixum result = ex items prima 10, ultima 3
```

### Ab Expression

```ebnf
abExpr := 'ab' expression filter? (',' transform)*
filter := ['non'] ('ubi' condition | identifier)
condition := expression
```

> 'ab' (away from) is the dedicated DSL entry point for filtering.
> The 'ex' preposition remains unchanged for iteration/import/destructuring.
> Include/exclude is handled via 'non' keyword.

**Examples:**

```fab
ab users activus                     # boolean property shorthand
ab users non banned                  # negated boolean property
ab users ubi aetas >= 18             # condition with ubi
ab users non ubi banned et suspended # negated compound condition
ab users activus, prima 10           # filter + transforms
ab users activus pro user { }        # iteration form
```

### Regex Literal

```ebnf
regexLiteral := 'sed' STRING IDENTIFIER?
```

> 'sed' (the Unix stream editor) is synonymous with pattern matching.
> The pattern string is passed through verbatim to the target.
> Flags are a bare identifier after the pattern (no comma).

**Examples:**

```fab
sed "\\d+"           # pattern only
sed "hello" i        # case insensitive
sed "^start" im      # multiple flags
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
de tabula pro clavis { ... }  # from table, for each key
de object pro k ergo scribe k # one-liner form
```

### In Statement

```ebnf
inStmt := 'in' expression blockStmt
```

> 'in' (into) for reaching into an object to modify it.
> Semantically mutable - contrasts with 'de' for read-only iteration.

**Examples:**

```fab
in user { nomen = "Marcus" }  # mutation block
```

### Elige Statement

```ebnf
eligeStmt := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
eligeCase := 'si' expression (blockStmt | 'ergo' expression)
defaultCase := ('secus' | 'secus') (blockStmt | statement)
```

> 'elige' (choose) for value-based switch.
> 'ergo' (therefore) for one-liners, 'secus'/'secus' (otherwise) for default.
> For variant matching on discretio types, use 'discerne' instead.

**Examples:**

```fab
elige status {
    si "pending" ergo scribe("waiting")
    si "active" { processActive() }
    secus iace "Unknown status"
}
```

### Discerne Statement

```ebnf
discerneStmt := 'discerne' expression '{' variantCase* '}'
variantCase := 'si' IDENTIFIER ('pro' IDENTIFIER (',' IDENTIFIER)*)? blockStmt
```

> 'discerne' (distinguish!) pairs with 'discretio' (the tagged union type).
> Uses 'si' for conditional match, 'pro' to introduce bindings.

**Examples:**

```fab
discerne event {
    si Click pro x, y { scribe "clicked at " + x + ", " + y }
    si Keypress pro key { scribe "pressed " + key }
    si Quit { mori "goodbye" }
}
```

### Custodi Statement

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

### Adfirma Statement

```ebnf
assertStmt := 'adfirma' expression (',' expression)?
```

> 'adfirma' (affirm/assert) for runtime invariant checks.

**Examples:**

```fab
adfirma x > 0
adfirma x > 0, "x must be positive"
```

### Redde Statement

```ebnf
returnStmt := 'redde' expression?
```

> 'redde' (give back/return) for return statements.

### Rumpe Statement

```ebnf
breakStmt := 'rumpe'
```

> 'rumpe' (break!) exits the innermost loop.

### Perge Statement

```ebnf
continueStmt := 'perge'
```

> 'perge' (continue/proceed!) skips to the next loop iteration.

### Iace Statement

```ebnf
throwStmt := ('iace' | 'mori') expression
```

> Two error severity levels:
> iace (throw!) → recoverable, can be caught
> mori (die!)   → fatal/panic, unrecoverable

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

### Tempta Statement

```ebnf
tryStmt := 'tempta' blockStmt ('cape' IDENTIFIER blockStmt)? ('demum' blockStmt)?
```

> 'tempta' (attempt/try), 'cape' (catch/seize), 'demum' (finally/at last).

### Cape Clause

```ebnf
catchClause := 'cape' IDENTIFIER blockStmt
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
    praepara { lexer = init() }
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

### Ad Statement

```ebnf
adStmt := 'ad' STRING '(' argumentList ')' adBinding? blockStmt? catchClause?
adBinding := adBindingVerb typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
adBindingVerb := 'fit' | 'fiet' | 'fiunt' | 'fient'
argumentList := (expression (',' expression)*)?
```

> Latin 'ad' (to/toward) dispatches to named endpoints:
> - Stdlib syscalls: "fasciculus:lege", "console:log"
> - External packages: "hono/Hono"
> - Remote services: "https://api.example.com/users"
> 
> Binding verbs encode sync/async and single/plural:
> - fit: sync, single ("it becomes")
> - fiet: async, single ("it will become")
> - fiunt: sync, plural ("they become")
> - fient: async, plural ("they will become")

**Examples:**

```fab
ad "console:log" ("hello")                           # fire-and-forget
ad "fasciculus:lege" ("file.txt") fit textus pro c { }  # sync binding
ad "http:get" (url) fiet Response pro r { }          # async binding
ad "http:batch" (urls) fient Response[] pro rs { }   # async plural
```

### Praepara Block

```ebnf
praeparaBlock := ('praepara' | 'praeparabit' | 'postpara' | 'postparabit') 'omnia'? blockStmt
```

> Latin "praepara" (prepare!) for test setup, "postpara" (cleanup!) for teardown.
> Uses -bit suffix for async (future tense), matching fit/fiet pattern.

**Examples:**

```fab
praepara { lexer = init() }
praepara omnia { db = connect() }
praeparabit omnia { db = cede connect() }
postpara { cleanup() }
postpara omnia { db.close() }
postparabit omnia { cede db.close() }
```

### Cura Statement

```ebnf
curaStmt := 'cura' curatorKind? expression? ('pro' | 'fit' | 'fiet') typeAnnotation? IDENTIFIER blockStmt catchClause?
curatorKind := 'arena' | 'page'
```

> Latin "cura" (care) + binding verb for scoped resources.
> - pro: neutral binding ("for")
> - fit: sync binding ("it becomes")
> - fiet: async binding ("it will become")
> Curator kinds declare explicit allocator types (arena, page).
> Guarantees cleanup via solve() on scope exit.

**Examples:**

```fab
cura arena fit mem { ... }                    # arena allocator
cura page fit mem { ... }                     # page allocator
cura aperi("data.bin") fit fd { lege(fd) }   # generic resource
cura connect(url) fiet conn { ... }          # async resource
```

### Incipit Statement

```ebnf
incipitStmt := 'incipit' (blockStmt | 'ergo' statement)
```

> 'incipit' (it begins) marks the program entry point.
> This is a pure structural marker with no magic injection.
> The source is responsible for any setup (allocators via cura, etc.).
> 
> The 'ergo' (therefore) form chains to a single statement, typically
> a cura block for allocator setup. This avoids extra nesting.

**Examples:**

```fab
incipit {
    scribe "Hello"
}

incipit ergo cura arena {
```

### Incipiet Statement

```ebnf
incipietStmt := 'incipiet' (blockStmt | 'ergo' statement)
```

> 'incipiet' (it will begin) marks the async program entry point.
> Mirrors the fit/fiet pattern: present for sync, future for async.
> 
> The 'ergo' form chains to a single statement for concise setup.

**Examples:**

```fab
incipiet {
    fixum data = cede fetchData()
    scribe data
}

incipiet ergo cura arena {
    fixum data = cede fetchData()
}
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

### Bitwise Or

```ebnf
bitwiseOr := bitwiseXor ('|' bitwiseXor)*
```

> Bitwise precedence is above comparison (unlike C), so
> `flags & MASK == 0` parses as `(flags & MASK) == 0`.

### Bitwise Xor

```ebnf
bitwiseXor := bitwiseAnd ('^' bitwiseAnd)*
```

### Bitwise And

```ebnf
bitwiseAnd := shift ('&' shift)*
```

### Shift

```ebnf
shift := range (('<<' | '>>') range)*
```

### Praefixum Expression

```ebnf
praefixumExpr := 'praefixum' (blockStmt | '(' expression ')')
```

> Latin 'praefixum' (pre-fixed) extends fixum vocabulary.
> Block form: praefixum { ... } for multi-statement computation
> Expression form: praefixum(expr) for simple expressions
> 
> TARGET SUPPORT:
> Zig:    comptime { } or comptime (expr)
> C++:    constexpr
> Rust:   const (in const context)
> TS/Py:  Semantic error - not supported

**Examples:**

```fab
fixum size = praefixum(256 * 4)
fixum table = praefixum {
    varia result = []
    ex 0..10 pro i { result.adde(i * i) }
    redde result
}
```

### Scriptum Expression

```ebnf
scriptumExpr := 'scriptum' '(' STRING (',' expression)* ')'
```

> "scriptum" (that which has been written) is the perfect passive participle
> of scribere. While scribe outputs to console, scriptum returns a formatted string.
> 
> Format string is passed through verbatim to target. User must use target-appropriate
> placeholders ({} for Zig/Rust, %s/%d for C++, etc.). Faber does not translate.

**Examples:**

```fab
scriptum("Hello, {}", name)
scriptum("{} + {} = {}", a, b, a + b)
```

### Lege Expression

```ebnf
legeExpr := 'lege' '(' ')'
```

### Qua Expression

```ebnf
castExpr := call ('qua' typeAnnotation)*
```

> Latin 'qua' (as, in the capacity of) for type assertions.
> Compile-time only — no runtime checking. Maps to:
> - TypeScript: x as T
> - Python: x (ignored, dynamic typing)
> - Zig: @as(T, x)
> - Rust: x as T
> - C++: static_cast<T>(x)

### Novum Expression

```ebnf
newExpr := 'novum' IDENTIFIER ('(' argumentList ')')? (objectLiteral | 'de' expression)?
```

> Two forms for property overrides:
> - Inline literal: `novum Persona { nomen: "Marcus" }`
> - From expression: `novum Persona de props` (props is variable/call/etc.)
> 
> The `de` (from) form allows dynamic overrides from variables or function results.

### Lambda Expression

```ebnf
lambdaExpr := ('pro' | 'fit' | 'fiet') params? ('->' type)? (':' expression | blockStmt)
params := IDENTIFIER (',' IDENTIFIER)*
```

### Identifier Or Keyword

```ebnf
identifierOrKeyword := IDENTIFIER | KEYWORD
```

> Import specifiers can be keywords (ex norma importa scribe).
> In this context, 'scribe' is a valid name, not a statement keyword.

---

*Generated from `fons/parser/index.ts` — do not edit directly.*
