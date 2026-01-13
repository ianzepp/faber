# Parser Implementation Gap Analysis
## Faber (TypeScript) vs Rivus (Faber)

**Generated:** 2026-01-12
**Reference:** EBNF.md (canonical grammar specification)

---

## Executive Summary

This document compares the parser implementations between:
- **Faber** (TypeScript): `fons/faber/parser/` - Reference implementation
- **Rivus** (Faber): `fons/rivus/parser/` - Self-hosting compiler

The Rivus parser is largely complete and follows the same architecture as Faber, but is **missing several advanced features** documented in EBNF.md that are implemented in the TypeScript version.

---

## 1. AST Node Types

### 1.1 Statements (Both Implementations)

**Core Control Flow** ✅ Both implement:
- `SiSententia` / `SiStatement` - if/else conditionals
- `DumSententia` / `DumStatement` - while loops
- `ExSententia` / `IteratioStatement` - for-of iteration (ex...pro)
- `DeSententia` / `IteratioStatement` - for-in iteration (de...pro)
- `ReddeSententia` / `ReddeStatement` - return
- `RumpeSententia` / `RumpeStatement` - break
- `PergeSententia` / `PergeStatement` - continue

**Declarations** ✅ Both implement:
- `VariaSententia` / `VariaDeclaration` - variable declarations
- `FunctioDeclaratio` / `FunctioDeclaration` - function definitions
- `GenusDeclaratio` / `GenusDeclaration` - class/struct types
- `PactumDeclaratio` / `PactumDeclaration` - interface contracts
- `OrdoDeclaratio` / `OrdoDeclaration` - enums
- `DiscretioDeclaratio` / `DiscretioDeclaration` - tagged unions
- `TypusAliasDeclaratio` / `TypeAliasDeclaration` - type aliases
- `ImportaSententia` / `ImportaDeclaration` - module imports

**Error Handling** ✅ Both implement:
- `TemptaSententia` / `TemptaStatement` - try/catch/finally
- `IaceSententia` / `IaceStatement` - throw/panic
- `AdfirmaSententia` / `AdfirmaStatement` - assertions

**Pattern Matching & Flow** ✅ Both implement:
- `EligeSententia` / `EligeStatement` - switch/case (value matching)
- `DiscerneSententia` / `DiscerneStatement` - pattern matching on variants
- `CustodiSententia` / `CustodiStatement` - guard clauses

**I/O & Output** ✅ Both implement:
- `ScribeSententia` / `ScribeStatement` - console output (scribe/vide/mone)

**Blocks & Scoping** ✅ Both implement:
- `MassaSententia` / `BlockStatement` - block statements
- `ExpressiaSententia` / `ExpressionStatement` - expression as statement
- `InSententia` / `InStatement` - mutation context blocks

### 1.2 Statements (Missing from Rivus)

**Testing Framework** ❌ Not in Rivus:
- `ProbandumStatement` - test suites (probandum "name" { })
- `ProbaStatement` - individual tests (proba "name" { })
- `PraeparaBlock` - setup/teardown hooks (praepara/postpara [omnia] { })

**Advanced Features** ❌ Not in Rivus:
- `CuraStatement` - resource management with allocators (cura arena/page fit binding { })
- `AdStatement` - dispatch/syscall mechanism (ad "target" (args) fit type pro binding { })
- `IncipitStatement` - sync entry point (incipit { })
- `IncipietStatement` - async entry point (incipiet { })
- `FacBlockStatement` - explicit scope with catch/do-while (fac { } cape { } dum)
- `DestructureDeclaration` - ex-prefixed destructuring (ex obj fixum x, y)

---

## 2. Expression Types

### 2.1 Expressions (Both Implementations)

**Primary Expressions** ✅ Both implement:
- `Nomen` / `Identifier` - variable/function names
- `Littera` / `Literal` - strings, numbers, booleans, null
- `EgoExpressia` / `EgoExpression` - self-reference (ego)
- `SeriesExpressia` / `ArrayExpression` - array literals
- `ObiectumExpressia` / `ObjectExpression` - object literals
- `LambdaExpressia` / `LambdaExpression` - anonymous functions

**Binary & Unary** ✅ Both implement:
- `BinariaExpressia` / `BinaryExpression` - infix operators
- `UnariaExpressia` / `UnaryExpression` - prefix operators
- `AssignatioExpressia` / `AssignmentExpression` - assignment
- `CondicioExpressia` / `ConditionalExpression` - ternary

**Type Operations** ✅ Both implement:
- `EstExpressia` / `EstExpression` - type checks (est/non est)
- `QuaExpressia` / `QuaExpression` - type casts

**Advanced Expressions** ✅ Both implement:
- `VocatioExpressia` / `CallExpression` - function calls
- `MembrumExpressia` / `MemberExpression` - property access
- `NovumExpressia` / `NovumExpression` - object construction
- `FingeExpressia` / `FingeExpression` - variant construction
- `CedeExpressia` / `CedeExpression` - await
- `AmbitusExpressia` / `RangeExpression` - ranges (.., ante, usque)

**DSL & I/O** ✅ Both implement:
- `CatenaExpressia` / `CollectionDSLExpression` - collection transforms (ex...prima/ultima)
- `AbExpressia` / `AbExpression` - filtering DSL (ab...ubi)
- `ScriptumExpressia` / `ScriptumExpression` - format strings
- `LegeExpressia` / `LegeExpression` - stdin reading
- `LitteraRegex` / `RegexLiteral` - regex patterns

### 2.2 Expressions (Missing from Rivus)

**Type Conversions** ❌ Not in Rivus:
- `InnatumExpression` - native type construction (innatum tabula<K,V>)
- `ConversionExpression` - type conversions (numeratum, fractatum, textatum, bivalentum)

**Advanced Operators** ❌ Not in Rivus:
- `ShiftExpression` - bitwise shifts (dextratum/sinistratum) - Note: Rivus uses binary expressions for shifts

**Compile-time** ❌ Not in Rivus:
- `PraefixumExpression` - compile-time evaluation (praefixum { } / praefixum(expr))

**Collection DSL** ⚠️ Partially in Rivus:
- `SpreadElement` / `DispersioElementum` - spread operator (sparge) ✅ Present
- Template literals - ✅ Present in both as Littera/Literal with species Exemplar

---

## 3. Expression Parsing (Precedence & Operators)

### 3.1 Precedence Hierarchy (Both Implementations)

Both parsers use the same precedence climbing algorithm:

```
1. Assignment (=, +=, -=, etc.)         [Right-associative]
2. Ternary (? : / sic secus)            [Right-associative]
3. Logical OR (|| / aut / ?? / vel)
4. Logical AND (&& / et)
5. Equality (== / != / === / !== / est / non est)
6. Comparison (< > <= >= / intra / inter)
7. Bitwise OR (|)
8. Bitwise XOR (^)
9. Bitwise AND (&)
10. Shift (<< >> >>> / dextratum / sinistratum)
11. Range (.., ante, usque)
12. Additive (+ -)
13. Multiplicative (*, /, %)
14. Unary (non, -, ~, nulla, nonnulla, nihil, nonnihil, negativum, positivum, cede)
15. Cast/Conversion (qua, innatum, numeratum, fractatum, textatum, bivalentum)
16. Call & Member ((), [], ., ?., !.)
17. Primary (literals, identifiers, grouping)
```

**Key Differences:**
- Rivus implements shifts at level 10 as `BinariaExpressia`, Faber has dedicated `ShiftExpression` type
- Rivus missing level 15 type conversion operators (numeratum/fractatum/textatum/bivalentum)
- Rivus missing innatum construction

### 3.2 Operator Coverage

**Arithmetic** ✅ Both:
- `+`, `-`, `*`, `/`, `%`

**Comparison** ✅ Both:
- `<`, `>`, `<=`, `>=`, `intra`, `inter`

**Equality** ✅ Both:
- `==`, `!=`, `===`, `!==`, `est`, `non est`

**Logical** ✅ Both:
- `&&` / `et`, `||` / `aut`, `??` / `vel`

**Bitwise** ✅ Both:
- `&`, `|`, `^`, `<<` / `dextratum`, `>>` / `sinistratum`, `>>>`

**Unary** ✅ Both:
- `!` / `non`, `-`, `~`
- Null checks: `nulla`, `nonnulla`, `nihil`, `nonnihil`
- Sign checks: `negativum`, `positivum`

**Assignment** ✅ Both:
- `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`

**Type Operations** ⚠️ Partial:
- `qua` (cast) ✅ Both
- `est` / `non est` (type check) ✅ Both
- `innatum` (native construction) ❌ Rivus missing
- `numeratum`, `fractatum`, `textatum`, `bivalentum` (conversions) ❌ Rivus missing

**Range** ✅ Both:
- `..` (exclusive), `ante` (exclusive), `usque` (inclusive), `per` (step)

**Async** ✅ Both:
- `cede` (await)

**Object Construction** ✅ Both:
- `novum` (new), `finge` (variant construction)

---

## 4. Statement Parsing

### 4.1 Control Flow Parsing

**Conditionals** ✅ Both fully implement:
- If/else with optional catch: `si condition { } sin condition { } secus { } cape err { }`
- Short forms: `si cond ergo stmt`, `si cond reddit expr`

**Loops** ✅ Both fully implement:
- While with catch: `dum condition { } cape err { }`
- For-of: `ex collection pro item { }` (sync), `ex collection fiet item { }` (async)
- For-in: `de object pro key { }` (sync), `de object fiet key { }` (async)
- Short forms: `dum cond ergo stmt`, `ex items pro x ergo stmt`

**Pattern Matching** ✅ Both fully implement:
- Switch: `elige discriminant { casu value { } ceterum { } } cape err { }`
- Variant matching: `discerne variant { casu Pattern pro x, y { } casu _ { } }`
- Multi-discriminant: `discerne left, right { casu P1, P2 { } }`
- Wildcard patterns: `_`
- Destructuring patterns: `casu Click pro x, y { }`
- Binding patterns: `casu Click ut c { }`

**Guards** ✅ Both fully implement:
- `custodi { si condition { redde/iace } }`

### 4.2 Declaration Parsing

**Variables** ✅ Both:
- `varia`, `fixum`, `figendum`, `variandum`
- Type annotations (type-first): `fixum numerus x = 42`
- Array destructuring: `fixum [x, y, ceteri rest] = arr`
- Object destructuring: `fixum {name, age} = obj`
- Ex-prefixed destructuring: ❌ Faber only

**Functions** ✅ Both:
- Parameters with prepositions: `de`, `in`, `ex`
- Optional parameters: `si`
- Default values: `vel`
- Rest parameters: `ceteri`
- Return verbs: `->`, `fit`, `fiet`, `fiunt`, `fient`
- Modifiers: `futura`, `cursor`, `curata NAME`
- Type parameters: `prae typus T`
- Abstract methods (no body)

**Classes/Structs** ✅ Both:
- Inheritance: `sub ParentClass`
- Interfaces: `implet Interface1, Interface2`
- Abstract: `abstractus genus`
- Fields with visibility
- Methods with visibility
- Static fields: `generis`
- Bound fields: `nexum`

**Interfaces** ✅ Both:
- Method signatures without body
- Type parameters

**Enums** ✅ Both:
- Constant values: `ordo Color { rubrum, viridis = 2, caeruleum }`
- String values: `ordo Status { activa = "active" }`

**Tagged Unions** ✅ Both:
- Variants: `discretio Result<T, E> { Recte { T valor }, Erratum { E error } }`
- Variant fields with types

**Type Aliases** ✅ Both:
- `typus ID = typeAnnotation`

**Imports** ✅ Both:
- `ex "module" importa specifier1, specifier2`
- `ex "module" importa *`
- Aliases: `ex "module" importa name ut alias`

### 4.3 Error Handling

**Try/Catch/Finally** ✅ Both:
- `tempta { } cape err { } demum { }`

**Throw** ✅ Both:
- `iace "error message"` - recoverable
- `mori "fatal error"` - panic/fatal

**Assertions** ✅ Both:
- `adfirma condition` - abort if false
- `adfirma condition, "message"` - abort with message

**Fac Blocks** ❌ Faber only:
- `fac { } cape err { } dum condition` - explicit scope with catch/do-while

### 4.4 Advanced Features

**Testing** ❌ Faber only:
- Test suites: `probandum "suite" { }`
- Individual tests: `proba "test" { }`, `proba omitte "skip" { }`, `proba futurum "todo" { }`
- Hooks: `praepara { }`, `praeparabit { }`, `postpara { }`, `postparabit { }`
- Scoped hooks: `praepara omnia { }` (suite-level)

**Resource Management** ❌ Faber only:
- Allocators: `cura arena fit mem { }`, `cura page fit mem { }`
- Resources: `cura open(file) fit fd { }`, `cura connect(url) fiet conn { }`
- Cleanup: automatic solve() call on scope exit

**Dispatch** ❌ Faber only:
- Syscalls: `ad "console:log" ("hello")`
- HTTP: `ad "https://api.example.com/users" (userId) fiet Response pro resp { }`
- Binding verbs: `fit`, `fiet`, `fiunt`, `fient`

**Entry Points** ❌ Faber only:
- `incipit { }` - sync main
- `incipiet { }` - async main

**Mutation Blocks** ✅ Both:
- `in object { field = value }`

**Output** ✅ Both:
- `scribe "message"` - log
- `vide "message"` - debug
- `mone "message"` - warn

---

## 5. Pattern Matching (discerne) Implementation

### 5.1 Pattern Types (Both Implementations)

**Variant Patterns** ✅ Both:
```faber
discerne result {
    casu Recte pro valor { scribe valor }
    casu Erratum pro error { mone error }
}
```

**Destructuring** ✅ Both:
```faber
discerne event {
    casu Click pro x, y { scribe scriptum("§, §", x, y) }
    casu Keypress pro key { scribe key }
}
```

**Binding** ✅ Both:
```faber
discerne event {
    casu Click ut c { scribe c.x, c.y }
    casu Quit ut q { scribe q }
}
```

**Wildcard** ✅ Both:
```faber
discerne result {
    casu Recte pro v { scribe v }
    casu _ { scribe "error" }
}
```

**Multi-discriminant** ✅ Both:
```faber
discerne left, right {
    casu Numerus ut l, Numerus ut r { redde l.valor === r.valor }
    casu Textus ut l, Textus ut r { redde l.valor === r.valor }
    casu _, _ { redde falsum }
}
```

### 5.2 Pattern Implementation Details

**Rivus Implementation:**
- Located in: `fons/rivus/parser/sententia/fluxus.fab`
- Function: `parseDiscerneSententia()`
- Pattern parsing: `parseVariansExemplar()`
- Disambiguation: `estProximumVinculum()` - distinguishes bindings from patterns
- AST nodes: `DiscerneSententia`, `VariansCasus`, `VariansExemplar`

**Faber Implementation:**
- Located in: `fons/faber/parser/index.ts`
- Function: `parseDiscerneStatement()`
- Pattern parsing: `parseVariantPattern()`
- AST nodes: `DiscerneStatement`, `VariantCase`, `VariantPattern`

**Both implementations handle:**
- Comma disambiguation (bindings vs patterns)
- Wildcard detection (`_`)
- Alias binding (`ut`)
- Field destructuring (`pro`)
- Multi-discriminant matching

---

## 6. Grammar Constructs Comparison

### 6.1 Features Present in Both

| Feature | Faber | Rivus | EBNF.md |
|---------|-------|-------|---------|
| Variable declarations | ✅ | ✅ | ✅ |
| Function declarations | ✅ | ✅ | ✅ |
| Class/struct declarations | ✅ | ✅ | ✅ |
| Interface declarations | ✅ | ✅ | ✅ |
| Enum declarations | ✅ | ✅ | ✅ |
| Tagged union declarations | ✅ | ✅ | ✅ |
| Type aliases | ✅ | ✅ | ✅ |
| Import statements | ✅ | ✅ | ✅ |
| If/else statements | ✅ | ✅ | ✅ |
| While loops | ✅ | ✅ | ✅ |
| For-of loops | ✅ | ✅ | ✅ |
| For-in loops | ✅ | ✅ | ✅ |
| Switch statements | ✅ | ✅ | ✅ |
| Pattern matching | ✅ | ✅ | ✅ |
| Guard clauses | ✅ | ✅ | ✅ |
| Try/catch/finally | ✅ | ✅ | ✅ |
| Throw/panic | ✅ | ✅ | ✅ |
| Assertions | ✅ | ✅ | ✅ |
| Return/break/continue | ✅ | ✅ | ✅ |
| Block statements | ✅ | ✅ | ✅ |
| Expression statements | ✅ | ✅ | ✅ |
| Binary expressions | ✅ | ✅ | ✅ |
| Unary expressions | ✅ | ✅ | ✅ |
| Assignment expressions | ✅ | ✅ | ✅ |
| Ternary expressions | ✅ | ✅ | ✅ |
| Function calls | ✅ | ✅ | ✅ |
| Member access | ✅ | ✅ | ✅ |
| Array literals | ✅ | ✅ | ✅ |
| Object literals | ✅ | ✅ | ✅ |
| Lambda expressions | ✅ | ✅ | ✅ |
| Range expressions | ✅ | ✅ | ✅ |
| Type casts | ✅ | ✅ | ✅ |
| Type checks | ✅ | ✅ | ✅ |
| Async/await | ✅ | ✅ | ✅ |
| Object construction | ✅ | ✅ | ✅ |
| Variant construction | ✅ | ✅ | ✅ |
| Collection DSL (ex/ab) | ✅ | ✅ | ✅ |
| Format strings | ✅ | ✅ | ✅ |
| Stdin reading | ✅ | ✅ | ✅ |
| Regex literals | ✅ | ✅ | ✅ |
| Console output | ✅ | ✅ | ✅ |
| Mutation blocks | ✅ | ✅ | ✅ |
| Spread operator | ✅ | ✅ | ✅ |
| Optional chaining | ✅ | ✅ | ✅ |
| Non-null assertion | ✅ | ✅ | ✅ |
| Template literals | ✅ | ✅ | ✅ |

### 6.2 Features Missing from Rivus

| Feature | Faber | Rivus | EBNF.md | Priority |
|---------|-------|-------|---------|----------|
| Test framework (probandum/proba) | ✅ | ❌ | ✅ | **HIGH** |
| Resource management (cura) | ✅ | ❌ | ✅ | **HIGH** |
| Entry points (incipit/incipiet) | ✅ | ❌ | ✅ | **HIGH** |
| Dispatch statements (ad) | ✅ | ❌ | ✅ | **MEDIUM** |
| Fac blocks | ✅ | ❌ | ✅ | **LOW** |
| Ex-destructuring | ✅ | ❌ | ✅ | **LOW** |
| Compile-time expressions (praefixum) | ✅ | ❌ | ✅ | **MEDIUM** |
| Type conversions (numeratum/fractatum/etc) | ✅ | ❌ | ✅ | **MEDIUM** |
| Native construction (innatum) | ✅ | ❌ | ✅ | **MEDIUM** |
| Dedicated shift expressions | ✅ | ❌ | ✅ | **LOW** |

### 6.3 Features Not in EBNF.md

| Feature | Faber | Rivus | Notes |
|---------|-------|-------|-------|
| Morphology tracking | ✅ | ✅ | Implementation detail, not grammar |
| Comment attachment | ✅ | ✅ | AST metadata, not grammar |
| Error recovery | ✅ | ✅ | Parser implementation detail |

---

## 7. Architectural Comparison

### 7.1 Parser Structure

**Faber (TypeScript):**
- Single-file parser: `fons/faber/parser/index.ts` (~3500 lines)
- AST definitions: `fons/faber/parser/ast.ts` (~2800 lines)
- Errors: `fons/faber/parser/errors.ts`
- Monolithic approach with all parsing logic in one file

**Rivus (Faber):**
- Modular parser split across files:
  - `nucleus.fab` - Core state and navigation primitives
  - `resolvitor.fab` - Parser interface (dependency injection)
  - `index.fab` - Main entry point and Parsator genus
  - `expressia/` - Expression parsing (binaria.fab, unaria.fab, primaria.fab)
  - `sententia/` - Statement parsing (actio.fab, declara.fab, fluxus.fab, etc.)
  - `typus.fab` - Type annotation parsing
  - `morphologia.fab` - Morphology tracking
  - `errores.fab` - Error types
- Modular approach with separation of concerns

**Key Architectural Differences:**
1. Faber uses direct recursive calls, Rivus uses `Resolvitor` interface to avoid circular imports
2. Faber has single large file, Rivus split into ~20 files by concern
3. Both use recursive descent with precedence climbing for expressions
4. Both use token lookahead for disambiguation
5. Both collect errors without throwing (error recovery)

### 7.2 Error Handling

**Both implementations:**
- Non-throwing error collection
- Synchronization at statement boundaries
- Error recovery with synthetic tokens
- Detailed error messages with source locations

**Error recovery strategies:**
- Statement synchronization: skip to next statement keyword or block boundary
- Genus member synchronization: skip to next field/method or closing brace
- Synthetic token insertion on expected token missing

### 7.3 Comment Handling

**Both implementations:**
- Leading comments: attached to next AST node
- Trailing comments: attached to same-line AST node
- Comment types: line, block, doc
- Comment preservation for codegen

---

## 8. Gap Analysis Summary

### 8.1 Critical Gaps (Block Self-Hosting)

**None.** Rivus can parse all constructs needed to compile itself.

### 8.2 High-Priority Gaps (Core Language Features)

1. **Test Framework** (probandum/proba/praepara/postpara)
   - Status: Fully documented in EBNF.md, implemented in Faber
   - Impact: Cannot run Faber test suites with Rivus
   - Complexity: Medium (new statement types, test runner integration)

2. **Resource Management** (cura)
   - Status: Fully documented in EBNF.md, implemented in Faber
   - Impact: Cannot use RAII-style resource cleanup
   - Complexity: High (requires codegen for allocators, cleanup logic)

3. **Entry Points** (incipit/incipiet)
   - Status: Fully documented in EBNF.md, implemented in Faber
   - Impact: Cannot define explicit program entry points
   - Complexity: Low (new statement types, codegen integration)

### 8.3 Medium-Priority Gaps (Advanced Features)

4. **Dispatch Statements** (ad)
   - Status: Grammar in EBNF.md, parsing in Faber, **codegen not implemented**
   - Impact: Cannot use syscall/HTTP dispatch mechanism
   - Complexity: High (requires runtime integration)

5. **Compile-time Expressions** (praefixum)
   - Status: Grammar in EBNF.md, parsing in Faber
   - Impact: Cannot use compile-time code execution
   - Complexity: High (requires interpreter or compile-time VM)

6. **Type Conversions** (numeratum/fractatum/textatum/bivalentum)
   - Status: Grammar in EBNF.md, parsing in Faber
   - Impact: Cannot use explicit type conversion operators
   - Complexity: Low (parsing), Medium (codegen)

7. **Native Construction** (innatum)
   - Status: Grammar in EBNF.md, parsing in Faber
   - Impact: Cannot use explicit native type construction
   - Complexity: Medium (parsing and codegen)

### 8.4 Low-Priority Gaps (Syntactic Sugar)

8. **Fac Blocks**
   - Status: Grammar in EBNF.md, parsing in Faber
   - Impact: Must use explicit try/catch or while instead
   - Workaround: `tempta { }` or `dum condition { }`
   - Complexity: Low

9. **Ex-Destructuring** (ex obj fixum x, y)
   - Status: Grammar in EBNF.md, parsing in Faber
   - Impact: Must use standard destructuring
   - Workaround: `fixum {x, y} = obj`
   - Complexity: Low

10. **Dedicated Shift Expression Type**
    - Status: Faber has `ShiftExpression`, Rivus uses `BinariaExpressia`
    - Impact: None (semantic equivalence)
    - Complexity: Low (AST refactor)

---

## 9. Implementation Recommendations

### 9.1 Immediate Actions (Week 1-2)

1. **Add Entry Point Statements**
   - Parser: Add `IncipitSententia` and `IncipietSententia` to `sententia/initus.fab`
   - AST: Define nodes in `ast/sententia` module
   - Dispatcher: Add to `sententia/index.fab`
   - Codegen: Map to `main()` function in target language

2. **Add Type Conversion Operators**
   - Parser: Add to `expressia/unaria.fab` (prefix operators)
   - AST: Add `ConversioExpressia` variant to `ast/expressia`
   - Codegen: Map to parseInt(), parseFloat(), String(), Boolean()

### 9.2 Short-term Goals (Week 3-4)

3. **Implement Test Framework**
   - Parser: Add to `sententia/proba.fab`
   - AST: Define `ProbandumSententia`, `ProbaSententia`, `PraeparaMassa`
   - Dispatcher: Add to `sententia/index.fab`
   - Runtime: Integrate with test runner (defer to codegen phase)

4. **Add Native Construction**
   - Parser: Add `innatum` to `expressia/unaria.fab`
   - AST: Add `InnatumExpressia` to `ast/expressia`
   - Codegen: Map to native type constructors

### 9.3 Medium-term Goals (Month 2)

5. **Implement Resource Management**
   - Parser: Add to `sententia/initus.fab`
   - AST: Define `CuraSententia` with allocator types
   - Codegen: Generate cleanup calls (defer, RAII, try-finally)
   - Runtime: Implement allocator interface

6. **Add Compile-time Expressions**
   - Parser: Add to `expressia/primaria.fab`
   - AST: Add `PraefixumExpressia` to `ast/expressia`
   - Interpreter: Implement compile-time evaluator
   - Codegen: Inline constant results

### 9.4 Long-term Goals (Month 3+)

7. **Implement Dispatch Statements**
   - Parser: Add to `sententia/initus.fab`
   - AST: Define `AdSententia` with binding verbs
   - Runtime: Implement dispatch mechanism
   - Codegen: Generate appropriate target code

8. **Add Syntactic Sugar**
   - Fac blocks: Add to `sententia/error.fab`
   - Ex-destructuring: Add to `sententia/varia.fab`
   - Shift expression type: Refactor `expressia/binaria.fab`

---

## 10. Testing Strategy

### 10.1 Parser Tests

**Current Coverage:**
- Expression precedence: ✅ Both
- Statement parsing: ✅ Both
- Pattern matching: ✅ Both
- Error recovery: ✅ Both

**Gaps to Test:**
- Entry points: ❌ Rivus
- Test framework: ❌ Rivus
- Resource management: ❌ Rivus
- Type conversions: ❌ Rivus
- Native construction: ❌ Rivus
- Compile-time expressions: ❌ Rivus
- Dispatch statements: ❌ Rivus

**Recommended Approach:**
1. Port Faber parser tests to Rivus test format
2. Add regression tests for each new feature
3. Verify AST equivalence between Faber and Rivus parsers
4. Test error recovery for new constructs

### 10.2 Integration Tests

**Test against:**
- Self-hosting: Rivus can parse itself ✅
- Faber corpus: Rivus can parse all Faber source files ⚠️ (missing features)
- EBNF compliance: All grammar constructs from EBNF.md ⚠️ (missing features)

**Success Criteria:**
1. Rivus parses all fons/rivus/**/*.fab files ✅
2. Rivus parses all fons/faber/**/*.ts files ⚠️ (N/A - different language)
3. Rivus parses all test fixtures ⚠️ (depends on test framework)
4. AST matches Faber's AST for equivalent code ✅

---

## 11. Conclusion

The Rivus parser is **largely complete** and successfully implements the core Faber grammar. It can parse itself and supports all fundamental language constructs including:
- Variables, functions, classes, interfaces, enums, tagged unions
- Control flow: if/else, while, for-of, for-in, switch, pattern matching, guards
- Expressions: binary, unary, assignment, ternary, calls, member access
- Advanced features: async/await, lambdas, ranges, collection DSL, I/O

**Missing features are primarily advanced/convenience features:**
1. Test framework (high priority for language development)
2. Resource management (high priority for systems programming)
3. Entry points (high priority for program organization)
4. Type conversions (medium priority, syntactic sugar)
5. Native construction (medium priority, metaprogramming)
6. Compile-time expressions (medium priority, metaprogramming)
7. Dispatch statements (future feature, not blocking)

**The parser gap does not block self-hosting.** Rivus can compile itself and other Faber programs that don't use the missing features.

**Recommended priority for closing gaps:**
1. Entry points (1-2 days) - enables better program organization
2. Type conversions (2-3 days) - frequently used operations
3. Test framework (1-2 weeks) - critical for language development
4. Resource management (2-3 weeks) - important for systems programming
5. Other features as needed

---

## Appendix A: EBNF.md Coverage Matrix

| EBNF Section | Faber | Rivus | Notes |
|--------------|-------|-------|-------|
| **Program Structure** ||||
| program | ✅ | ✅ | |
| statement | ✅ | ✅ | |
| blockStmt | ✅ | ✅ | |
| **Declarations** ||||
| varDecl | ✅ | ✅ | |
| arrayDestruct | ✅ | ✅ | |
| objectDestruct | ✅ | ✅ | |
| funcDecl | ✅ | ✅ | |
| paramList | ✅ | ✅ | |
| typeParamDecl | ✅ | ✅ | |
| parameter | ✅ | ✅ | |
| funcModifier | ✅ | ✅ | |
| returnClause | ✅ | ✅ | |
| lambdaExpr | ✅ | ✅ | |
| genusDecl | ✅ | ✅ | |
| genusMember | ✅ | ✅ | |
| fieldDecl | ✅ | ✅ | |
| methodDecl | ✅ | ✅ | |
| annotation | ✅ | ✅ | |
| stdlibAnnotation | ✅ | ✅ | |
| pactumDecl | ✅ | ✅ | |
| pactumMethod | ✅ | ✅ | |
| typeAliasDecl | ✅ | ✅ | |
| enumDecl | ✅ | ✅ | |
| enumMember | ✅ | ✅ | |
| discretioDecl | ✅ | ✅ | |
| variant | ✅ | ✅ | |
| variantFields | ✅ | ✅ | |
| importDecl | ✅ | ✅ | |
| specifierList | ✅ | ✅ | |
| specifier | ✅ | ✅ | |
| **Types** ||||
| typeAnnotation | ✅ | ✅ | |
| functionType | ✅ | ✅ | |
| typeList | ✅ | ✅ | |
| typeParams | ✅ | ✅ | |
| typeParameter | ✅ | ✅ | |
| arrayBrackets | ✅ | ✅ | |
| **Control Flow** ||||
| ifStmt | ✅ | ✅ | |
| elseClause | ✅ | ✅ | |
| whileStmt | ✅ | ✅ | |
| exStmt | ✅ | ✅ | |
| forBinding | ✅ | ✅ | |
| deStmt | ✅ | ✅ | |
| eligeStmt | ✅ | ✅ | |
| eligeCase | ✅ | ✅ | |
| defaultCase | ✅ | ✅ | |
| discerneStmt | ✅ | ✅ | |
| discriminants | ✅ | ✅ | |
| variantCase | ✅ | ✅ | |
| patterns | ✅ | ✅ | |
| pattern | ✅ | ✅ | |
| patternBind | ✅ | ✅ | |
| guardStmt | ✅ | ✅ | |
| guardClause | ✅ | ✅ | |
| curaStmt | ✅ | ❌ | Missing from Rivus |
| curatorKind | ✅ | ❌ | Missing from Rivus |
| returnStmt | ✅ | ✅ | |
| breakStmt | ✅ | ✅ | |
| continueStmt | ✅ | ✅ | |
| **Error Handling** ||||
| tryStmt | ✅ | ✅ | |
| throwStmt | ✅ | ✅ | |
| catchClause | ✅ | ✅ | |
| assertStmt | ✅ | ✅ | |
| **Expressions** ||||
| expression | ✅ | ✅ | |
| assignment | ✅ | ✅ | |
| ternary | ✅ | ✅ | |
| or | ✅ | ✅ | |
| and | ✅ | ✅ | |
| equality | ✅ | ✅ | |
| comparison | ✅ | ✅ | |
| bitwiseOr | ✅ | ✅ | |
| bitwiseXor | ✅ | ✅ | |
| bitwiseAnd | ✅ | ✅ | |
| shift | ✅ | ✅ | |
| shiftOp | ✅ | ✅ | |
| range | ✅ | ✅ | |
| additive | ✅ | ✅ | |
| multiplicative | ✅ | ✅ | |
| unary | ✅ | ✅ | |
| cast | ✅ | ✅ | |
| conversionOp | ✅ | ❌ | Missing from Rivus |
| call | ✅ | ✅ | |
| callSuffix | ✅ | ✅ | |
| memberSuffix | ✅ | ✅ | |
| optionalSuffix | ✅ | ✅ | |
| nonNullSuffix | ✅ | ✅ | |
| argumentList | ✅ | ✅ | |
| argument | ✅ | ✅ | |
| primary | ✅ | ✅ | |
| newExpr | ✅ | ✅ | |
| fingeExpr | ✅ | ✅ | |
| praefixumExpr | ✅ | ❌ | Missing from Rivus |
| scriptumExpr | ✅ | ✅ | |
| legeExpr | ✅ | ✅ | |
| regexLiteral | ✅ | ✅ | |
| **Patterns** ||||
| objectPattern | ✅ | ✅ | |
| patternProperty | ✅ | ✅ | |
| arrayPattern | ✅ | ✅ | |
| arrayPatternElement | ✅ | ✅ | |
| **Output** ||||
| outputStmt | ✅ | ✅ | |
| **Entry Points** ||||
| incipitStmt | ✅ | ❌ | Missing from Rivus |
| incipietStmt | ✅ | ❌ | Missing from Rivus |
| **Testing** ||||
| probandumDecl | ✅ | ❌ | Missing from Rivus |
| probandumBody | ✅ | ❌ | Missing from Rivus |
| probaStmt | ✅ | ❌ | Missing from Rivus |
| probaModifier | ✅ | ❌ | Missing from Rivus |
| praeparaBlock | ✅ | ❌ | Missing from Rivus |
| **Endpoint Dispatch** ||||
| adStmt | ✅ | ❌ | Missing from Rivus (parse+codegen) |
| adBinding | ✅ | ❌ | Missing from Rivus |
| adBindingVerb | ✅ | ❌ | Missing from Rivus |
| **DSL Transforms** ||||
| dslExpr | ✅ | ✅ | |
| dslTransforms | ✅ | ✅ | |
| dslTransform | ✅ | ✅ | |
| dslVerb | ✅ | ✅ | |
| abExpr | ✅ | ✅ | |
| filter | ✅ | ✅ | |
| **Fac Block** ||||
| facBlockStmt | ✅ | ❌ | Missing from Rivus |
| **Mutation Block** ||||
| inStmt | ✅ | ✅ | |

**Coverage Summary:**
- Total EBNF productions: ~120
- Implemented in both: ~108 (90%)
- Faber only: ~12 (10%)
- Rivus only: 0 (0%)

---

## Appendix B: File Structure Comparison

### Faber Parser (TypeScript)
```
fons/faber/parser/
├── index.ts           # Main parser (3500+ lines)
├── ast.ts             # AST definitions (2800+ lines)
├── errors.ts          # Error types and messages
└── index.test.ts      # Parser tests
```

### Rivus Parser (Faber)
```
fons/rivus/parser/
├── index.fab          # Entry point and Parsator genus
├── nucleus.fab        # Core state and navigation
├── resolvitor.fab     # Parser interface
├── errores.fab        # Error types and messages
├── typus.fab          # Type annotation parsing
├── morphologia.fab    # Morphology tracking
├── expressia/
│   ├── index.fab      # Expression entry point
│   ├── binaria.fab    # Binary operators (precedence climbing)
│   ├── unaria.fab     # Unary operators and casts
│   └── primaria.fab   # Primary expressions (literals, arrays, objects, lambdas)
└── sententia/
    ├── index.fab      # Statement dispatcher
    ├── actio.fab      # Control flow (return/break/continue/throw/output)
    ├── declara.fab    # Declarations (functio/genus/pactum/ordo/discretio/typus)
    ├── error.fab      # Error handling (tempta/adfirma)
    ├── fluxus.fab     # Flow control (elige/discerne/custodi)
    ├── imperium.fab   # Loops and conditionals (si/dum/ex/de/in)
    ├── initus.fab     # Entry points and resource mgmt (incipit/cura/ad)
    ├── massa.fab      # Block statements
    ├── varia.fab      # Variable declarations
    ├── proba.fab      # Test framework (probandum/proba)
    └── annotatio.fab  # Annotation parsing
```

**Observations:**
- Faber: Monolithic (single 3500-line file)
- Rivus: Modular (20 files, max ~800 lines each)
- Rivus uses dependency injection (Resolvitor) to avoid circular imports
- Rivus separates concerns by grammatical category
- Both have similar line counts (~6000 total)

---

**End of Gap Analysis**
