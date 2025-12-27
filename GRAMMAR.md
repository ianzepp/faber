# Faber Romanus Grammar

Complete syntax reference for the Faber Romanus programming language.

## For LLMs

This document is designed for both human readers and LLM code generation. When generating Faber code:

**Style preferences:**

- PREFER Latin keywords over symbols: `et` over `&&`, `aut` over `||`, `non` over `!`
- PREFER `si`/`sin`/`secus` (poetic) or `si`/`aliter si`/`aliter` (literal) consistently
- PREFER `pro x: expr` for short lambdas, `pro x redde expr` when clarity helps
- ALWAYS use type-first syntax: `textus nomen` not `nomen: textus`
- NEVER use JavaScript/TypeScript/Python syntax where Faber has its own

**Common mistakes to avoid:**

- `return` instead of `redde`
- `const`/`let` instead of `fixum`/`varia`
- `if`/`else` instead of `si`/`aliter`
- `for...of` instead of `ex...pro`
- `string`/`number`/`boolean` instead of `textus`/`numerus`/`bivalens`
- `null` instead of `nihil`
- `this` instead of `ego`
- `new` instead of `novum`
- `await` instead of `cede`
- `async function` instead of `futura functio`

---

## Quick Reference

### Types

| Faber      | TypeScript   | Python     | Zig          | Meaning        |
| ---------- | ------------ | ---------- | ------------ | -------------- |
| `textus`   | `string`     | `str`      | `[]const u8` | text/string    |
| `numerus`  | `number`     | `int`      | `i64`        | integer        |
| `fractus`  | `number`     | `float`    | `f64`        | floating point |
| `decimus`  | `number`     | `Decimal`  | -            | decimal        |
| `magnus`   | `bigint`     | `int`      | `i128`       | big integer    |
| `bivalens` | `boolean`    | `bool`     | `bool`       | boolean        |
| `nihil`    | `null`       | `None`     | `null`       | null           |
| `vacuum`   | `void`       | `None`     | `void`       | void           |
| `numquam`  | `never`      | `NoReturn` | `noreturn`   | never          |
| `ignotum`  | `unknown`    | `Any`      | -            | unknown        |
| `octeti`   | `Uint8Array` | `bytes`    | `[]u8`       | bytes          |
| `objectum` | `object`     | `object`   | -            | object         |

### Generic Collections

| Faber          | TypeScript    | Python         | Meaning        |
| -------------- | ------------- | -------------- | -------------- |
| `lista<T>`     | `T[]`         | `list[T]`      | array/list     |
| `tabula<K,V>`  | `Map<K,V>`    | `dict[K,V]`    | map/dictionary |
| `copia<T>`     | `Set<T>`      | `set[T]`       | set            |
| `promissum<T>` | `Promise<T>`  | `Awaitable[T]` | promise/future |
| `cursor<T>`    | `Iterator<T>` | `Iterator[T]`  | iterator       |
| `unio<A,B>`    | `A \| B`      | `A \| B`       | union type     |

### Literals

| Faber    | Meaning   |
| -------- | --------- |
| `verum`  | true      |
| `falsum` | false     |
| `nihil`  | null      |
| `ego`    | this/self |

### Keywords by Category

**Declarations:**

- `fixum` — immutable binding (const)
- `varia` — mutable binding (let)
- `functio` — function
- `genus` — class/struct
- `pactum` — interface/protocol
- `typus` — type alias
- `ordo` — enum
- `discretio` — tagged union

**Control flow:**

- `si` / `sin` / `aliter` / `secus` — if / else-if / else
- `dum` — while
- `ex...pro` — for-of (iteration)
- `de...pro` — for-in (keys)
- `elige` — switch/match
- `custodi` — guard clauses
- `rumpe` — break
- `perge` — continue

**Functions:**

- `redde` — return
- `futura` — async modifier
- `cede` — await
- `cursor` — generator modifier
- `pro x: expr` — lambda expression

**Error handling:**

- `tempta` — try
- `cape` — catch
- `demum` — finally
- `iace` — throw (recoverable)
- `mori` — panic (fatal)
- `adfirma` — assert

**Output:**

- `scribe` — console.log
- `vide` — console.debug
- `mone` — console.warn

**Operators:**

- `et` — logical and (&&)
- `aut` — logical or (||)
- `non` — logical not (!)
- `vel` — nullish coalescing (??)
- `est` — instanceof/typeof check
- `qua` — type cast (as)

### Collection Methods (lista)

Common array methods (see README for complete list):

| Latin               | JavaScript          | Mutates? |
| ------------------- | ------------------- | -------- |
| `adde(x)`           | `push(x)`           | yes      |
| `remove()`          | `pop()`             | yes      |
| `primus`            | `[0]`               | no       |
| `ultimus`           | `[arr.length-1]`    | no       |
| `longitudo`         | `.length`           | no       |
| `mappata(fn)`       | `.map(fn)`          | no       |
| `filtrata(fn)`      | `.filter(fn)`       | no       |
| `reducta(fn, init)` | `.reduce(fn, init)` | no       |
| `inveni(fn)`        | `.find(fn)`         | no       |
| `continet(x)`       | `.includes(x)`      | no       |
| `coniunge(sep)`     | `.join(sep)`        | no       |

### Collection Methods (tabula)

| Latin        | JavaScript   | Mutates? |
| ------------ | ------------ | -------- |
| `pone(k, v)` | `.set(k, v)` | yes      |
| `accipe(k)`  | `.get(k)`    | no       |
| `habet(k)`   | `.has(k)`    | no       |
| `dele(k)`    | `.delete(k)` | yes      |
| `claves()`   | `.keys()`    | no       |
| `valores()`  | `.values()`  | no       |

### Collection Methods (copia)

| Latin                | JavaScript       | Mutates? |
| -------------------- | ---------------- | -------- |
| `adde(x)`            | `.add(x)`        | yes      |
| `habet(x)`           | `.has(x)`        | no       |
| `dele(x)`            | `.delete(x)`     | yes      |
| `unio(other)`        | set union        | no       |
| `intersectio(other)` | set intersection | no       |

---

## Complete Program Example

```fab
// A simple API handler demonstrating multiple features
ex hono importa Hono, Context

genus UserService {
    privatus textus baseUrl

    functio creo(textus url) {
        ego.baseUrl = url
    }

    futura functio fetch(numerus id) fiet User? {
        fixum response = cede ego.client.get(`${ego.baseUrl}/users/${id}`)

        custodi {
            si response.status !== 200 { redde nihil }
        }

        redde response.json() qua User
    }

    futura functio fetchAll() fiet lista<User> {
        fixum response = cede ego.client.get(`${ego.baseUrl}/users`)
        fixum users = cede response.json() qua User[]

        redde users.filtrata(pro u: u.active)
    }
}

fixum app = novum Hono()

app.get("/users/:id", futura functio(Context ctx) {
    fixum id = ctx.param("id") qua numerus
    fixum service = novum UserService("https://api.example.com")
    fixum user = cede service.fetch(id)

    si user === nihil {
        redde ctx.json({ error: "Not found" }, 404)
    }

    redde ctx.json(user)
})
```

---

## Table of Contents

- [Fundamenta](#fundamenta) — basic language elements
- [Typi](#typi) — type system
- [Operatores](#operatores) — operators
- [Functiones](#functiones) — function declarations
- [Regimen](#regimen) — control flow
- [Errores](#errores) — error handling
- [Structurae](#structurae) — data structures
- [Importa](#importa) — module system

---

<a id="fundamenta"></a>

## Fundamenta

Basic language elements: variables, constants, literals, and output.

### Object Pattern

```ebnf
objectPattern := '{' patternProperty (',' patternProperty)* '}'
patternProperty := 'ceteri'? IDENTIFIER (':' IDENTIFIER)?
```

**Examples:**

```fab
{ nomen, aetas }              // extract nomen and aetas
{ nomen: localName, aetas }   // rename nomen to localName
{ nomen, ceteri rest }        // extract nomen, collect rest

T SUPPORTED (will produce parser errors):
{ ...rest }    // JS spread syntax
{ *rest }      // Python unpack syntax
{ **rest }     // Python kwargs syntax
```

---

<a id="typi"></a>

## Typi

Type system: type annotations, aliases, enums, nullable types, and collections.

### Type Alias Declaration

```ebnf
typeAliasDecl := 'typus' IDENTIFIER '=' typeAnnotation
```

> Enables creating named type aliases for complex types.

**Examples:**

```fab
typus ID = textus
typus UserID = numerus<32, Naturalis>
typus ConfigTypus = typus config    // typeof
```

### Type Annotation

```ebnf
typeAnnotation := ('de' | 'in')? IDENTIFIER typeParams? '?'? arrayBrackets*
typeParams := '<' typeParameter (',' typeParameter)* '>'
typeParameter := typeAnnotation | NUMBER | MODIFIER
arrayBrackets := '[]' '?'?
```

> Supports generics (lista<textus>), nullable (?), union types (unio<A, B>),
> and array shorthand (numerus[] desugars to lista<numerus>).

---

<a id="operatores"></a>

## Operatores

Operators: arithmetic, logical, comparison, ternary, nullish coalescing, and ranges.

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
comparison := bitwiseOr (('<' | '>' | '<=' | '>=') bitwiseOr)*
```

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
unary := ('!' | '-' | 'non' | 'nulla' | 'nonnulla' | 'nihil' | 'nonnihil' | 'negativum' | 'positivum' | 'cede' | 'novum') unary | cast
```

> Latin 'non' (not), 'nulla' (none/empty), 'nonnulla' (some/non-empty),
> 'nihil' (is null), 'nonnihil' (is not null),
> 'negativum' (< 0), 'positivum' (> 0), 'cede' (await), 'novum' (new).

---

<a id="functiones"></a>

## Functiones

Function declarations: basic functions, typed parameters, async, generators, and lambdas.

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

### Arrow Function

```ebnf
arrowFunction := '(' paramList ')' '=>' (expression | blockStmt)
```

> Called after detecting '() =>' pattern in parsePrimary.

---

<a id="regimen"></a>

## Regimen

Control flow: conditionals, loops, guards, assertions, and program structure.

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

### Importa Declaration

```ebnf
importDecl := 'ex' (STRING | IDENTIFIER) 'importa' (identifierList | '*')
identifierList := IDENTIFIER (',' IDENTIFIER)*
```

**Examples:**

```fab
ex norma importa scribe, lege
ex "norma/tempus" importa nunc, dormi
ex norma importa *
```

### Varia Declaration

```ebnf
varDecl := ('varia' | 'fixum') (objectPattern '=' expression | typeAnnotation IDENTIFIER | IDENTIFIER) ('=' expression)?
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
[a, b, c]                 // extract first three elements
[first, ceteri rest]     // extract first, collect rest
[_, second, _]           // skip first and third, extract second

T SUPPORTED:
[...rest]                // JS spread syntax
[*rest]                  // Python unpack syntax
```

### Functio Declaration

```ebnf
funcDecl := ('futura' | 'cursor')* 'functio' IDENTIFIER '(' paramList ')' returnClause? blockStmt
paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
typeParamDecl := 'prae' 'typus' IDENTIFIER
returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
```

> Arrow syntax for return types: "functio greet(textus name) -> textus"
> 'futura' prefix marks async functions (future/promise-based).
> 'cursor' prefix marks generator functions (yield-based).
> 
> TYPE PARAMETERS: 'prae typus T' declares compile-time type parameters.
> functio max(prae typus T, T a, T b) -> T { ... }
> Maps to: <T> (TS/Rust), TypeVar (Py), comptime T: type (Zig)
> 
> RETURN TYPE VERBS: Latin verb forms encode async/generator semantics directly:
> '->'    neutral arrow (semantics from prefix only)
> 'fit'   "it becomes" - sync, returns single value
> 'fiet'  "it will become" - async, returns Promise<T>
> 'fiunt' "they become" - sync generator, yields multiple values
> 'fient' "they will become" - async generator, yields Promise values
> 
> When using verb forms, the futura/cursor prefix is NOT required - the verb
> itself carries the semantic information. The prefix becomes redundant:
> functio compute() -> numerus { ... }    // arrow: sync by default
> functio compute() fit numerus { ... }   // verb: explicitly sync
> functio fetch() fiet textus { ... }     // verb implies async (no futura needed)
> functio items() fiunt numerus { ... }   // verb implies generator (no cursor needed)
> functio stream() fient datum { ... }    // verb implies async generator
> 
> Prefix is still allowed for emphasis, but verb/prefix conflicts are errors.
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
enumMember := IDENTIFIER ('=' (NUMBER | STRING))?
```

> Latin 'ordo' (order/rank) for enumerated constants.

**Examples:**

```fab
ordo color { rubrum, viridis, caeruleum }
ordo status { pendens = 0, actum = 1, finitum = 2 }
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
elseClause := ('aliter' | 'secus') (ifStmt | blockStmt | statement)
```

> 'cape' (catch/seize) clause allows error handling within conditionals.
> 'ergo' (therefore) for one-liner consequents.
> 
> TWO STYLE OPTIONS (both supported, can be mixed within the same chain):
> 
> Literal style: si / aliter si / aliter
> si x > 0 { positive() }
> aliter si x < 0 { negative() }
> aliter { zero() }
> 
> Poetic style: si / sin / secus
> si x > 0 { positive() }
> sin x < 0 { negative() }    // "sin" = "but if" (classical Latin)
> secus { zero() }            // "secus" = "otherwise"
> 
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
exStmt := 'ex' expression (forBinding | destructBinding)
forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement) catchClause?
destructBinding := ('fixum' | 'varia' | 'figendum' | 'variandum') objectPattern
```

> 'ex' (from/out of) introduces both iteration and extraction:
> - Iteration: ex items pro item { ... } (for each item from items)
> - Destructuring: ex response fixum { data } (extract data from response)
> - Async destructuring: ex promise figendum { result } (await + extract)
> 
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

### Elige Statement

```ebnf
eligeStmt := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
eligeCase := 'si' expression (blockStmt | 'ergo' expression)
defaultCase := ('aliter' | 'secus') (blockStmt | statement)
```

> 'elige' (choose) for value-based switch.
> 'ergo' (therefore) for one-liners, 'aliter'/'secus' (otherwise) for default.
> For variant matching on discretio types, use 'discerne' instead.

**Examples:**

```fab
elige status {
    si "pending" ergo scribe("waiting")
    si "active" { processActive() }
    aliter iace "Unknown status"
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

> Latin "cura" (care, concern) for test resource management.
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

### Cura Statement

```ebnf
curaStmt := 'cura' 'cede'? expression 'fit' IDENTIFIER blockStmt catchClause?
```

> Latin "cura" (care) + "fit" (it becomes) for scoped resources.
> Reads as: "Care for [resource] as [name] { use it }"
> Guarantees cleanup via solve() on scope exit.

**Examples:**

```fab
cura aperi("data.bin") fit fd { lege(fd) }
cura cede connect(url) fit conn { cede conn.query(sql) }
cura mutex.lock() fit guard { counter += 1 } cape err { mone(err) }
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
lambdaExpr := ('pro' | 'fit' | 'fiet') params? ((':' | 'redde') expression | blockStmt)
params := IDENTIFIER (',' IDENTIFIER)*
```

---

<a id="errores"></a>

## Errores

Error handling: try/catch, throw, panic, and scoped error handling.

### Fac Block Statement

```ebnf
facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)?
```

> 'fac' (do/make) creates an explicit scope boundary for grouping
> statements with optional error handling via 'cape' (catch).
> Useful for:
> - Scoped variable declarations
> - Grouping related operations with shared error handling
> - Creating IIFE-like constructs

**Examples:**

```fab
fac { fixum x = computeValue() }
fac { riskyOperation() } cape e { scribe e }
```

---

<a id="structurae"></a>

## Structurae

Data structures: classes (genus), objects, member access, and instantiation.

### Genus Declaration

```ebnf
genusDecl := 'genus' IDENTIFIER typeParams? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
genusMember := fieldDecl | methodDecl
```

> Latin 'genus' (kind/type) for data structures.
> 'implet' (fulfills) for implementing pactum interfaces.

### Genus Member

```ebnf
genusMember := fieldDecl | methodDecl
fieldDecl := 'privatus'? 'generis'? typeAnnotation IDENTIFIER (':' expression)?
methodDecl := 'privatus'? 'generis'? ('futura' | 'cursor')* 'functio' ...
```

> Distinguishes between fields and methods by looking for 'functio' keyword.
> Fields are public by default (struct semantics), use 'privatus' for private.

### Pactum Declaration

```ebnf
pactumDecl := 'pactum' IDENTIFIER typeParams? '{' pactumMethod* '}'
typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
```

> Latin 'pactum' (agreement/contract) for interfaces.
> Defines method signatures that genus types can implement via 'implet'.

**Examples:**

```fab
pactum Legibilis { functio lege() -> textus }
pactum Mappabilis<T, U> { functio mappa(T valor) -> U }
```

### Pactum Method

```ebnf
pactumMethod := ('futura' | 'cursor')* 'functio' IDENTIFIER '(' paramList ')' returnClause?
returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
```

> Method signatures without bodies. Same syntax as function declarations
> but terminates after return type (no block).

### Call

```ebnf
call := primary (callSuffix | memberSuffix | optionalSuffix | nonNullSuffix)*
callSuffix := '(' argumentList ')'
memberSuffix := '.' IDENTIFIER | '[' expression ']'
optionalSuffix := '?.' IDENTIFIER | '?[' expression ']' | '?(' argumentList ')'
nonNullSuffix := '!.' IDENTIFIER | '![' expression ']' | '!(' argumentList ')'
```

> Handles function calls, member access, and computed member access.
> Left-associative via loop (obj.a.b parsed as (obj.a).b).
> 
> OPTIONAL CHAINING: ?. ?[ ?( return nihil if object is nihil
> NON-NULL ASSERTION: !. ![ !( assert object is not nihil

### Argument List

```ebnf
argumentList := (argument (',' argument)*)?
argument := 'sparge' expression | expression
```

### Primary

```ebnf
primary := IDENTIFIER | NUMBER | STRING | TEMPLATE_STRING
| 'ego' | 'verum' | 'falsum' | 'nihil'
| '(' (expression | arrowFunction) ')'
```

> Latin literals: verum (true), falsum (false), nihil (null).
> 'ego' (I/self) is the self-reference keyword (like 'this' in JS).
> Parenthesized expressions require lookahead to distinguish from arrow functions.

### Identifier

```ebnf
identifier := IDENTIFIER
```

---

*Generated from `fons/parser/index.ts` — do not edit directly.*