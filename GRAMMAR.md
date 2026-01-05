# Faber Romanus Grammar

Complete syntax reference for the Faber Romanus programming language.

## For LLMs

This document is designed for both human readers and LLM code generation. When generating Faber code:

**Style preferences:**

- PREFER Latin keywords over symbols: `et` over `&&`, `aut` over `||`, `non` over `!`
- PREFER `pro x: expr` for short lambdas, `pro x redde expr` when clarity helps
- ALWAYS use type-first syntax: `textus nomen` not `nomen: textus`
- NEVER use JavaScript/TypeScript/Python syntax where Faber has its own

**Common mistakes to avoid:**

- `return` instead of `redde`
- `const`/`let` instead of `fixum`/`varia`
- `if`/`else` instead of `si`/`secus`
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

- `si` / `sin` / `secus` / `secus` — if / else-if / else
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
# A simple API handler demonstrating multiple features
ex hono importa Hono, Context

genus UserService {
    @ privatum
    textus baseUrl

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
    fixum service = novum UserService("https:#api.example.com")
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
comparison := bitwiseOr (('<' | '>' | '<=' | '>=' | 'intra' | 'inter') bitwiseOr)*
```

> intra/inter at comparison level - same precedence as relational operators

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
unary := ('!' | '-' | 'non' | 'nulla' | 'nonnulla' | 'nihil' | 'nonnihil' | 'negativum' | 'positivum' | 'cede' | 'novum' | 'finge') unary | cast
```

> Latin 'non' (not), 'nulla' (none/empty), 'nonnulla' (some/non-empty),
> 'nihil' (is null), 'nonnihil' (is not null),
> 'negativum' (< 0), 'positivum' (> 0), 'cede' (await), 'novum' (new),
> 'finge' (form variant).

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

<a id="regimen"></a>

## Regimen

Control flow: conditionals, loops, guards, assertions, and program structure.

### Annotation

```ebnf
annotation := '@' IDENTIFIER (expression)?
```

### Program

```ebnf
program := statement*
```

### Annotations

```ebnf
annotation := '@' IDENTIFIER+
```

> Annotations modify the following declaration with metadata like
> visibility (publicum, privatum), async (futura), abstract (abstractum).

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
[a, b, c]                 // extract first three elements
[first, ceteri rest]     // extract first, collect rest
[_, second, _]           // skip first and third, extract second

T SUPPORTED:
[...rest]                // JS spread syntax
[*rest]                  // Python unpack syntax
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
ifStmt := 'si' expression (blockStmt | 'ergo' statement | 'reddit' expression) ('cape' IDENTIFIER blockStmt)? (elseClause | 'sin' ifStmt)?
elseClause := ('secus' | 'secus') (ifStmt | blockStmt | statement)
```

> 'cape' (catch/seize) clause allows error handling within conditionals.
> 'ergo' (therefore) for one-liner consequents.
> 'reddit' (it returns) for early return one-liners.
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
si x > 5 reddit verum            // early return
si x > 5 { scribe("big") } secus scribe("small")
si x < 0 { ... } sin x == 0 { ... } secus { ... }
```

### Dum Statement

```ebnf
whileStmt := 'dum' expression (blockStmt | 'ergo' statement | 'reddit' expression) ('cape' IDENTIFIER blockStmt)?
```

> 'dum' (while/until) for while loops.

**Examples:**

```fab
dum x > 0 { x = x - 1 }
dum x > 0 ergo x = x - 1
dum x > 0 reddit x
```

### Ex Statement

```ebnf
exStmt := 'ex' expression (forBinding | destructBinding | arrayDestructBinding)
forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement | 'reddit' expression) catchClause?
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
ex numeri pro n { ... }              // for-loop (sync)
ex numeri fiet n { ... }             // for-await-of loop (async)
ex persona fixum nomen, aetas        // object destructuring
ex persona fixum nomen ut n          // object destructuring with alias
ex persona fixum nomen, ceteri rest  // object destructuring with rest
ex coords fixum [x, y, z]            // array destructuring
ex fetchData() figendum result       // async destructuring

llection DSL forms:
ex items prima 5 pro item { }        // iteration with transforms
ex items prima 5, ultima 2 pro x {}  // multiple transforms
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
ab users activus                     // boolean property shorthand
ab users non banned                  // negated boolean property
ab users ubi aetas >= 18             // condition with ubi
ab users non ubi banned et suspended // negated compound condition
ab users activus, prima 10           // filter + transforms
ab users activus pro user { }        // iteration form
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
sed "\\d+"           // pattern only
sed "hello" i        // case insensitive
sed "^start" im      // multiple flags
```

### De Statement

```ebnf
deStmt := 'de' expression ('pro' | 'fit' | 'fiet') IDENTIFIER
(blockStmt | 'ergo' statement | 'reddit' expression) catchClause?
```

> 'de' (from/concerning) for extracting keys from an object.
> Semantically read-only - contrasts with 'in' for mutation.

**Examples:**

```fab
de tabula pro clavis { ... }  // from table, for each key
de object pro k ergo scribe k // one-liner form
de object pro k reddit k      // return first key
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
eligeCase := 'casu' expression (blockStmt | 'ergo' statement | 'reddit' expression)
defaultCase := 'ceterum' (blockStmt | statement)
```

> 'elige' (choose) for value-based switch.
> 'ergo' (therefore) for one-liners, 'ceterum' (otherwise) for default.
> 'reddit' (it returns) for early return one-liners.
> For variant matching on discretio types, use 'discerne' instead.

**Examples:**

```fab
elige status {
    casu "pending" ergo scribe("waiting")
    casu "active" reddit verum
    ceterum iace "Unknown status"
}
```

### Discerne Statement

```ebnf
discerneStmt := 'discerne' expression '{' variantCase* '}'
variantCase := 'casu' IDENTIFIER (('ut' IDENTIFIER) | ('pro' IDENTIFIER (',' IDENTIFIER)*))? (blockStmt | 'ergo' statement | 'reddit' expression)
```

> 'discerne' (distinguish!) pairs with 'discretio' (the tagged union type).
> Uses 'casu' for match arms, 'ut' to bind whole variants, and 'pro' to introduce positional bindings.
> 'ergo' for one-liners, 'reddit' for early return one-liners.

**Examples:**

```fab
discerne event {
    casu Click pro x, y { scribe "clicked at " + x + ", " + y }
    casu Keypress pro key reddit key
    casu Quit ergo mori "goodbye"
}
```

### Custodi Statement

```ebnf
guardStmt := 'custodi' '{' guardClause+ '}'
guardClause := 'si' expression (blockStmt | 'ergo' statement | 'reddit' expression)
```

> 'custodi' (guard!) groups early-exit conditions.
> 'ergo' for one-liner actions, 'reddit' for early return one-liners.

**Examples:**

```fab
custodi {
    si user == nihil reddit nihil
    si user.age < 0 ergo iace "Invalid age"
    si user.name == "" { redde defaultUser() }
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
ad "console:log" ("hello")                           // fire-and-forget
ad "fasciculus:lege" ("file.txt") fit textus pro c { }  // sync binding
ad "http:get" (url) fiet Response pro r { }          // async binding
ad "http:batch" (urls) fient Response[] pro rs { }   // async plural
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
cura arena fit mem { ... }                    // arena allocator
cura page fit mem { ... }                     // page allocator
cura aperi("data.bin") fit fd { lege(fd) }   // generic resource
cura connect(url) fiet conn { ... }          // async resource
```

### Incipit Statement

```ebnf
incipitStmt := 'incipit' (blockStmt | 'ergo' statement | 'reddit' expression)
```

> 'incipit' (it begins) marks the program entry point.
> This is a pure structural marker with no magic injection.
> The source is responsible for any setup (allocators via cura, etc.).
> 
> The 'ergo' (therefore) form chains to a single statement, typically
> a cura block for allocator setup. This avoids extra nesting.
> The 'reddit' form returns an exit code directly.

**Examples:**

```fab
incipit {
    scribe "Hello"
}

incipit ergo cura arena {
```

### Incipiet Statement

```ebnf
incipietStmt := 'incipiet' (blockStmt | 'ergo' statement | 'reddit' expression)
```

> 'incipiet' (it will begin) marks the async program entry point.
> Mirrors the fit/fiet pattern: present for sync, future for async.
> 
> The 'ergo' form chains to a single statement for concise setup.
> The 'reddit' form returns an exit code directly.

**Examples:**

```fab
incipiet {
    fixum data = cede fetchData()
    scribe data
}

incipiet ergo cura arena {
    fixum data = cede fetchData()
}

incipiet reddit 0
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
> The § placeholder is converted to target-appropriate format specifiers.

**Examples:**

```fab
scriptum("Hello, §", name)
scriptum("§ + § = §", a, b, a + b)
```

### Lege Expression

```ebnf
legeExpr := 'lege' ('lineam')?
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

### Finge Expression

```ebnf
fingeExpr := 'finge' IDENTIFIER ('{' fieldList '}')? ('qua' IDENTIFIER)?
```

> Latin 'finge' (form/shape) for constructing discretio variants.
> Variant name comes first, optional fields in braces, optional qua for
> explicit discretio type when not inferrable from context.

**Examples:**

```fab
finge Click { x: 10, y: 20 }           - payload variant
finge Click { x: 10, y: 20 } qua Event - with explicit type
finge Active                            - unit variant
finge Active qua Status                 - unit variant with explicit type
```

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

<a id="errores"></a>

## Errores

Error handling: try/catch, throw, panic, and scoped error handling.

### Fac Block Statement

```ebnf
facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)? ('dum' expression)?
```

> 'fac' (do/make) creates an explicit scope boundary for grouping
> statements with optional error handling via 'cape' (catch).
> When followed by 'dum', creates a do-while loop where the body
> executes at least once before the condition is checked.
> Useful for:
> - Scoped variable declarations
> - Grouping related operations with shared error handling
> - Creating IIFE-like constructs
> - Do-while loops (body executes first, then condition checked)

**Examples:**

```fab
fac { fixum x = computeValue() }
fac { riskyOperation() } cape e { scribe e }
fac { process() } dum hasMore()
fac { process() } cape e { log(e) } dum hasMore()
```

---

<a id="structurae"></a>

## Structurae

Data structures: classes (genus), objects, member access, and instantiation.

### Genus Declaration

```ebnf
genusDecl := 'abstractus'? 'genus' IDENTIFIER typeParams? ('sub' IDENTIFIER)? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
genusMember := fieldDecl | methodDecl
```

> Latin 'genus' (kind/type) for data structures.
> 'sub' (under) for inheritance - child is under parent.
> 'implet' (fulfills) for implementing pactum interfaces.
> 'abstractus' for abstract classes that cannot be instantiated.

### Genus Member

```ebnf
genusMember := annotation* (fieldDecl | methodDecl)
annotation := '@' IDENTIFIER+
fieldDecl := 'generis'? 'nexum'? typeAnnotation IDENTIFIER (':' expression)?
methodDecl := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt?
funcModifier := 'futura' | 'cursor' | 'curata' IDENTIFIER
```

> Distinguishes between fields and methods by looking for 'functio' keyword.
> Fields are public by default (struct semantics).
> Use annotations for visibility: @ privatum, @ protectum.
> Use annotations for abstract methods: @ abstracta (no body, must be overridden).

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
pactumMethod := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause?
funcModifier := 'futura' | 'cursor' | 'curata' IDENTIFIER
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

*Generated from `fons/faber/parser/index.ts` — do not edit directly.*