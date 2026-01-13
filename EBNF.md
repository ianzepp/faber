# Faber Language Specification

Formal grammar for the Faber programming language. This is the authoritative specification that both faber (TypeScript) and rivus (Faber) compilers implement.

---

## Program Structure

```ebnf
program     := statement*
statement   := importDecl | varDecl | funcDecl | genusDecl | pactumDecl
             | typeAliasDecl | enumDecl | discretioDecl
             | ifStmt | whileStmt | exStmt | deStmt | inStmt
             | eligeStmt | discerneStmt | guardStmt | curaStmt
             | tryStmt | returnStmt | breakStmt | continueStmt | throwStmt
             | assertStmt | outputStmt | adStmt | incipitStmt
             | probandumDecl | probaStmt | blockStmt | exprStmt
blockStmt   := '{' statement* '}'
```

---

## Declarations

### Variables

```ebnf
varDecl      := ('fixum' | 'varia' | 'figendum' | 'variandum') typeAnnotation? IDENTIFIER ('=' expression)?
arrayDestruct := ('fixum' | 'varia') arrayPattern '=' expression
objectDestruct := ('fixum' | 'varia') objectPattern '=' expression
```

- `fixum` = const, `varia` = let
- `figendum`/`variandum` = const/let with await

### Functions

```ebnf
funcDecl     := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt
paramList    := (typeParamDecl ',')* (parameter (',' parameter)*)?
typeParamDecl := 'prae' 'typus' IDENTIFIER
parameter    := ('de' | 'in' | 'ex')? 'si'? 'ceteri'? typeAnnotation? IDENTIFIER ('ut' IDENTIFIER)? ('vel' expression)?
funcModifier := 'futura' | 'cursor' | 'curata' IDENTIFIER
returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
lambdaExpr   := ('pro' | 'fit' | 'fiet') params? ('->' type)? (':' expression | blockStmt)
```

- Return verbs: `fit` (sync), `fiet` (async), `fiunt` (generator), `fient` (async generator)
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
stdlibAnnotation := innatumAnnotation | subsidiaAnnotation | radixAnnotation | verteAnnotation | externaAnnotation
innatumAnnotation := '@' 'innatum' targetMapping (',' targetMapping)*
subsidiaAnnotation := '@' 'subsidia' targetMapping (',' targetMapping)*
radixAnnotation := '@' 'radix' IDENTIFIER (',' IDENTIFIER)*
verteAnnotation := '@' 'verte' IDENTIFIER (STRING | '(' IDENTIFIER (',' IDENTIFIER)* ')' '->' STRING)
externaAnnotation := '@' 'externa'
targetMapping := IDENTIFIER STRING
```

- `@ innatum` maps a genus to native types per target
- `@ subsidia` specifies external implementation files
- `@ radix` declares morphological stem and valid verb forms
- `@ verte` defines codegen transformation (method name or template)
- `@ externa` marks declarations as externally provided (no initializer/body required)

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

### Imports

```ebnf
importDecl    := 'ex' (STRING | IDENTIFIER) 'importa' (specifierList | '*')
specifierList := specifier (',' specifier)*
specifier     := IDENTIFIER ('ut' IDENTIFIER)?
```

---

## Types

```ebnf
typeAnnotation := ('de' | 'in')? (functionType | IDENTIFIER typeParams? '?'? arrayBrackets*)
functionType   := '(' typeList? ')' '->' typeAnnotation
typeList       := typeAnnotation (',' typeAnnotation)*
typeParams     := '<' typeParameter (',' typeParameter)* '>'
typeParameter  := typeAnnotation | NUMBER | MODIFIER
arrayBrackets  := '[]' '?'?
```

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
ifStmt     := 'si' expression (blockStmt | 'ergo' statement | 'reddit' expression)
              ('cape' IDENTIFIER blockStmt)? (elseClause | 'sin' ifStmt)?
elseClause := 'secus' (ifStmt | blockStmt | statement)
```

- `si` = if, `sin` = else-if, `secus` = else
- `ergo` for one-liners, `reddit` for early return

### Loops

```ebnf
whileStmt := 'dum' expression (blockStmt | 'ergo' statement | 'reddit' expression) ('cape' IDENTIFIER blockStmt)?
exStmt    := 'ex' expression (forBinding | destructBinding)
forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement | 'reddit' expression) catchClause?
deStmt    := 'de' expression ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement) catchClause?
```

- `dum` = while
- `ex...pro` = for-of (values)
- `de...pro` = for-in (keys)

### Switch/Match

```ebnf
eligeStmt    := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
eligeCase    := 'casu' expression (blockStmt | 'ergo' statement | 'reddit' expression)
defaultCase  := 'ceterum' (blockStmt | statement)
```

### Pattern Matching

```ebnf
discerneStmt := 'discerne' discriminants '{' variantCase* defaultCase? '}'
discriminants := expression (',' expression)*
variantCase  := 'casu' patterns (blockStmt | 'ergo' statement | 'reddit' expression)
patterns     := pattern (',' pattern)*
pattern      := '_' | (IDENTIFIER patternBind?)
patternBind  := ('ut' IDENTIFIER) | ('pro' IDENTIFIER (',' IDENTIFIER)*)
```

### Guards

```ebnf
guardStmt   := 'custodi' '{' guardClause+ '}'
guardClause := 'si' expression (blockStmt | 'ergo' statement | 'reddit' expression)
```

### Resource Management

```ebnf
curaStmt    := 'cura' curatorKind? expression? ('pro' | 'fit' | 'fiet') typeAnnotation? IDENTIFIER blockStmt catchClause?
curatorKind := 'arena' | 'page'
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
scriptumExpr  := 'scriptum' '(' STRING (',' expression)* ')'
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
incipitStmt  := 'incipit' (blockStmt | 'ergo' statement | 'reddit' expression)
incipietStmt := 'incipiet' (blockStmt | 'ergo' statement | 'reddit' expression)
```

- `incipit` = sync entry, `incipiet` = async entry

---

## Testing

```ebnf
probandumDecl := 'probandum' STRING '{' probandumBody '}'
probandumBody := (praeparaBlock | probandumDecl | probaStmt)*
probaStmt     := 'proba' probaModifier? STRING blockStmt
probaModifier := 'omitte' STRING | 'futurum' STRING
praeparaBlock := ('praepara' | 'praeparabit' | 'postpara' | 'postparabit') 'omnia'? blockStmt
```

---

## Endpoint Dispatch

```ebnf
adStmt        := 'ad' STRING '(' argumentList ')' adBinding? blockStmt? catchClause?
adBinding     := adBindingVerb typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
adBindingVerb := 'fit' | 'fiet' | 'fiunt' | 'fient'
```

**Note:** `ad` statement is parsed but codegen is not yet implemented. See `consilia/futura/ad.md`.

---

## DSL Transforms

```ebnf
dslExpr      := 'ex' expression dslTransforms
dslTransforms := dslTransform (',' dslTransform)*
dslTransform := dslVerb expression?
dslVerb      := 'prima' | 'ultima' | 'summa'

abExpr := 'ab' expression filter? (',' dslTransform)*
filter := 'non'? ('ubi' condition | IDENTIFIER)
```

**Note:** Collection DSL transforms (`prima`, `ultima`, `summa`) are implemented for TypeScript target only. Other targets not yet supported.

---

## Fac Block

```ebnf
facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)? ('dum' expression)?
```

- Creates scope, optionally with catch or do-while

---

## Mutation Block

```ebnf
inStmt := 'in' expression blockStmt
```

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
|                     | `ex...pro`                    | for-of              |
|                     | `de...pro`                    | for-in              |
|                     | `elige` / `casu`              | switch / case       |
|                     | `discerne`                    | pattern match       |
|                     | `custodi`                     | guard               |
|                     | `redde`                       | return              |
|                     | `rumpe`                       | break               |
|                     | `perge`                       | continue            |
| **Error Handling**  | `tempta`                      | try                 |
|                     | `cape`                        | catch               |
|                     | `demum`                       | finally             |
|                     | `iace`                        | throw               |
|                     | `mori`                        | panic               |
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
3. **For-of loops**: `ex collection pro item { }` (collection first)
4. **Parentheses around conditions are valid but not idiomatic**: prefer `si x > 0 { }` or `si positivum x { }` over `si (x > 0) { }`
5. **Output keywords are statements**, not functions — `scribe x` works, `scribe(x)` also works (parentheses group the expression), but `scribe` is not a callable value
