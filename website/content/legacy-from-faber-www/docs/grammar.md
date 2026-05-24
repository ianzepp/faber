---
title: Grammar Reference
description: Formal Faber language grammar in EBNF
section: docs
order: 1
---

# Faber Language Specification

Formal grammar for the Faber programming language. This is the authoritative specification that both faber (TypeScript) and rivus (Faber) compilers implement.

---

## Program Structure

```ebnf
program     := statement*
statement   := directiveDecl | importDecl | varDecl | funcDecl | genusDecl | pactumDecl
             | typeAliasDecl | enumDecl | discretioDecl
             | ifStmt | whileStmt | iteraStmt
             | eligeStmt | discerneStmt | guardStmt | curaStmt
             | tryStmt | returnStmt | breakStmt | continueStmt | throwStmt
             | assertStmt | outputStmt | adStmt | incipitStmt
             | extractStmt
             | probandumDecl | probaStmt | blockStmt | exprStmt
blockStmt   := '{' statement* '}'
```

---

## Declarations

### Variables

```ebnf
varDecl      := ('fixum' | 'varia') typeAnnotation? IDENTIFIER ('=' expression)?
arrayDestruct := ('fixum' | 'varia') arrayPattern '=' expression
objectDestruct := ('fixum' | 'varia') objectPattern '=' expression
```

- `fixum` = const, `varia` = let

### Functions

```ebnf
funcDecl     := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt?
paramList    := (typeParamDecl ',')* (parameter (',' parameter)*)?
typeParamDecl := 'prae' 'typus' IDENTIFIER
parameter    := 'si'? ('de' | 'in' | 'ex')? 'ceteri'? typeAnnotation IDENTIFIER ('ut' IDENTIFIER)? ('vel' expression)?
funcModifier := 'curata' IDENTIFIER | 'errata' IDENTIFIER | 'exitus' (IDENTIFIER | NUMBER) | 'immutata' | 'iacit' | 'optiones' IDENTIFIER
returnClause := '->' typeAnnotation
clausuraExpr   := 'clausura' clausuraParams? ('->' typeAnnotation)? (':' expression | blockStmt)
clausuraParams := clausuraParam (',' clausuraParam)*
clausuraParam  := typeAnnotation IDENTIFIER
```

- Return syntax: `->` (arrow) with optional `@ futura`/`@ cursor` annotations for async/generator
- Parameter prefixes: `de` (read), `in` (mutate), `ex` (consume)
- `si` marks optional, `ceteri` marks rest parameter
- `curata NAME` declares allocator requirement; NAME is auto-injected at call sites within `cura` blocks

### Classes

```ebnf
genusDecl    := 'abstractus'? 'genus' IDENTIFIER typeParams? ('sub' IDENTIFIER)? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
genusMember  := annotation* (fieldDecl | methodDecl)
fieldDecl    := 'generis'? 'nexum'? typeAnnotation IDENTIFIER (':' expression)?
methodDecl   := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt?
annotation   := '@' IDENTIFIER+ | stdlibAnnotation
```

### Stdlib Annotations

```ebnf
annotation       := '@' IDENTIFIER+ | stdlibAnnotation | cliAnnotation
stdlibAnnotation := innatumAnnotation | subsidiaAnnotation | radixAnnotation | verteAnnotation | externaAnnotation
cliAnnotation    := imperiumAnnotation | optioAnnotation | operandusAnnotation | cliAnnotationSimple

innatumAnnotation  := '@' 'innatum' targetMapping (',' targetMapping)*
subsidiaAnnotation := '@' 'subsidia' targetMapping (',' targetMapping)*
radixAnnotation    := '@' 'radix' IDENTIFIER (',' IDENTIFIER)*
verteAnnotation    := '@' 'verte' IDENTIFIER (STRING | '(' IDENTIFIER (',' IDENTIFIER)* ')' '->' STRING)
externaAnnotation  := '@' 'externa'

imperiumAnnotation := '@' 'imperium' | '@' 'cli'
optioAnnotation    := '@' 'optio' IDENTIFIER ('brevis' STRING)? ('longum' STRING)?
                      ('bivalens')? ('descriptio' STRING)?
operandusAnnotation := '@' 'operandus' ('ceteri')? IDENTIFIER IDENTIFIER ('descriptio' STRING)?
cliAnnotationSimple := '@' 'futura' | '@' 'cursor' | '@' 'tag' | '@' 'solum' | '@' 'omitte'
                      | '@' 'metior' | '@' 'publica' | '@' 'protecta' | '@' 'privata'

targetMapping := IDENTIFIER STRING
```

- `@ innatum` maps a genus to native types per target
- `@ subsidia` specifies external implementation files
- `@ radix` declares morphological stem and valid verb forms
- `@ verte` defines codegen transformation (method name or template)
- `@ externa` marks declarations as externally provided (no initializer/body required)
- `@ imperium` / `@ cli` marks a function as a CLI command entry point
- `@ optio` defines a CLI flag option
- `@ operandus` defines a CLI positional argument
- `@ futura` marks function as async (equivalent to `futura` modifier)
- `@ cursor` marks function as generator (equivalent to `cursor` modifier)

- `sub` = extends, `implet` = implements
- `generis` = static, `nexum` = bound/property

### Interfaces

```ebnf
pactumDecl   := 'pactum' IDENTIFIER typeParams? '{' pactumMethod* '}'
pactumMethod := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause?
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
importa ex "norma:scribe" privata scribe ut s
importa ex "lodash" privata * ut _
importa ex "./types" publica User               # re-export
```

---

## Types

```ebnf
typeAnnotation := 'si'? ('de' | 'in')? (functionType | IDENTIFIER typeParams? arrayBrackets*)
functionType   := '(' typeList? ')' '->' typeAnnotation
typeList       := typeAnnotation (',' typeAnnotation)*
typeParams     := '<' typeParameter (',' typeParameter)* '>'
typeParameter  := typeAnnotation | NUMBER | MODIFIER
arrayBrackets  := '[]'
```

- `si` prefix marks nullable types: `si textus` = nullable string
- `de`/`in` mark ownership for Rust/Zig targets: `de textus` = borrowed string
- Combined: `si de textus` = nullable borrowed string

Function types enable higher-order function signatures:

```fab
functio filtrata((T) -> bivalens pred) -> lista<T>
functio compose((A) -> B f, (B) -> C g) -> (A) -> C
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
ifStmt     := 'si' expression (blockStmt | 'ergo' statement | inlineReturn)
              ('cape' IDENTIFIER blockStmt)? (elseClause | 'sin' ifStmt)?
elseClause := 'secus' (ifStmt | blockStmt | statement | inlineReturn)
inlineReturn := 'reddit' expression | 'iacit' expression | 'moritor' expression | 'tacet'
```

- `si` = if, `sin` = else-if, `secus` = else
- `ergo` for one-liners, `reddit` for early return
- `iacit` for inline throw, `moritor` for inline panic
- `tacet` for explicit no-op (from musical notation: "it is silent")

### Loops

```ebnf
whileStmt  := 'dum' expression (blockStmt | 'ergo' statement | inlineReturn) ('cape' IDENTIFIER blockStmt)?
iteraStmt  := 'itera' (('ex' | 'de') expression | 'pro' expression ('per' expression)?) ('fixum' | 'varia') IDENTIFIER (blockStmt | 'ergo' statement | inlineReturn) catchClause?
```

- `dum` = while
- `itera ex...fixum`/`itera ex...varia` = for-of (values)
- `itera de...fixum`/`itera de...varia` = for-in (keys)
- `itera pro range ('per' step)? fixum/varia i` = range iteration (e.g. `itera pro 0..10 per 2 fixum i { scribe i }`)

### Switch/Match

```ebnf
eligeStmt    := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
eligeCase    := 'casu' expression (blockStmt | 'ergo' statement | inlineReturn)
defaultCase  := 'ceterum' (blockStmt | statement | inlineReturn)
```

### Pattern Matching

```ebnf
discerneStmt := 'discerne' 'omnia'? discriminants '{' variantCase* defaultCase? '}'
discriminants := expression (',' expression)*
variantCase  := 'casu' patterns (blockStmt | 'ergo' statement | inlineReturn)
patterns     := pattern ((',' | 'et') pattern)*
pattern      := '_' | literal | (IDENTIFIER patternBind?)
patternBind  := ('ut' IDENTIFIER) | (('fixum' | 'varia') IDENTIFIER (',' IDENTIFIER)*)
```

### Guards

```ebnf
guardStmt   := 'custodi' '{' guardClause+ '}'
guardClause := 'si' expression (blockStmt | 'ergo' statement | inlineReturn)
```

### Resource Management

```ebnf
curaStmt    := 'cura' curatorKind? expression? ('fixum' | 'varia') typeAnnotation? IDENTIFIER blockStmt catchClause?
curatorKind := 'arena' | 'page'
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
```

---

## Error Handling

```ebnf
tryStmt     := 'tempta' blockStmt ('cape' IDENTIFIER blockStmt)? ('demum' blockStmt)?
throwStmt   := ('iace' | 'mori') expression
catchClause := 'cape' IDENTIFIER blockStmt
assertStmt  := 'adfirma' expression (',' expression)?
```

- `tempta` = try, `cape` = catch, `demum` = finally
- `iace` = throw (recoverable), `mori` = panic (fatal)

---

## Expressions

### Operators (by precedence, lowest to highest)

```ebnf
expression := assignment
assignment := ternary (('=' | '+=' | '-=' | '*=' | '/=' | '&=' | '|=') assignment)?
ternary    := or (('?' expression ':' | 'sic' expression 'secus') ternary)?
or         := and (('||' | 'aut') and)* | and ('vel' and)*
and        := equality (('&&' | 'et') equality)*
equality   := comparison (('==' | '!=' | '===' | '!==' | 'est' | 'non' 'est') comparison)*
comparison := bitwiseOr (('<' | '>' | '<=' | '>=' | 'intra' | 'inter') bitwiseOr)*
bitwiseOr  := bitwiseXor ('|' bitwiseXor)*
bitwiseXor := bitwiseAnd ('^' bitwiseAnd)*
bitwiseAnd := shift ('&' shift)*
shift      := range (shiftOp range)*
shiftOp    := 'sinistratum' | 'dextratum'
range      := additive (('..' | 'ante' | 'usque') additive ('per' additive)?)?
additive   := multiplicative (('+' | '-') multiplicative)*
multiplicative := unary (('*' | '/' | '%') unary)*
unary      := ('-' | '~' | 'non' | 'nulla' | 'nonnulla' | 'nihil' | 'nonnihil' | 'negativum' | 'positivum' | 'cede' | 'novum' | 'finge') unary | cast
cast       := call ('qua' typeAnnotation | 'innatum' typeAnnotation | conversionOp)*
conversionOp := ('numeratum' | 'fractatum') typeParams? ('vel' unary)?
             | ('textatum' | 'bivalentum')
```

**Type casting vs construction:**

- `qua` = compile-time type assertion only. No runtime effect. Use when you know a value's type but the compiler doesn't.
    - `data qua textus` → TypeScript: `(data as string)`

- `innatum` = native type construction. Actually builds the target type at runtime. Use for built-in collections:
    - `[] innatum lista<T>` → typed empty array
    - `{} innatum tabula<K,V>` → `new Map<K,V>()`
    - `[] innatum copia<T>` → `new Set<T>()`

**Important:** Do NOT use `qua` for collection construction. `{} qua copia<T>` produces a plain object with a type assertion—it will NOT have Set methods like `.add()` or `.has()`. Use `innatum` instead.

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

### Primary Expressions

```ebnf
primary := IDENTIFIER | NUMBER | STRING | TEMPLATE_STRING
         | 'ego' | 'verum' | 'falsum' | 'nihil'
         | arrayLiteral | objectLiteral
         | '(' expression ')'
```

### Special Expressions

```ebnf
newExpr       := 'novum' IDENTIFIER ('(' argumentList ')')? (objectLiteral | 'de' expression)?
fingeExpr     := 'finge' IDENTIFIER ('{' fieldList '}')? ('qua' IDENTIFIER)?
praefixumExpr := 'praefixum' (blockStmt | '(' expression ')')
scriptumExpr  := 'scriptum' '(' STRING (',' expression)* ')'  # § placeholders filled positionally
legeExpr      := 'lege' 'lineam'?
regexLiteral  := 'sed' STRING IDENTIFIER?
```

---

## Patterns

```ebnf
objectPattern  := '{' patternProperty (',' patternProperty)* '}'
patternProperty := 'ceteri'? IDENTIFIER (':' IDENTIFIER)?
arrayPattern   := '[' arrayPatternElement (',' arrayPatternElement)* ']'
arrayPatternElement := '_' | 'ceteri'? IDENTIFIER
```

---

## Output

```ebnf
outputStmt := ('scribe' | 'vide' | 'mone') expression (',' expression)*
```

- `scribe` = log, `vide` = debug, `mone` = warn

---

## Entry Points

```ebnf
incipitStmt  := 'incipit' (blockStmt | 'ergo' statement | inlineReturn)
incipietStmt := 'incipiet' (blockStmt | 'ergo' statement | inlineReturn)
```

- `incipit` = sync entry, `incipiet` = async entry

---

## Testing

```ebnf
probandumDecl := 'probandum' STRING '{' probandumBody '}'
probandumBody := (praeparaBlock | probandumDecl | probaStmt)*
probaStmt     := 'proba' probaModifier? STRING blockStmt
probaModifier := 'omitte' STRING | 'futurum' STRING | 'solum' | 'tag' STRING
              | 'temporis' NUMBER | 'metior' | 'repete' NUMBER | 'fragilis' NUMBER
              | 'requirit' STRING | 'solum_in' STRING
praeparaBlock := ('praepara' | 'praeparabit' | 'postpara' | 'postparabit') 'omnia'? blockStmt
```

---

## CLI Framework

```ebnf
cliDecl     := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt?
cliAnnotation := '@' ('imperium' | 'cli' | 'optio' | 'operandus')
```

Faber supports building CLI applications with automatic argument parsing and help generation.

### CLI Entry Point

```fab
@ imperium
functio main() {
    # CLI framework automatically parses arguments
}
```

### CLI Options and Arguments

```fab
@ imperium
functio deploy(
    @ optio target brevis "t" longum "target" descriptio "Deployment target"
    textus target,

    @ optio verbose brevis "v" longum "verbose" bivalens descriptio "Enable verbose output"
    bivalens verbose,

    @ operandus textus file descriptio "File to deploy"
    textus file
) {
    # Arguments automatically parsed and passed
}
```

---

## Endpoint Dispatch

```ebnf
adStmt        := 'ad' STRING '(' argumentList ')' adBinding? blockStmt? catchClause?
adBinding     := '->' typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
```

**Note:** `ad` statement is parsed but codegen is not yet implemented. See `consilia/futura/ad.md`.

---

## Collection DSL

```ebnf
abExpr        := 'ab' expression filter? (',' dslTransform)*
filter        := 'non'? ('ubi' condition | IDENTIFIER)
dslTransforms := dslTransform (',' dslTransform)*
dslTransform  := dslVerb expression?
dslVerb       := 'prima' | 'ultima' | 'summa'
```

`ab` is the sole collection DSL entry point. Filter is optional, allowing direct transforms:

- `ab users activus` - filter by boolean property
- `ab users ubi aetas >= 18` - filter by condition
- `ab items prima 5` - direct transform (no filter)
- `ab items prima 10, ultima 3` - chained transforms

`ex` is used for iteration (`itera ex items fixum x`) and imports (`importa ex "path"`).

**Note:** Collection DSL transforms (`prima`, `ultima`, `summa`) are implemented for TypeScript target only. Other targets not yet supported.

---

## Fac Block

```ebnf
facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)? ('dum' expression)?
```

- Creates scope, optionally with catch or do-while

---

## Target-Specific Features

Not all Faber features are supported across all compilation targets. Some features are currently limited to specific targets:

### TypeScript-Only Features

- Collection DSL transforms (`prima`, `ultima`, `summa`, `ordina`, etc.)
- Some runtime stdlib functions

### Zig-Only Features

- `curata` allocator binding (for manual memory management)
- Arena and page allocators (`cura arena`, `cura pagina`)

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
|                     | `itera pro...fixum`           | range iteration     |
|                     | `elige` / `casu`              | switch / case       |
|                     | `discerne`                    | pattern match       |
|                     | `custodi`                     | guard               |
|                     | `redde`                       | return              |
|                     | `reddit`                      | inline return       |
|                     | `rumpe`                       | break               |
|                     | `perge`                       | continue            |
|                     | `tacet`                       | no-op (silence)     |
| **Error Handling**  | `tempta`                      | try                 |
|                     | `cape`                        | catch               |
|                     | `demum`                       | finally             |
|                     | `iace`                        | throw               |
|                     | `iacit`                       | inline throw        |
|                     | `mori`                        | panic               |
|                     | `moritor`                     | inline panic        |
|                     | `adfirma`                     | assert              |
| **Async**           | `futura`                      | async modifier      |
|                     | `cede`                        | await               |
| **Boolean**         | `verum`                       | true                |
|                     | `falsum`                      | false               |
|                     | `et`                          | and                 |
|                     | `aut`                         | or                  |
|                     | `non`                         | not                 |
|                     | `vel`                         | nullish coalescing  |
| **Objects**         | `ego`                         | this/self           |
|                     | `novum`                       | new                 |
|                     | `finge`                       | construct variant   |
| **Type Conversion** | `numeratum`                   | parse to integer    |
|                     | `fractatum`                   | parse to float      |
|                     | `textatum`                    | convert to string   |
|                     | `bivalentum`                  | convert to boolean  |
|                     | `Hex` / `Oct` / `Bin` / `Dec` | radix types         |
| **Bitwise**         | `sinistratum`                 | left shift (<<)     |
|                     | `dextratum`                   | right shift (>>)    |
| **Output**          | `scribe`                      | log                 |
|                     | `vide`                        | debug               |
|                     | `mone`                        | warn                |

---

## Critical Syntax Rules

1. **Type-first parameters**: `functio f(numerus x)` NOT `functio f(x: numerus)`
2. **Type-first declarations**: `fixum textus name` NOT `fixum name: textus`
3. **Iteration loops**: `itera ex/de/pro collection/range fixum/varia item { }` (verb-first, source, then binding)
4. **Parentheses around conditions are valid but not idiomatic**: prefer `si x > 0 { }` or `si positivum x { }` over `si (x > 0) { }`
5. **Output keywords are statements**, not functions — `scribe x` works, `scribe(x)` also works (parentheses group the expression), but `scribe` is not a callable value

