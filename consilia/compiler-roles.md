# Compiler Role Separation: Faber and Rivus

**Status**: Audit Complete
**Author**: Design discussion 2026-01-12
**Related**: Self-hosting, rivus bootstrap, Issue #112

## Summary

Refocus the two compilers:

- **Faber**: Reference compiler targeting TypeScript only. Rock solid, production quality.
- **Rivus**: Multi-target compiler (TS, Python, Rust, Zig, C++). Written in Faber, eventually self-hosting.

## Rationale

The current approach spreads effort across six codegen targets in faber while simultaneously trying to bootstrap rivus. This leads to:

1. **Diluted quality** - Each target has gaps; none are production-ready
2. **Duplicate work** - Rivus reimplements codegen that faber already has
3. **Unclear ownership** - Which compiler should fix a Rust codegen bug?
4. **Self-hosting blockers** - Rivus can't compile itself due to missing features

The new model:

1. **Faber becomes the gold standard** for TypeScript output
2. **Rivus inherits multi-target responsibility** once self-hosting works
3. **Clear ownership** - TS bugs go to faber, other targets go to rivus
4. **Focused effort** - Fix one thing well instead of six things poorly

---

# AUDIT RESULTS

## Overview

| Component | Faber | Rivus | Parity | Blocking Self-Host? |
|-----------|-------|-------|--------|---------------------|
| Lexer/Tokenizer | 42 token types | 73 token types | Rivus MORE complete | No |
| Parser | Full EBNF | ~90% | Minor gaps | No |
| Semantic Analyzer | 5,424 lines | 3,985 lines | **MAJOR GAPS** | **YES** |
| Codegen Expressions | 24 handlers | ~95% coverage | Minor gaps | Partially |
| Codegen Statements | 24 handlers | ~83% coverage | Missing 4 handlers | Partially |
| Shared Infrastructure | Complete | Partial | Missing capability system | For multi-target |

---

# 1. LEXER/TOKENIZER COMPARISON

## Summary

**Surprising finding**: Rivus lexor is MORE feature-complete than Faber tokenizer.

- **Faber**: 42 token types, minimal approach (pushes complexity to parser)
- **Rivus**: 73 token types, rich approach (simplifies parser logic)

## Token Type Comparison

### Literal Types

| Category | Faber (TS) | Rivus (Faber) | Status |
|----------|-----------|---------------|--------|
| Integer | `NUMBER` | `Numerus` | Equivalent |
| Float | `NUMBER` (same) | `Fractus` (distinct) | **Rivus distinguishes** |
| BigInt | `BIGINT` | `Numerus` with 'n' suffix | Faber has dedicated type |
| String | `STRING` | `Textus` | Equivalent |
| Template | `TEMPLATE_STRING` | `Exemplar` | Equivalent |
| Boolean true | `KEYWORD` | `Verum` (dedicated) | **Rivus has dedicated token** |
| Boolean false | `KEYWORD` | `Falsum` (dedicated) | **Rivus has dedicated token** |
| Null | `KEYWORD` | `Nihil` (dedicated) | **Rivus has dedicated token** |

### Operators - Gaps in Faber

| Operator | Purpose | EBNF Line | Faber | Rivus |
|----------|---------|-----------|-------|-------|
| `=>` | Fat arrow (lambdas) | 45, 270 | Missing | `SagittaCrassa` |
| `...` | Spread/rest | 42, 286, 314 | Missing | `PunctumTer` |
| `?.` | Optional chaining | 282 | Missing | `RogatioPunctum` |
| `!.` | Non-null assertion | 283 | Missing | `NonPunctum` |
| `??` | Nullish coalescing | - | Missing | `RogatioBis` |
| `>>>` | Unsigned right shift | - | Missing | `MaiorTer` |

### Comment Types

| Type | Faber | Rivus |
|------|-------|-------|
| `#` line comment | `COMMENT` | `NotaLinea` |
| `//` line comment | Missing | `NotaLinea` |
| `/* */` block | Missing | `NotaMassa` |
| `/** */` doc | Missing | `NotaDocens` |

**Impact**: Faber cannot tokenize rivus's own source code (uses `//` and `/* */`).

### Dialect Support

- **Faber**: No dialect awareness
- **Rivus**: `Dialectus` enum with `Latinus` and `Anglicus` modes
- **Rivus**: Can tokenize English keywords (`becomes` → `Fit`, `true`/`false`/`null`)

## EBNF Compliance

| Feature | EBNF Line | Faber | Rivus | Status |
|---------|-----------|-------|-------|--------|
| Fat arrow `=>` | 45, 270 | Missing | Has | **Faber non-compliant** |
| Spread `...` | 42, 286, 314 | Missing | Has | **Faber non-compliant** |
| Optional chain `?.` | 282 | Missing | Has | **Faber non-compliant** |
| Non-null `!.` | 283 | Missing | Has | **Faber non-compliant** |
| Range `..` | 267 | Has | Has | Both compliant |
| Shift keywords | 266 | Uses keywords | Has tokens | Both work |

## Error Codes (Identical)

Both implement L001-L006:
- L001: Unterminated string
- L002: Unterminated template
- L003: Unexpected character
- L004: Invalid hex literal
- L005: Invalid binary literal
- L006: Invalid octal literal

## Recommendations

### For Faber (if keeping as reference)
1. Add `//` and `/* */` comment support
2. Add missing operators: `=>`, `...`, `?.`, `!.`, `??`

### For Rivus
1. Document why boolean/null are tokens not keywords
2. Decide on shift operators: tokens vs keywords-only
3. Fix template escape handling (skips backslash instead of preserving it)

---

# 2. PARSER COMPARISON

## Summary

**Rivus parser is ~90% complete** with all core language features for self-hosting.

## Coverage Matrix

### Declarations (All Present)

| Declaration | Faber | Rivus | Notes |
|-------------|-------|-------|-------|
| Variable (`varia`/`fixum`) | Yes | Yes | Full support |
| Function (`functio`) | Yes | Yes | Async, generators |
| Class (`genus`) | Yes | Yes | Fields, methods, inheritance |
| Interface (`pactum`) | Yes | Yes | Method signatures |
| Enum (`ordo`) | Yes | Yes | Named members |
| Union (`discretio`) | Yes | Yes | Tagged variants |
| Type alias (`typus`) | Yes | Yes | Generic support |

### Control Flow (All Present)

| Statement | Faber | Rivus | Notes |
|-----------|-------|-------|-------|
| If/else (`si`/`secus`) | Yes | Yes | With inline catch |
| While (`dum`) | Yes | Yes | With catch support |
| For-of/in (`ex`/`pro`) | Yes | Yes | Range, async iteration |
| Switch value (`elige`) | Yes | Yes | Value matching |
| Pattern match (`discerne`) | Yes | Yes | Multi-discriminant |
| Guard (`custodi`) | Yes | Yes | Defer-like cleanup |
| Try/catch (`tempta`/`cape`) | Yes | Yes | Full support |

### Expressions (All Present)

| Expression | Faber | Rivus | Notes |
|------------|-------|-------|-------|
| Binary operators | Yes | Yes | 17 precedence levels |
| Unary operators | Yes | Yes | Prefix and postfix |
| Member access | Yes | Yes | Dot and bracket |
| Function calls | Yes | Yes | With optional chaining |
| Lambdas | Yes | Yes | Arrow functions |
| Type operations | Yes | Yes | `est`, `qua`, `innatum` |
| Collection DSL | Yes | Yes | `ex`/`ab` expressions |

## Critical Gaps in Rivus

### P0 - Function Type Annotations (CRITICAL)

**EBNF specifies**: `(T, U) -> V` syntax for higher-order functions

**Faber has**:
```typescript
interface TypeAnnotation {
    parameterTypes?: TypeAnnotation[];  // For (T, U) -> V
    returnType?: TypeAnnotation;
}
```

**Rivus missing**: No support for function type annotations.

**Location**: `fons/rivus/ast/typus.fab:38-46`

**Impact**: Blocks callbacks, map/filter/reduce, functional patterns.

**Example that fails**:
```fab
functio map(lista<T> items, (T) -> U transform) -> lista<U>
```

### P0 - Shift Operator Keywords (BUG)

**EBNF specifies**: Keywords `sinistratum` (left shift), `dextratum` (right shift)

**Faber**: Correctly uses keywords per EBNF

**Rivus BUG**: Parser checks for operator symbols (`<<`, `>>`) instead of keywords

**Location**: `fons/rivus/parser/expressia/binaria.fab:336-359`

**Impact**: Grammar non-compliance, code using keyword syntax will fail.

## Minor Gaps in Rivus

### P1 - Discerne Default Case

**EBNF allows**: Explicit `ceterum` default case

**Rivus lacks**: `defaultCase` field in DiscerneStatement

**Workaround**: Use wildcard pattern `casu _ { ... }`

### P1 - Template Literal Interpolation

Both implementations incomplete. Rivus actually has better AST structure with separate `partes`/`expressiae` arrays.

## Missing Features (Not Critical for Self-Hosting)

| Feature | Priority | Notes |
|---------|----------|-------|
| Test framework (`probandum`/`proba`) | High | Needed for development |
| Resource management (`cura`) | High | RAII-style cleanup |
| Entry points (`incipit`/`incipiet`) | High | Explicit main functions |
| Dispatch statements (`ad`) | Medium | Syscall/HTTP |
| Compile-time expressions (`praefixum`) | Medium | Metaprogramming |
| Type conversions (`numeratum`/`textatum`) | Medium | Syntactic sugar |
| Native construction (`innatum`) | Medium | Explicit native types |
| Fac blocks | Low | Explicit scope |
| Ex-destructuring | Low | Alternative syntax |

## Operator Precedence

Both compilers implement identical 17-level precedence (verified).

---

# 3. SEMANTIC ANALYZER COMPARISON

## Summary

**This is the critical blocker for self-hosting.**

- **Faber**: 5,424 lines across 6 files
- **Rivus**: 3,985 lines across 18 files

Rivus is missing several critical semantic analysis features.

## Type System Comparison

### Primitives (Parity)

Both support: `textus`, `numerus`, `fractus`, `decimus`, `magnus`, `bivalens`, `nihil`, `vacuum`, `octeti`

Both support sized numeric types: `numerus<32>`, `numerus<i32>`

### Generics (Parity)

Both support: `lista<T>`, `tabula<K,V>`, `promissum<T>`, `cursor<T>`, `fluxus<T>`

### Structured Types

| Type | Faber | Rivus | Status |
|------|-------|-------|--------|
| Genus (class) | Fields, methods, statics | Fields, methods, statics | Parity |
| Pactum (interface) | Method signatures | Methods | Parity |
| Ordo (enum) | Named members | Members | Parity |
| **Discretio (union)** | **Variant info with ordered field lists** | **Placeholder only** | **GAP** |
| **Namespace** | **Wraps norma registry** | **Missing** | **GAP** |

### Union Types (Parity)

Both support `A | B | C` union syntax.

### Function Types (Parity)

Both track: parameters, return type, async flag, curator param.

## Critical Gap 1: DiscretioType Variant Information

**Faber has**:
```typescript
interface VariantInfo {
    fields: { name: string; type: SemanticType }[];
}

interface DiscretioType {
    kind: 'discretio';
    name: string;
    variants: Map<string, VariantInfo>;  // Ordered field list per variant
}
```

**Rivus missing**: DiscretioType variant is not in type system. Discretio declarations are predeclared as `usitatumTypus` (user type placeholder) but never resolved to concrete DiscretioType with variant field information.

**Impact**: Pattern matching in `discerne` cannot infer types for variant field bindings.

```fab
discerne result {
    casu Ok ut value { /* value type is IGNOTUM (unknown) */ }
}
```

**Fix Required**:

1. Add `Discretio` variant to `SemanticTypus` discretio:
```fab
Discretio {
    textus nomen
    tabula<textus, VariantInfo> variantes
    bivalens nullabilis
}

genus VariantInfo {
    lista<CampusInfo> campi
}

genus CampusInfo {
    textus nomen
    SemanticTypus typus
}
```

2. Update `modulus.extraheExporta()` to extract variant info
3. Update pattern matching analyzer to infer binding types

**Location**: `fons/rivus/semantic/typi.fab`, `modulus.fab`

## Critical Gap 2: User Type Reconciliation

**Faber has** (types.ts:398-434):
```typescript
// WHY: `user` is a nominal reference to a declared type
if (a.kind === 'user') {
    if ((b.kind === 'enum' || b.kind === 'genus' || ...) && a.name === b.name) {
        return true;
    }
}
```

**Rivus missing**: No reconciliation logic in `typiAequales()`.

**Impact**: Cross-module types don't work:
```fab
ex "./parser" importa ParserResult  // Gets usitatumTypus("ParserResult")
fixum r = parse(...)                // Returns genusTypus("ParserResult", fields...)
r.errors.longitudo()                // FAILS: r is usitatumTypus, fields unknown
```

**Fix Required**: Add reconciliation in `fons/rivus/semantic/typi.fab:225-315`:
```fab
discerne a {
    casu Usitatum ut ua {
        discerne b {
            casu Genus ut gb { si ua.nomen == gb.nomen { redde verum } }
            casu Pactum ut pb { si ua.nomen == pb.nomen { redde verum } }
            casu Discretio ut db { si ua.nomen == db.nomen { redde verum } }
            casu _ { }
        }
    }
    casu _ { }
}
```

## Critical Gap 3: Namespace Symbol Support

**Faber has**: `'namespace'` symbol kind for norma wildcard imports
```typescript
ex "norma/json" importa * ut json  // Creates namespace symbol
json.solve(data)  // Validates method against norma registry
```

**Rivus missing**: No namespace symbol kind in `SymbolumSpecies` ordo.

**Location**: `fons/rivus/semantic/scopus.fab`

**Impact**: Norma wildcard imports cannot be properly typed.

**Fix Required**:
1. Add `Namespace` to `SymbolumSpecies` ordo
2. Add namespace import handling in predeclaration phase
3. Add namespace method call validation (query norma registry)

## Critical Gap 4: Analysis Phases

**Faber uses 3-phase approach**:
1. Phase 1a - Predeclaration: Register all top-level names with placeholder types
2. Phase 1b - Signature Resolution: Resolve type annotations, update placeholders
3. Phase 2 - Body Analysis: Analyze function bodies, expressions, statements

**Rivus uses 2-phase approach**:
1. Phase 1 - Predeclaration: Register placeholder types
2. Phase 2 - Analysis: Analyze statements

**Missing**: Phase 1b signature resolution.

**Impact**: Forward function calls may fail type checks because callee signature is still `(IGNOTUM...) -> IGNOTUM`.

**Location**: `fons/rivus/semantic/index.fab:60-76`

**Fix Required**: Add signature resolution phase between predeclaration and body analysis.

## Critical Gap 5: Module Export Resolution

**Faber uses 3-pass extraction** (modules.ts):
1. Extract genus/ordo/discretio types
2. Re-extract genus with full type context (nested references)
3. Extract all exports using complete type context

**Rivus uses 1-pass extraction**:
- Genus fields resolved to `IGNOTUM` or shallow types
- Discretio exported as placeholder `usitatumTypus`, not `discretioTypus` with variant info
- No intra-module type references

**Location**: `fons/rivus/semantic/modulus.fab`

**Impact**: Cross-file pattern matching impossible, field access unreliable.

## Medium Gap: Array Element Validation

**Faber validates**: Array elements match inferred type, reports errors

**Rivus missing**: No validation - heterogeneous arrays silently accepted

**Location**: `fons/rivus/semantic/expressia/primaria.fab:99-107`

## Feature Parity Summary

| Feature | Faber | Rivus | Status |
|---------|-------|-------|--------|
| Primitives | Full | Full | PARITY |
| Generics | Full | Full | PARITY |
| Genus Types | Fields+Methods | Fields+Methods | PARITY |
| Pactum Types | Methods | Methods | PARITY |
| **Discretio Types** | **Variant Info** | **Placeholder** | **GAP** |
| **Namespace Types** | **norma/*** | **Missing** | **GAP** |
| Union Types | Full | Full | PARITY |
| **Type Equality** | **w/ Reconciliation** | **No Reconciliation** | **GAP** |
| Assignability | Full | Full | PARITY |
| Scope Management | Full | Full | PARITY |
| Variable Inference | Full | Full | PARITY |
| **Array Inference** | **w/ Validation** | **No Validation** | **GAP** |
| Loop Var Inference | Full | Full | PARITY |
| **Pattern Inference** | **From Variants** | **Always IGNOTUM** | **GAP** |
| **Module Resolution** | **3-pass** | **1-pass** | **GAP** |
| Error Reporting | 14 codes | 12+ codes | PARITY |
| **Analysis Phases** | **3-phase** | **2-phase** | **GAP** |

---

# 4. CODEGEN EXPRESSIONS COMPARISON

## Summary

- **Faber**: 24 separate handler files (modular approach)
- **Rivus**: 6 files with most handlers inline in `index.fab` (consolidated)

**Coverage**: ~95% with critical gaps in namespace/module calls.

## Handler Inventory

| Expression Type | Faber Handler | Rivus Handler | Status |
|-----------------|---------------|---------------|--------|
| Identifier | identifier.ts | index.fab | Parity |
| Literal | literal.ts | index.fab | Parity |
| Binary | binary.ts | index.fab | Parity |
| Unary | unary.ts | index.fab | Parity |
| Member access | member.ts | index.fab | Parity |
| Function call | call.ts | index.fab | Parity |
| Optional chaining | optional-call.ts | index.fab | Parity |
| Assignment | assignment.ts | index.fab | Parity |
| Lambda | lambda.ts | index.fab | Parity |
| Conditional | conditional.ts | index.fab | Parity |
| Array literal | array.ts | index.fab | Parity |
| Object literal | object.ts | index.fab | Parity |
| Type cast (`qua`) | qua.ts | index.fab | Parity |
| Type check (`est`) | est.ts | index.fab | Parity |
| Construction (`innatum`) | innatum.ts | index.fab | Parity |
| Range (`..`) | range.ts | index.fab | Parity |
| Spread (`...`) | spread.ts | index.fab | Parity |
| Template literal | template.ts | index.fab | Parity |
| Yield (`cede`) | cede.ts | index.fab | Parity |
| **Namespace call** | namespace.ts | **MISSING** | **GAP** |
| **Norma module call** | norma-namespace.ts | **MISSING** | **GAP** |
| Computed optional call | computed-optional.ts | **Partial** | Minor gap |
| Non-null call | non-null-call.ts | **MISSING** | Minor gap |

## Critical Gap 1: Namespace Calls

**Purpose**: Handles `solum.lege()`, `solum.scribe()` patterns where a norma module is imported as namespace.

**Faber** (`fons/faber/codegen/shared/norma-namespace.ts`):
```typescript
export function getNamespaceTranslation(callee: MemberExpression, target: string) {
    const moduleName = callee.object.resolvedType.moduleName;
    const methodName = (callee.property as Identifier).name;
    return getNormaTranslation(target, moduleName, methodName);
}
```

**Rivus**: No equivalent handler.

**Impact**: `solum.lege(path)` fails to translate to `fs.readFileSync(path)`.

## Critical Gap 2: Norma Module Calls

**Purpose**: Handles bare function calls to norma modules like `pavimentum(x)` → `Math.floor(x)`.

**Faber**: Detects norma-imported functions and applies translations.

**Rivus**: No handling for bare norma function calls.

**Impact**: `pavimentum(3.7)` emits `pavimentum(3.7)` instead of `Math.floor(3.7)`.

## Minor Gap: Tabula Optional Computed Call

**Pattern**: `m?[k]()`

**Expected**: `(m?.get(k))?.()`

**Rivus may emit**: Incorrect chaining order.

## Minor Gap: QuaExpression Edge Cases

**Pattern**: Object literal cast to class `{...} qua Genus`

**Faber**: Special cases for genus instantiation → `new Genus({...})`

**Rivus**: Simple type assertion for all cases.

**Note**: Source should use `innatum` instead (Issue #112).

## Strengths Comparison

**Faber strengths**:
- Robust error handling and validation
- Explicit morphology validation before translation
- Feature flag tracking for import optimization

**Rivus strengths**:
- More concise (one dispatch file vs many)
- Has `CedeExpressia` for generator/async yield
- Template system with § placeholders works well

---

# 5. CODEGEN STATEMENTS COMPARISON

## Summary

**Coverage**: ~83% (20/24 handlers)

## Handler Inventory

| Statement Type | Faber Handler | Rivus Handler | Status |
|----------------|---------------|---------------|--------|
| **DECLARATIONS** |
| Variable/constant | varia.ts | varia.ts | Parity |
| Function | functio.ts | functio.ts | Parity |
| Class | genus.ts | genus.ts | Parity |
| Type alias | typealias.ts | typealias.ts | Parity |
| Interface | pactum.ts | pactum.ts | Parity |
| Enum/Union | discretio.ts | discretio.ts | Parity |
| Array destructure | destructure.ts | **MISSING** | **GAP** |
| **CONTROL FLOW** |
| Conditional | si.ts | si.ts | Parity |
| While loop | dum.ts | dum.ts | Parity |
| For/iteration | iteratio.ts | iteratio.ts | Parity |
| Match value | elige.ts | elige.ts | Parity |
| Match variant | discerne.ts | discerne.ts | Parity |
| **ERROR HANDLING** |
| Try/catch | tempta.ts | tempta.ts | Parity |
| Throw/panic | iace.ts | iace.ts | Parity |
| **FLOW CONTROL** |
| Return | redde.ts | redde.ts | Parity |
| Continue | perge.ts | **MISSING** | **GAP** |
| Break | rumpe.ts | **MISSING** | **GAP** |
| **IMPORTS/EXPORTS** |
| Import | importa.ts | importa.ts | Parity |
| Assert | adfirma.ts | adfirma.ts | Parity |
| **SPECIAL** |
| Entry point (sync) | incipit.ts | incipit.ts | Parity |
| Entry point (async) | incipiet.ts | **MISSING** | **GAP** |
| Context block | in.ts | in.ts | Parity |
| Guard/custody | custodi.ts | custodi.ts | Parity |
| Care | cura.ts | cura.ts | Parity |
| Do statement | fac.ts | fac.ts | Parity |
| Order | ordo.ts | ordo.ts | Parity |
| Log/debug | scribe.ts | scribe.ts | Parity |
| **TESTING** |
| Test case | proba.ts | proba.ts | Parity |
| Test suite | probandum.ts | **MISSING** | **GAP** |

## Missing Handler 1: perge.ts (Continue)

```fab
perge → continue;
```

**Impact**: Loop continuation control missing.

## Missing Handler 2: rumpe.ts (Break)

```fab
rumpe → break;
```

**Impact**: Loop/switch break control missing.

## Missing Handler 3: incipiet.ts (Async Entry Point)

```fab
incipiet { body }
→ (async () => { body })()
```

**Purpose**: Wraps top-level code in async IIFE for top-level await.

**Impact**: Top-level await not supported in rivus-compiled code.

## Missing Handler 4: probandum.ts (Test Suite)

```fab
probandum "Tokenizer" {
    praepara { lexer = init() }
    proba "test" { ... }
}
→ describe("Tokenizer", () => {
    beforeEach(() => { lexer = init(); });
    test("test", () => { ... });
});
```

**Impact**: Test organization and setup/teardown not available.

## Missing Handler 5: Array Destructuring

```fab
fixum [a, b, ceteri rest] = coords
→ const [a, b, ...rest] = coords

fixum [_, b, _] = values
→ const [, b, ] = values
```

**Note**: Object destructuring handled via `importa.ts` as `ex obj importa {a, b}`. Array destructuring may be handled via different AST path.

## Flumina Protocol (Both Compilers)

Both fully implement the flumina protocol:
- `fit` → `asFit(function* () { ... })` (sync single-value)
- `fiet` → `asFiet(async function* () { ... })` (async single-value)
- `fiunt` → `asFiunt((function* () { ... })())` (sync multi-value)
- `fient` → `asFient((async function* () { ... })())` (async multi-value)

Internal `redde`/`iace` emit `yield respond.ok()`/`yield respond.error()`.

---

# 6. SHARED INFRASTRUCTURE AUDIT

## Overview

| Component | Location | Used By | Status | Migration |
|-----------|----------|---------|--------|-----------|
| Norma stdlib | `fons/norma/*.fab` | Both | Working | Stays shared |
| Registry generator | `scripta/build-norma.ts` | Build | Working | Stays shared |
| Faber registry | `fons/norma/index.json` | Faber | Working | Faber-specific |
| Rivus registry | `fons/rivus/codegen/norma-registry.gen.fab` | Rivus | **Verify integration** | Rivus-specific |
| Test infrastructure | `fons/proba/` | Both | Working | Stays shared |
| Capability system | `fons/faber/codegen/capabilities.ts` | Faber only | **Missing in rivus** | **Migrate** |
| Feature detector | `fons/faber/codegen/feature-detector.ts` | Faber only | **Missing in rivus** | **Migrate** |

## Norma Stdlib Definitions

**Location**: `fons/norma/*.fab` (15 files)

**Collections**: aleator, arca, caelum, copia, fractus, json, lista, mathesis, numerus, solum, tabula, tempus, textus, toml, yaml

**Each file defines**:
- Type mappings via `@ innatum` annotations
- Method translations via `@ verte` annotations (target-specific)
- Morphology metadata via `@ radix` annotations (Latin verb forms)

**Example** from `lista.fab`:
```fab
@ radix add, imperativus, perfectum
@ verte ts "push"
@ verte py "append"
@ verte rs "push"
@ verte cpp "push_back"
@ verte zig (ego, elem, alloc) -> "§0.adde(§2, §1)"
@ externa
functio adde(T elem) -> vacuum
```

**Decision**: STAYS SHARED - Single source of truth for both compilers.

## Registry Generation

**Script**: `scripta/build-norma.ts` (408 lines)

**Generates**:
1. `fons/norma/index.json` - Flat key structure for faber
2. `fons/rivus/codegen/norma-registry.gen.fab` - Nested elige for rivus

**Uses faber's parser** to extract annotations from `.fab` files.

**Decision**: STAYS SHARED - Build tool, not runtime component.

## Rivus Registry Integration

**Critical check needed**:
```bash
rg "getNormaTranslation" fons/rivus/ --glob '!*.gen.fab'
```

If no matches, rivus isn't using the registry yet.

**Action**: Verify integration, implement if missing.

## Test Infrastructure

**Location**: `fons/proba/`

**Components**:
- `harness/schema.ts` - SQLite schema for test results
- `harness/runner.ts` - Main test runner
- `harness/report.ts` - Feature matrix generator
- `shared.ts` - Common utilities
- `faber.test.ts` - Bun test runner for faber
- `rivus.test.ts` - Bun test runner for rivus
- `rivus-compile.ts` - Subprocess isolation for rivus

**Architecture**:
- YAML test files define expectations per target
- `Compiler` type: `'faber' | 'rivus' | 'artifex'`
- Results stored in SQLite with compiler tag
- Can compare same test across compilers

**Decision**: STAYS SHARED - Tests define language semantics.

## Capability System (MISSING IN RIVUS)

**Faber files**:
- `capabilities.ts` (240 lines) - Target capability matrix
- `feature-detector.ts` (462 lines) - AST feature detection

**Defines** `TARGET_SUPPORT` matrix for 6 targets:
```typescript
ts: {
    controlFlow: { asyncFunction: 'supported', generatorFunction: 'supported' },
    errors: { tryCatch: 'supported', throw: 'supported' },
    binding: { pattern: { object: 'supported' } },
    params: { defaultValues: 'supported' },
},
zig: {
    controlFlow: { asyncFunction: 'unsupported', generatorFunction: 'unsupported' },
    errors: { tryCatch: 'emulated', throw: 'emulated' },
    binding: { pattern: { object: 'emulated' } },
    params: { defaultValues: 'unsupported' },
},
```

**Rivus status**: ZERO references to capability validation.

**Impact**: Rivus will silently generate broken code for unsupported features.

**Decision**: MUST MIGRATE TO RIVUS for multi-target support.

**Migration plan**:
1. Create `fons/rivus/codegen/capacitas.fab`
2. Implement `getTargetCapacitas(textus target) -> TargetCapacitas`
3. Port feature detector (AST visitor)
4. Add validation in semantic phase

---

# PRIORITY ACTION ITEMS

## For Self-Hosting (In Order)

### P0 - Semantic Analyzer Gaps

1. **DiscretioType variant info** (1-2 days)
   - Add Discretio variant to SemanticTypus
   - Update module export extraction
   - Update pattern matching analyzer
   - Location: `fons/rivus/semantic/typi.fab`, `modulus.fab`

2. **User type reconciliation** (0.5 days)
   - Add reconciliation logic to `typiAequales()`
   - Location: `fons/rivus/semantic/typi.fab:225-315`

3. **Signature resolution phase** (1 day)
   - Add Phase 1b between predeclaration and body analysis
   - Location: `fons/rivus/semantic/index.fab`

### P1 - Parser Gaps

4. **Function type annotations** (1 day)
   - Add `parameterTypes`/`returnType` to TypeAnnotation
   - Location: `fons/rivus/ast/typus.fab`

5. **Fix shift operator keywords** (0.5 days)
   - Change from `<<`/`>>` symbols to `sinistratum`/`dextratum` keywords
   - Location: `fons/rivus/parser/expressia/binaria.fab:336-359`

### P2 - Codegen Gaps

6. **Namespace calls** (0.5 days)
   - Add handler for `solum.lege()` patterns
   - Location: `fons/rivus/codegen/ts/expressia/`

7. **Missing statements** (1 day)
   - `perge` (continue)
   - `rumpe` (break)
   - `incipiet` (async entry)
   - Location: `fons/rivus/codegen/ts/sententia/`

## For Multi-Target (After Self-Hosting)

### P3 - Capability System Migration

8. **Port capabilities.ts** (2 days)
   - Create `fons/rivus/codegen/capacitas.fab`
   - Create `fons/rivus/semantic/detector.fab`
   - Integrate into validation phase

---

# SUCCESS CRITERIA

1. **Rivus self-hosts**: `build:artifex` produces working compiler
2. **Faber = TS-only reference**: Rock solid TypeScript output
3. **Rivus = multi-target**: Takes over py/rs/zig/cpp codegen
4. **Test parity**: Same YAML tests pass on both compilers
5. **Clear ownership**: TS bugs → faber, other targets → rivus

---

# OPEN QUESTIONS

1. Should faber keep non-TS codegen as "reference implementations" even if unused?
2. How do we handle norma methods that only make sense for certain targets?
3. What's the versioning story? (faber 1.0 = TS-only, rivus 1.0 = multi-target?)
4. Should the capability matrix be shared data (YAML) or compiler-specific code?
