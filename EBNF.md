# Faber Language Specification

Formal grammar for the Faber programming language. The active implementation is the root Rust workspace: `crates/faber` for package/project tooling and `crates/radix` for the compiler pipeline.

Documentation contract: this file is the canonical grammar and spec-commentary surface. The embedded `explain/` corpus is the canonical user-facing reference used by `faber explain`. Do not recreate a separate prose grammar surface in `docs/`.

---

## Program Structure

```ebnf
program     := statement*
statement   := directiveDecl | importDecl | varDecl | funcDecl | genusDecl | pactumDecl
             | typeAliasDecl | enumDecl | discretioDecl
             | ifStmt | whileStmt | iteraStmt
             | eligeStmt | discerneStmt | guardStmt | curaStmt | facBlockStmt
             | returnStmt | breakStmt | continueStmt | noopStmt | throwStmt
             | assertStmt | outputStmt | adStmt | incipitStmt
             | incipietStmt | extractStmt
             | probandumDecl | probaStmt | blockStmt | exprStmt
blockStmt   := '{' statement* '}'
```

---

## Declarations

### Variables

```ebnf
varDecl      := ('fixum' | 'varia') typeAnnotation IDENTIFIER ('←' expression)?
arrayDestruct := ('fixum' | 'varia') arrayPattern '←' expression
objectDestruct := ('fixum' | 'varia') objectPattern '←' expression
```

- `fixum` = const, `varia` = let
- Use `_` as the type annotation when the initializer determines the type: `fixum _ name ← value`

### Functions

```ebnf
funcDecl     := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? alternateExitClause? blockStmt?
paramList    := (typeParamDecl ',')* (parameter (',' parameter)*)?
typeParamDecl := 'prae' 'typus' IDENTIFIER
parameter    := ('de' | 'in' | 'ex')? 'ceteri'? typeAnnotation IDENTIFIER ('sponte' 'fixus'? | 'fixus')? ('ut' IDENTIFIER)? ('vel' expression)?
funcModifier := 'argumenta' IDENTIFIER | 'curata' IDENTIFIER ('ut' IDENTIFIER)? | 'errata' IDENTIFIER | 'exitus' (IDENTIFIER | NUMBER) | 'immutata' | 'iacit' | 'optiones' IDENTIFIER
returnClause := '→' typeAnnotation
alternateExitClause := '⇥' typeAnnotation
ergoToken      := '∴' | 'ergo'
clausuraExpr   := compactClausuraExpr | legacyClausuraExpr
compactClausuraExpr := clausuraSignature ergoToken (expression | closureFacBlock)
clausuraSignature := (clausuraParam | '(' clausuraParams? ')') returnClause? alternateExitClause?
closureFacBlock := 'fac' blockStmt catchClause?
legacyClausuraExpr := 'clausura' clausuraParams? ('→' typeAnnotation)? (':' expression | blockStmt)
clausuraParams := clausuraParam (',' clausuraParam)*
clausuraParam  := typeAnnotation IDENTIFIER
```

- Return syntax: `→` declares the normal success type. A bodyful function with no `→` is effect-only (`vacuum`) and must not contain `redde`. A statement-bodied closure (`fac { ... }` or legacy block body) must also spell `→ T` before it can use `redde`; expression-bodied closures may infer their result from the expression.
- Recoverable alternate-exit syntax: `⇥` declares the error-channel type. It can appear after `→ T` or alone on an effect-only failable function or closure. A closure body that uses an escaping `iace` must declare its own `⇥ E`; it cannot inherit the enclosing function's error channel. A local `fac { ... } cape err { ... }` may catch `iace` without an enclosing `⇥`.
- Parameter prefixes: `de` (read), `in` (mutate), `ex` (consume)
- Post-name markers: `sponte` (voluntary/optional provision), `fixus` (fixed after first assignment); canonical order `sponte fixus`
- `ceteri` marks rest parameter
- `curata NAME ('ut' LOCAL)?` declares a Zig allocator requirement; `LOCAL` is the function-body alias
- `∴` is the canonical compact therefore marker; Latin `ergo` is accepted with the same semantics.
- Compact closure block bodies must use `fac { ... }`; a closure-local `fac` body may attach `cape`, but cannot use postfix `dum`.

### Classes

```ebnf
genusDecl    := 'abstractus'? 'genus' IDENTIFIER typeParams? ('sub' IDENTIFIER)? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
genusMember  := annotation* (fieldDecl | methodDecl)
fieldDecl    := 'generis'? 'nexum'? typeAnnotation IDENTIFIER ('sponte' 'fixus'? | 'fixus')? ('=' expression)?
methodDecl   := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? alternateExitClause? blockStmt?
annotation   := '@' IDENTIFIER+ | stdlibAnnotation
```

### Stdlib Annotations

```ebnf
annotation       := '@' IDENTIFIER+ | stdlibAnnotation | cliAnnotation
stdlibAnnotation := innatumAnnotation | subsidiaAnnotation | radixAnnotation | verteAnnotation | externaAnnotation
cliAnnotation    := cliProgramAnnotation | imperiumAnnotation | optioAnnotation | operandusAnnotation | cliAnnotationSimple

innatumAnnotation  := '@' 'innatum' targetMapping (',' targetMapping)*
subsidiaAnnotation := '@' 'subsidia' targetMapping (',' targetMapping)*
radixAnnotation    := '@' 'radix' IDENTIFIER (',' IDENTIFIER)*
verteAnnotation    := '@' 'verte' IDENTIFIER (STRING | '(' IDENTIFIER (',' IDENTIFIER)* ')' '→' STRING)
externaAnnotation  := '@' 'externa'

cliProgramAnnotation := '@' 'cli' STRING
imperiumAnnotation := '@' 'imperium' STRING
optioAnnotation    := '@' 'optio' IDENTIFIER optioModifier*
optioModifier      := 'brevis' STRING | 'longum' STRING | 'typus' typeAnnotation
                    | 'descriptio' STRING | 'ubique' | 'vel' expression
operandusAnnotation := '@' 'operandus' ('ceteri')? typeAnnotation IDENTIFIER operandusModifier*
operandusModifier  := 'descriptio' STRING | 'ubique' | 'vel' expression
cliAnnotationSimple := '@' 'futura' | '@' 'cursor' | '@' 'tag' | '@' 'solum' | '@' 'omitte'
                      | '@' 'metior' | '@' 'publica' | '@' 'protecta' | '@' 'privata'

targetMapping := IDENTIFIER STRING
```

- `@ innatum` maps a genus to native types per target
- `@ subsidia` specifies external implementation files
- `@ radix` declares morphological stem and valid verb forms
- `@ verte` defines codegen transformation (method name or template)
- `@ externa` marks declarations as externally provided (no initializer/body required)
- `@ cli "NAME"` marks an `incipit` entry as a CLI program
- `@ imperium "NAME"` marks a function as a CLI command entry point
- `@ optio NAME ...` defines a CLI option; use `typus bivalens` for boolean flags
- `@ operandus [ceteri] TYPE NAME ...` defines a CLI positional argument
- `@ futura` marks a function as async
- `@ cursor` marks a function as generator
- `@ publica`, `@ protecta`, and `@ privata` are still parsed as annotations, but the active Radix compiler contract does not treat them as a stable genus member-visibility model

- `sub` = extends, `implet` = implements
- `generis` = static, `nexum` = bound/property

### Interfaces

```ebnf
pactumDecl   := 'pactum' IDENTIFIER typeParams? '{' pactumMethod* '}'
pactumMethod := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? alternateExitClause?
```

### Type Aliases

```ebnf
typeAliasDecl := 'typus' IDENTIFIER '=' typeAnnotation
```

### Enums

```ebnf
enumDecl   := 'ordo' IDENTIFIER '{' enumMember (',' enumMember)* ','? '}'
enumMember := IDENTIFIER ('=' ('-'? NUMBER | STRING))?
```

### Tagged Unions

```ebnf
discretioDecl := 'discretio' IDENTIFIER typeParams? '{' variant (',' variant)* ','? '}'
variant       := IDENTIFIER ('{' variantFields '}')?
variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
```

### File-Level Directives

```ebnf
directiveDecl  := '§' directiveName directiveArgs?
directiveName  := IDENTIFIER
directiveArgs  := (STRING | IDENTIFIER)+
```

Common directives:

- `§ dependentia` - declare module dependencies
- `§ externa` - declare external packages
- `§ opus` - project configuration (name, version, target)
- `§ scopos` - compilation target
- `§ modulus` - module identity

Example:

```fab
§ dependentia "hono" github "honojs/hono#main" via "."
§ externa "lodash" ex "@types/lodash"
§ opus nomen "myproject"
§ scopos "ts"
```

Common directives:

- `§ dependentia` - declare module dependencies
- `§ externa` - mark external declarations
- `§ radix` - morphological verb forms
- `§ verte` - codegen transformations
- `§ innatum` - native type mappings

Example:

```fab
§ dependentia "hono" github "honojs/hono#main" via "."
§ externa fixum textus VERSION
§ radix adde imperativus perfectum
```

### Imports

```ebnf
importDecl    := 'importa' 'ex' STRING visibility (namedImport | wildcardImport)
visibility    := 'privata' | 'publica'
namedImport   := IDENTIFIER ('ut' IDENTIFIER)?
wildcardImport := '*' 'ut' IDENTIFIER
```

Example:

```fab
importa ex "hono" privata Hono
importa ex "hono" privata Context
importa ex "norma:hal/consolum" privata consolum
importa ex "lodash" privata * ut _
importa ex "./types" publica User               # re-export
```

---

## Types

```ebnf
typeAnnotation := ('de' | 'in')? baseType ( '∪' typeAnnotation )*
baseType       := '_' arrayBrackets* | functionType | IDENTIFIER typeParams? arrayBrackets* | '(' typeAnnotation ')'
functionType   := '(' typeList? ')' '→' typeAnnotation alternateExitClause?
typeList       := typeAnnotation (',' typeAnnotation)*
typeParams     := '<' typeParameter (',' typeParameter)* '>'
typeParameter  := typeAnnotation | NUMBER | MODIFIER
arrayBrackets  := '[]'
```

- `de`/`in` mark ownership (borrow/mut-borrow) as prefixes on the type.
- Inline union `T ∪ U` (cup) for ad-hoc value unions; `T ∪ nihil` is the canonical nullable type form (lowers to Option<T>).
- Unions are right-associative in the grammar but parsed flat; duplicates and `nihil`-only cases are diagnosed in semantic lowering.
- `sponte` and `fixus` are declaration markers (post-name on params/fields), never prefixes on types.

Function types enable higher-order function signatures:

```fab
functio filtrata((T) → bivalens pred) → lista<T>
functio compose((A) → B f, (B) → C g) → (A) → C
functio apply((numerus) → numerus ⇥ textus op, numerus n) → numerus ⇥ textus
```

### Primitive Types

| Faber      | Meaning |
| ---------- | ------- |
| `textus`   | string  |
| `numerus`  | integer |
| `fractus`  | float   |
| `bivalens` | boolean |
| `nihil`    | null    |
| `vacuum`   | void    |
| `numquam`  | never   |
| `ignotum`  | unknown |
| `octeti`   | bytes   |

### Generic Collections

| Faber          | Meaning  |
| -------------- | -------- |
| `lista<T>`     | array    |
| `tabula<K,V>`  | map      |
| `copia<T>`     | set      |
| `promissum<T>` | promise  |
| `cursor<T>`    | iterator |
| `unio<A,B>`    | union    |

---

## Control Flow

### Conditionals

```ebnf
ifStmt     := 'si' expression arm ('sin' ifStmt | elseClause)?
elseClause := 'secus' elseArm
arm        := (blockStmt | ergoToken statement) catchClause?
elseArm    := (blockStmt | ergoToken statement) catchClause?
```

- `si` = if, `sin` = else-if, `secus` = else
- `∴`/`ergo` for one-statement bodies, including `∴ redde`, `∴ iace`, `∴ mori`, and `∴ tacet`
- `tacet` for explicit no-op (from musical notation: "it is silent")

### Loops

```ebnf
whileStmt  := 'dum' expression (blockStmt | ergoToken statement) catchClause?
iteraStmt  := 'itera' (('ex' | 'de') expression | 'ab' expression) ('fixum' | 'varia') IDENTIFIER (blockStmt | ergoToken statement) catchClause?
```

- `dum` = while
- `itera ex...fixum`/`itera ex...varia` = for-of (values)
- `itera de...fixum`/`itera de...varia` = for-in (keys)
- `itera ab range fixum/varia i` = range iteration (e.g. `itera ab 0‥10 per 2 fixum i { nota i }`; `per` belongs to the range expression)

### Switch/Match

```ebnf
eligeStmt    := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
eligeCase    := 'casu' expression (blockStmt | ergoToken statement)
defaultCase  := 'ceterum' (blockStmt | ergoToken statement)
```

### Pattern Matching

```ebnf
discerneStmt := 'discerne' 'omnia'? discriminants '{' variantCase* defaultCase? '}'
discriminants := expression (',' expression)*
variantCase  := 'casu' patterns (blockStmt | ergoToken statement)
patterns     := pattern ((',' | 'et') pattern)*
pattern      := '_' | literal | (IDENTIFIER patternBind?)
patternBind  := ('ut' IDENTIFIER) | (('fixum' | 'varia') IDENTIFIER (',' IDENTIFIER)*)
```

### Guards

```ebnf
guardStmt   := 'custodi' '{' guardClause+ '}'
guardClause := 'si' expression (blockStmt | ergoToken statement)
```

### Resource Management

```ebnf
curaStmt    := 'cura' STRING ('fixum' | 'varia') typeAnnotation IDENTIFIER blockStmt catchClause?
```

### Destructuring Extraction

```ebnf
extractStmt   := 'ex' expression ('fixum' | 'varia') extractFields
extractFields := extractField (',' extractField)* (',' restField)? | restField
extractField  := IDENTIFIER ('ut' IDENTIFIER)?
restField     := 'ceteri' IDENTIFIER
```

### Control Transfer

```ebnf
returnStmt   := 'redde' expression?
breakStmt    := 'rumpe'
continueStmt := 'perge'
noopStmt     := 'tacet'
```

---

## Error Handling

```ebnf
throwStmt   := ('iace' | 'mori') expression
catchClause := 'cape' IDENTIFIER blockStmt
assertStmt  := 'adfirma' expression (',' expression)?
```

- `cape` attaches to structured statements and conditional arms. It does not attach to arbitrary bare blocks.
- `fac { ... } cape err { ... }` is the canonical one-shot local recoverable-error boundary.
- `tempta` is a legacy try/catch surface and is rejected with a migration diagnostic.
- `demum` cleanup/finally semantics are deferred until resource cleanup rules are designed.
- `iace` = throw (recoverable), `mori` = panic (fatal)

---

## Expressions

### Operators (by precedence, lowest to highest)

```ebnf
expression := assignment
assignment := ternary (('←' | '⊕' | '⊖' | '⊛' | '⊘' | '⊜' | '⊚') assignment)?
ternary    := or (('?' expression ':' | 'sic' expression 'secus') ternary)?
or         := and (('aut') and)* | and ('vel' and)*
and        := equality (('et') equality)*
equality   := comparison (('≡' | '≠' | 'est' | 'non' 'est') comparison)*
comparison := bitwiseOr (('<' | '>' | '≤' | '≥' | 'intra' | 'inter') bitwiseOr)*
bitwiseOr  := bitwiseXor ('∨' bitwiseXor)*
bitwiseXor := bitwiseAnd ('⊻' bitwiseAnd)*
bitwiseAnd := shift ('∧' shift)*
shift      := range (('≪' | '≫') range)*
range      := additive (('‥' | '…' | 'ante' | 'usque') additive ('per' additive)?)?
additive   := multiplicative (('+' | '-') multiplicative)*
multiplicative := unary (('*' | '/' | '%') unary)*
unary      := ('-' | '¬' | 'non' | 'nulla' | 'nonnulla' | 'nihil' | 'nonnihil' | 'negativum' | 'positivum' | 'cede' | 'finge') unary | cast
cast       := call ('∷' typeAnnotation | conversio)*
conversio  := '⇒' typeAnnotation typeParams? ('vel' unary)?
```

**Static type ascription (`∷` / verte):**

The `∷` glyph (U+2237, "proportion") explicitly ascribes a target type to an expression. Use it when the source expression already exists and the compiler needs a static target shape:

- Primitive/alias → cast (no runtime effect): `data ∷ textus` → TypeScript: `(data as string)`
- Built-in collection → target-shaped collection value: `[1, 2, 3] ∷ lista<numerus>`
- Variant expression → enum/interface target ascription: `finge Click { x = 10 } ∷ Event`

Prefer typed construction for ordinary `genus` values and `vacua` for ordinary empty collection values:

```fab
fixum _ point ← Point { x = 10 }
fixum lista<numerus> xs ← vacua
```

Only the `∷` glyph is accepted as the postfix static type-ascription operator. The Latin forms `qua`, `innatum`, and `novum` were aliases and have been removed (see verte-alias-clean-break).

**Runtime conversion (`⇒` / conversio):**

The `⇒` glyph (U+21D2, "rightwards double arrow") is the runtime value conversion operator. Unlike `∷` (compile-time cast), this performs actual parsing/conversion that can fail:

- `"22" ⇒ numerus` → Rust: `"22".parse::<i64>().unwrap()`
- `"bad" ⇒ numerus vel 0` → Rust: `"bad".parse::<i64>().unwrap_or(0)`
- `42 ⇒ textus` → Rust: `42.to_string()`

### Call and Member Access

```ebnf
call          := primary (callSuffix | memberSuffix | optionalSuffix | nonNullSuffix)*
callSuffix    := '(' argumentList ')'
memberSuffix  := '.' IDENTIFIER | '[' expression ']'
optionalSuffix := '?.' IDENTIFIER | '?[' expression ']' | '?(' argumentList ')'
nonNullSuffix := '!.' IDENTIFIER | '![' expression ']' | '!(' argumentList ')'
argumentList  := (argument (',' argument)*)?
argument      := 'sparge'? expression
```

String literal call syntax is the canonical source form for format-template application:

```fab
"status: § (§)"(sample_status(), "ok")
"status: §1 (§0)"("ok", sample_status())
```

This lowers to the compiler's `scriptum("...", args...)` form. Use the string-template form in ordinary source; reserve `scriptum(...)` for explicit desugaring examples and compiler-facing documentation.

For `textus`, bracket indexing is Unicode-scalar based:

```fab
"Salve, §!"[7]            # "§"
"hello world"[0‥5]        # "hello"
"hello world"[0 usque 10] # "hello world"
"abcdef"[0‥6 per 2]      # "ace"
```

Text slices accept the full range form, including `per`.

### Primary Expressions

```ebnf
primary := IDENTIFIER | NUMBER | STRING
         | 'ego' | 'verum' | 'falsum' | 'nihil'
         | 'vacua' | arrayLiteral | objectLiteral | typedConstructor
         | '(' expression ')'
arrayLiteral := '[' argumentList? ']'
objectLiteral := '{' (objectField (',' objectField)*)? '}'
typedConstructor := typeAnnotation '{' fieldList? '}'
fieldList := objectField (',' objectField)*
objectField := ('sparge' expression) | (objectKey '=' expression) | IDENTIFIER
objectKey := IDENTIFIER | STRING | '[' expression ']'
```

`STRING` includes short strings delimited by `"` and block strings delimited by `❝` and `❞`.

### Special Expressions

```ebnf
// verte (∷) is postfix — parsed in the cast production above
fingeExpr     := 'finge' IDENTIFIER ('{' fieldList '}')? ('∷' IDENTIFIER)?
praefixumExpr := 'praefixum' (blockStmt | '(' expression ')')
formatStringExpr := STRING '(' argumentList ')'                # canonical source form for string formatting
scriptumExpr  := 'scriptum' '(' STRING (',' expression)* ')'   # explicit/desugared form
legeExpr      := 'lege' 'lineam'?
regexLiteral  := 'sed' STRING IDENTIFIER?
```

---

## Patterns

```ebnf
objectPattern  := '{' patternProperty (',' patternProperty)* '}'
patternProperty := 'ceteri'? IDENTIFIER ('ut' IDENTIFIER)?
arrayPattern   := '[' arrayPatternElement (',' arrayPatternElement)* ']'
arrayPatternElement := '_' | 'ceteri'? IDENTIFIER
```

---

## Diagnostics

```ebnf
outputStmt := ('nota' | 'vide' | 'mone' | 'scribe') expression (',' expression)*
```

- `nota` = neutral diagnostic note, `vide` = debug/inspect, `mone` = warn
- `scribe` is a compatibility alias for neutral diagnostic output; use HAL stdlib methods for real output

---

## Entry Points

```ebnf
incipitStmt  := 'incipit' blockStmt
incipietStmt := 'incipiet' blockStmt
```

- `incipit` = sync entry, `incipiet` = async entry

---

## Testing

```ebnf
probandumDecl := 'probandum' STRING probaModifier* '{' probandumBody '}'
probandumBody := (praeparaBlock | probandumDecl | probaStmt)*
probaStmt     := 'proba' STRING probaModifier* blockStmt
probaModifier := 'omitte' STRING | 'futurum' STRING | 'solum' | 'tag' STRING
              | 'temporis' NUMBER | 'metior' | 'repete' NUMBER | 'fragilis' NUMBER
              | 'requirit' STRING | 'solum_in' STRING
praeparaBlock := ('praepara' | 'praeparabit' | 'postpara' | 'postparabit') 'omnia'? blockStmt
```

---

## CLI Framework

```ebnf
cliDecl       := annotation* (incipitStmt | funcDecl)
cliAnnotation := cliProgramAnnotation | imperiumAnnotation | optioAnnotation | operandusAnnotation
```

Faber supports building CLI applications with automatic argument parsing and help generation.

### CLI Entry Point

```fab
@ cli "faber"
@ optio verbose longum "verbose" typus bivalens
incipit argumenta args {
    # CLI framework automatically parses arguments
}
```

### CLI Options and Arguments

```fab
@ imperium "deploy"
@ optio target brevis "t" longum "target" typus textus descriptio "Deployment target"
@ optio verbose brevis "v" longum "verbose" typus bivalens descriptio "Enable verbose output"
@ operandus textus file descriptio "File to deploy"
functio deploy() argumenta args {
    # Arguments automatically parsed and passed
}
```

---

## Capability Calls

```ebnf
adStmt        := 'ad' STRING '(' argumentList ')' adSuccess? adError? blockStmt? catchClause?
adSuccess     := '→' typeAnnotation IDENTIFIER ('ut' IDENTIFIER)?
adError       := '⇥' typeAnnotation
```

**Note:** `ad` names a host/provider capability. Normal compilation is
permissive; unresolved providers compile and fail explicitly at runtime in the
current Rust path. Success bindings are type-first (`→ textus content`), and
recoverable error-channel types use the same `⇥` glyph as failable functions.

---

## Collection Operations

The former `ab` collection pipeline DSL is retired. Collection filtering,
slicing, and aggregation are expressed through ordinary `lista`/`tabula`
library methods and closures instead of a grammar-level query expression.

`prima`, `ultima`, and `summa` are ordinary method names when provided by the
collection library; they are not transform keywords. `ubi` is not active
collection syntax.

`ex` is used for iteration (`itera ex items fixum x`) and imports (`importa ex "path"`).

---

## Fac Block

```ebnf
facBlockStmt := 'fac' blockStmt catchClause? ('dum' expression)?
```

- `fac { ... }` executes the scoped block once.
- `fac { ... } cape err { ... }` is the canonical local recoverable-error boundary.
- `fac { ... } dum condition` is the post-test loop form; postfix `dum` attaches only to `fac`, not arbitrary preceding blocks.

---

## Target-Specific Features

Not all Faber features are supported across all compilation targets. Some features are currently limited to specific targets:

### TypeScript-Only Features

- Some runtime stdlib functions

### Manual-Memory Features (partial)

- `curata` allocator-alias modifiers parse and bind in Faber; Zig-specific lowering
  remains incomplete.
- Arena and page allocators (`cura "arena"`, `cura "page"`) are design targets, not
  fully shipped runtime surfaces.

### Cross-Target Features

- Basic control flow, functions, and types work on all targets
- Collection types (`lista`, `tabula`, `copia`) work on all targets
- Pattern matching and destructuring work on all targets

---

## Keyword Reference

| Category            | Faber                         | Meaning             |
| ------------------- | ----------------------------- | ------------------- |
| **Declarations**    | `fixum`                       | const               |
|                     | `varia`                       | let                 |
|                     | `functio`                     | function            |
|                     | `genus`                       | class               |
|                     | `pactum`                      | interface           |
|                     | `typus`                       | type alias          |
|                     | `ordo`                        | enum                |
|                     | `discretio`                   | tagged union        |
| **Control Flow**    | `si` / `sin` / `secus`        | if / else-if / else |
|                     | `dum`                         | while               |
|                     | `itera ex...fixum`            | for-of (values)     |
|                     | `itera de...fixum`            | for-in (keys)       |
|                     | `itera ab...fixum`            | range iteration     |
|                     | `elige` / `casu`              | switch / case       |
|                     | `discerne`                    | pattern match       |
|                     | `custodi`                     | guard               |
|                     | `redde`                       | return              |
|                     | `rumpe`                       | break               |
|                     | `perge`                       | continue            |
|                     | `tacet`                       | no-op (silence)     |
| **Error Handling**  | `cape`                        | structured local handler |
|                     | `iace`                        | throw               |
|                     | `iacit`                       | throws modifier     |
|                     | `mori`                        | panic               |
|                     | `adfirma`                     | assert              |
| **Async**           | `@ futura`                    | async annotation    |
|                     | `@ cursor`                    | generator annotation |
|                     | `cede`                        | await/yield by context |
| **Boolean**         | `verum`                       | true                |
|                     | `falsum`                      | false               |
|                     | `et`                          | and                 |
|                     | `aut`                         | or                  |
|                     | `non`                         | not                 |
|                     | `vel`                         | nullish coalescing  |
| **Objects**         | `ego`                         | this/self           |
|                     | `finge`                       | construct variant   |
| **Type Shape**      | `∷` | static type ascription / compile-time cast |
| **Type Conversion** | `⇒ target`                    | runtime value conversion |
|                     | `⇒ numerus`                   | parse to integer    |
|                     | `⇒ fractus`                   | parse to float      |
|                     | `⇒ textus`                    | convert to string   |
|                     | `⇒ bivalens`                  | convert to boolean  |
|                     | `Hex` / `Oct` / `Bin` / `Dec` | radix types         |
| **Bitwise**         | `∧` / `∨` / `⊻` / `¬`         | and/or/xor/not      |
|                     | `≪` / `≫`                     | left/right shift    |
| **Diagnostics**     | `nota`                        | neutral note        |
|                     | `vide`                        | debug/inspect       |
|                     | `mone`                        | warn                |
|                     | `scribe`                      | legacy note alias   |

---

## Critical Syntax Rules

1. **Type-first parameters**: `functio f(numerus x)` NOT `functio f(x: numerus)`
2. **Type-first declarations**: `fixum textus name` NOT `fixum name: textus`
3. **Iteration loops**: `itera ex/de collection fixum/varia item { }` or `itera ab range fixum/varia item { }` (verb-first, source, then binding)
4. **Parentheses around conditions are valid but not idiomatic**: prefer `si x > 0 { }` or `si positivum x { }` over `si (x > 0) { }`
5. **Diagnostic keywords are statements**, not functions — `nota x` works, `nota(x)` also works (parentheses group the expression), but `nota` is not a callable value
