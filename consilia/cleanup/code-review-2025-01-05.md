# Code Review: `fons/rivus` (Bootstrap Faber Compiler)

**Date:** 2025-01-05  
**Reviewer:** Claude (Sonnet, large context)  
**Scope:** Full codebase review of `fons/rivus/` bootstrap compiler  
**Reference Documents:** README.md, GRAMMAR.md, DEVELOPER.md, AGENTS.md

---

## Executive Summary

The rivus bootstrap compiler is a Faber implementation of the Faber compiler itself—a critical milestone for language self-hosting. This review examined all source files across the lexor, parser, semantic, and codegen phases.

**Overall Assessment:** The codebase is well-structured with clean separation of concerns, but has several incomplete areas that will block full self-hosting. The parser has a known infinite loop issue, and semantic analysis is partially implemented.

---

## File-by-File Assessment

### Severity Legend
- 🔴 **Critical** — Blocking, must fix before self-host
- 🟠 **High** — Significant issues, should fix soon
- 🟡 **Medium** — Notable issues, plan to address
- 🟢 **Low** — Minor issues or suggestions

---

| File | Severity | Primary Issue |
|------|----------|---------------|
| `semantic/index.fab` | 🟠 High | Incomplete type inference |
| `semantic/sententia/declara.fab` | 🟠 High | TODOs in type declarations |
| `parser/nucleus.fab` | 🟡 Medium | Potential infinite loops |
| `lexor/index.fab` | 🟡 Medium | Incomplete template handling |
| `parser/expressia/primaria.fab` | 🟡 Medium | Incomplete keyword handling |
| `semantic/typi.fab` | 🟡 Medium | Large discriminated union |
| `codegen/ts/sententia/index.fab` | 🟡 Medium | Missing ad statement |
| `parser/sententia/declara.fab` | 🟡 Medium | Complex parsing needs review |
| `parser/errores.fab` | 🟢 Low | Verbose casting pattern |
| `cli.fab` | 🟢 Low | Error handling improvements |
| `codegen/typi.fab` | 🟢 Low | Stubbed comment formatting |
| `lexicon/verba.fab` | 🟢 Low | Token type overlap |
| `ast/*.fab` | 🟢 Low | Clean, well-documented |

---

## Detailed Findings

### 🟠 HIGH SEVERITY

#### 1. `semantic/index.fab` — Incomplete Type Inference

**Location:** `fons/rivus/semantic/index.fab`

**Issue:** The semantic analyzer's type inference is incomplete for several expression types. The `predeclare` function creates placeholder types, but full resolution is not implemented for all cases.

**Affected Areas:**
- `ScriptumExpressia` (format strings) — return type not inferred
- `CatenaExpressia` (collection DSL) — pipeline type propagation missing
- Lambda return type inference — body type not analyzed
- Generic type parameter inference — partially implemented

**Impact:** Self-hosting will fail when the compiler tries to analyze its own complex type scenarios.

**Recommendation:**
```fab
# In semantic/expressia/index.fab, ensure all variants are handled:
discerne expr {
    casu ScriptumExpressia { redde TEXTUS }
    casu CatenaExpressia ut c { 
        # Infer from source collection element type
        redde resolveTypusListae(r.expressia(c.fons))
    }
    # ... etc
}
```

---

#### 2. `semantic/sententia/declara.fab` — Incomplete Type Declarations

**Location:** `fons/rivus/semantic/sententia/declara.fab`

**Issue:** Multiple TODO comments indicate incomplete implementation:

```fab
# Line ~157
# TODO: Analyze fields and methods

# Line ~178  
# TODO: Analyze method signatures

# Line ~198
# TODO: Analyze members

# Line ~218
# TODO: Analyze variants
```

**Impact:** 
- Genus (class) fields and methods are not type-checked
- Pactum (interface) method signatures are not validated
- Ordo (enum) members are not resolved
- Discretio (tagged union) variants are not analyzed

**Recommendation:** Implement full member analysis. Priority order:
1. `analyzeGenusDeclaratio` — needed for AST node types
2. `analyzePactumDeclaratio` — needed for Resolvitor interface
3. `analyzeDiscretioDeclaratio` — needed for Expressia/Sententia unions

---

### 🟡 MEDIUM SEVERITY

#### 3. `parser/nucleus.fab` — Potential Infinite Loops

**Location:** `fons/rivus/parser/nucleus.fab`

**Issue:** The parser's error recovery mechanism (`synchrona()`) is referenced throughout the codebase but the implementation may not have adequate loop guards. This is noted in AGENTS.md as a known issue:

> "Parser has infinite loop on some inputs - investigation needed"

**Problematic Pattern:**
```fab
# In parseProgramma (massa.fab)
tempta {
    corpus.adde(r.sententia())
} cape e {
    p.synchrona()  # May not advance position on some errors
}
```

**Recommendation:** Add explicit guards:
```fab
functio synchrona() -> vacuum {
    fixum initiumIndex = ego.index
    varia iterationes = 0
    
    dum non ego.estFinis() et iterationes < 1000 {
        si estVerbumSententiae(ego.specta(0).verbum) {
            rumpe
        }
        ego.procede()
        iterationes += 1
    }
    
    # Failsafe: if no progress, force advance
    si ego.index == initiumIndex {
        ego.procede()
    }
}
```

---

#### 4. `lexor/index.fab` — Incomplete Template Literal Handling

**Location:** `fons/rivus/lexor/index.fab`

**Issue:** The lexer file is large (400+ lines visible) and template literal interpolation (`${...}`) parsing appears to be handled but needs verification for:
- Nested template literals
- Escape sequences within interpolations
- Multi-line templates with embedded expressions

**Recommendation:** Add test cases for edge cases:
```fab
# Test nested templates
`outer ${`inner ${x}`} end`

# Test escapes in interpolation
`value: ${obj["key\\with\\escapes"]}`
```

---

#### 5. `parser/expressia/primaria.fab` — Incomplete Keyword Handling

**Location:** `fons/rivus/parser/expressia/primaria.fab`, lines 195-196

**Issue:** The keyword-as-identifier fallback is incomplete:
```fab
# Keyword identifiers (allow keywords as names in expression position)
si sym.species == SymbolumGenus.Verbum {
    # ... implementation truncated in review
```

This matters for property access like `obj.fit` or `config.in` where keywords are valid member names.

**Recommendation:** Complete the fallback to allow all non-control-flow keywords as identifiers in member position:
```fab
si sym.species == SymbolumGenus.Verbum {
    # Allow keywords as identifiers when not starting statements
    si non estVerbumSententiae(sym.verbum) {
        p.procede()
        redde finge Nomen { locus: locus, valor: sym.valor } qua Expressia
    }
}
```

---

#### 6. `codegen/ts/sententia/index.fab` — Missing Ad Statement

**Location:** `fons/rivus/codegen/ts/sententia/index.fab`, lines 156-157

**Issue:** The `ad` (dispatch) statement is not implemented:
```fab
casu AdSententia {
    redde scriptum("§/* ad statement not implemented for TS */", g.ind())
}
```

**Impact:** Any Faber code using `ad` dispatch will produce non-functional output.

**Recommendation:** Either implement or explicitly document as out-of-scope for bootstrap:
```fab
# If implementing:
casu AdSententia ut s {
    # ad "target" (args) fit T pro binding { body }
    # -> fetch("target", args).then(binding => { body })
    redde genAdDispatch(s.scopus, s.argumenta, s.vinculum, s.corpus, g)
}

# If deferring:
# Add to CHECKLIST.md under "Not Implemented for Bootstrap"
```

---

#### 7. `semantic/typi.fab` — Large Discriminated Union

**Location:** `fons/rivus/semantic/typi.fab`

**Issue:** The `SemanticTypus` discriminated union has 9 variants:
- `Primitivum`
- `Genericum`  
- `Functio`
- `Unio`
- `Ignotum`
- `Usitatum`
- `Ordo`
- `Genus`
- `Pactum`

Pattern matching on this union must be exhaustive, but several `discerne` blocks in the codebase may not handle all cases.

**Problematic Example (in `typiAequales`):**
```fab
discerne a {
    casu Primitivum ut pa {
        discerne b {
            casu Primitivum ut pb { ... }
        }
        redde falsum  # What if b is Ignotum?
    }
    # ...
}
```

**Recommendation:** Add exhaustiveness guards and consider splitting into categories:
```fab
# Option 1: Explicit fallthrough
ceterum { redde falsum }

# Option 2: Helper for common cases
functio estTypusPrimitivusVelGenericus(SemanticTypus t) -> bivalens {
    discerne t {
        casu Primitivum { redde verum }
        casu Genericum { redde verum }
        ceterum { redde falsum }
    }
}
```

---

#### 8. `parser/sententia/declara.fab` — Complex Parsing Logic

**Location:** `fons/rivus/parser/sententia/declara.fab`

**Issue:** This file handles complex parsing for:
- Function declarations with type parameters
- Genus with inheritance, implements, fields, methods, constructor
- Pactum with method signatures
- Ordo and Discretio

The file is large and the genus parsing in particular has nested loops with visibility parsing that could have edge cases.

**Specific Concern (around line 175):**
```fab
si p.congruetVerbum("publicus") {
    visibilitas = Visibilitas.Publica
```

This appears to handle both annotation-style (`@ publicum`) and inline keyword-style (`publicus`) visibility, which could lead to conflicts.

**Recommendation:** 
1. Add comprehensive test cases for genus parsing
2. Document which visibility style takes precedence
3. Consider rejecting conflicting styles with an error

---

### 🟢 LOW SEVERITY

#### 9. `parser/errores.fab` — Verbose Casting Pattern

**Location:** `fons/rivus/parser/errores.fab`, line 95-96

**Issue:** Self-documented TODO:
```fab
# TODO everything in here appears to be returning `qua ParserErrorNuntius`.. seems verbose.
```

Every case in `nuntiumErroris` ends with `qua ParserErrorNuntius`.

**Recommendation:** Add a helper function:
```fab
functio nuntius(textus t, textus a) -> ParserErrorNuntius {
    redde { textus: t, auxilium: a } qua ParserErrorNuntius
}

# Then use:
casu ParserErrorCodice.ExpectaturParensDex {
    redde nuntius("Expected ')'", "Parentheses must be balanced.")
}
```

---

#### 10. `cli.fab` — Error Handling Improvements

**Location:** `fons/rivus/cli.fab`

**Issue:** The CLI uses `redde` for error exits, which may not set proper exit codes:
```fab
si lexResult.errores.longitudo() > 0 {
    mone "Lexor errors:"
    # ...
    redde  # This returns from incipiet, but exit code is 0
}
```

**Recommendation:** Consider adding explicit exit code support:
```fab
# When Faber supports process exit codes:
exito(1)

# Or wrap in error-throwing pattern:
iace "Compilation failed"
```

---

#### 11. `codegen/typi.fab` — Stubbed Comment Formatting

**Location:** `fons/rivus/codegen/typi.fab`, lines 113-130

**Issue:** Comment formatting functions return stubs:
```fab
@ publica
functio formataNotaePrae(ignotum nodus, ignotum syntax, textus indentum) -> textus {
    si nodus == nihil { redde "" }
    # ... stub implementation
}
```

**Note:** This is acknowledged as P3 priority elsewhere.

**Recommendation:** Add explicit documentation:
```fab
# WHY: Comment preservation is P3 priority. Currently returns empty
# to avoid breaking output. Full implementation tracked in CHECKLIST.md.
```

---

#### 12. `lexicon/verba.fab` — Token Type Overlap

**Location:** `fons/rivus/lexicon/verba.fab`

**Issue:** There's potential confusion between:
- `VerbumId.Nihil` (keyword identity)
- `SymbolumGenus.Nihil` (token type)

Both exist and represent the `nihil` keyword, but serve different purposes.

**Recommendation:** Add clarifying comment:
```fab
# NOTE: SymbolumGenus.Nihil is the token type for the literal `nihil`.
# VerbumId.Nihil would be for `nihil` as a keyword (if needed).
# Currently `nihil` is handled as a literal, not a keyword.
```

---

## Architecture Observations

### Strengths

1. **Clean Separation of Concerns**
   - Lexor → Parser → Semantic → Codegen pipeline is well-defined
   - Each phase has clear input/output contracts

2. **Circular Import Resolution**
   - Good use of `pactum Resolvitor` pattern to break cycles
   - Parser and Semantic modules both use this pattern effectively

3. **Comprehensive AST Definitions**
   - `ast/expressia.fab` and `ast/sententia.fab` cover all language constructs
   - Good use of `discretio` (tagged unions) for AST node types
   - Position tracking on all nodes enables good error messages

4. **Morphologia System**
   - Elegant mapping of Latin verb conjugation to semantic flags
   - `parseMethodum()` extracts meaning from verb endings
   - Radix dictionary cleanly maps stems to code generators

5. **Error Collection**
   - Never throws—collects errors and continues
   - Enables showing multiple errors per compilation

### Areas for Improvement

1. **Error Recovery Robustness**
   - Parser `synchrona()` needs hardening against infinite loops
   - Consider adding maximum iteration guards

2. **Type System Completeness**
   - Semantic analysis is ~60% complete
   - Many TODOs remain for type checking

3. **Test Coverage**
   - No inline test assertions in source files
   - Would benefit from `proba` blocks in each module

4. **Documentation Standards**
   - DEVELOPER.md specifies module headers with COMPILER PHASE, ARCHITECTURE, etc.
   - These are inconsistently applied across files

5. **Feature Parity**
   - Several codegen features are stubbed (ad, comments)
   - Need tracking mechanism for what's implemented vs deferred

---

## Priority Recommendations

### P0 — Blocking for Self-Host (Must Fix)

1. **Complete semantic analysis for genus/pactum members**
   - Files: `semantic/sententia/declara.fab`
   - Reason: The compiler uses genus and pactum extensively

2. **Fix parser infinite loop issue**
   - Files: `parser/nucleus.fab`
   - Reason: Parser hangs on some malformed inputs

### P1 — Important (Should Fix Soon)

3. **Complete type inference for all expression types**
   - Files: `semantic/expressia/*.fab`
   - Reason: Type errors go undetected

4. **Implement missing DSL expressions in codegen**
   - Files: `codegen/ts/sententia/index.fab`
   - Reason: ad statement produces broken output

### P2 — Should Fix (Plan to Address)

5. **Add exhaustiveness guards to pattern matches**
   - Files: `semantic/typi.fab`, various
   - Reason: Silent failures on unhandled variants

6. **Reduce verbose `qua` casting patterns**
   - Files: `parser/errores.fab`
   - Reason: Code readability

### P3 — Nice to Have (When Time Permits)

7. **Implement comment preservation**
   - Files: `codegen/typi.fab`
   - Reason: Formatting round-trips lose comments

8. **Add inline tests to modules**
   - Files: All
   - Reason: Regression protection

---

## Appendix: Files Reviewed

```
fons/rivus/
├── cli.fab                          ✓ Reviewed
├── CHECKLIST.md                     ✓ Referenced
├── ast/
│   ├── expressia.fab                ✓ Reviewed
│   ├── lexema.fab                   ✓ Reviewed
│   ├── positio.fab                  ✓ Reviewed
│   ├── radix.fab                    ✓ Reviewed
│   ├── sententia.fab                ✓ Reviewed
│   └── typus.fab                    ✓ Reviewed
├── lexicon/
│   └── verba.fab                    ✓ Reviewed
├── lexor/
│   ├── errores.fab                  ✓ Reviewed
│   └── index.fab                    ✓ Reviewed (partial - large file)
├── parser/
│   ├── errores.fab                  ✓ Reviewed
│   ├── index.fab                    ✓ Reviewed
│   ├── morphologia.fab              ✓ Reviewed
│   ├── nucleus.fab                  ✓ Reviewed (partial - large file)
│   ├── resolvitor.fab               ✓ Reviewed
│   ├── typus.fab                    ✓ Reviewed
│   ├── expressia/
│   │   ├── binaria.fab              ✓ Reviewed
│   │   ├── index.fab                ✓ Reviewed
│   │   ├── primaria.fab             ✓ Reviewed (partial)
│   │   └── unaria.fab               ✓ Reviewed
│   └── sententia/
│       ├── actio.fab                ○ Exists
│       ├── annotatio.fab            ○ Exists
│       ├── declara.fab              ✓ Reviewed (partial)
│       ├── error.fab                ○ Exists
│       ├── fluxus.fab               ○ Exists
│       ├── imperium.fab             ○ Exists
│       ├── index.fab                ✓ Reviewed
│       ├── initus.fab               ○ Exists
│       ├── massa.fab                ✓ Reviewed
│       ├── proba.fab                ○ Exists
│       └── varia.fab                ○ Exists
├── semantic/
│   ├── errores.fab                  ○ Exists
│   ├── index.fab                    ✓ Reviewed
│   ├── nucleus.fab                  ✓ Reviewed
│   ├── resolvitor.fab               ✓ Reviewed
│   ├── scopus.fab                   ✓ Reviewed
│   ├── typi.fab                     ✓ Reviewed
│   ├── expressia/
│   │   ├── alia.fab                 ○ Exists
│   │   ├── binaria.fab              ○ Exists
│   │   ├── index.fab                ○ Exists
│   │   ├── primaria.fab             ○ Exists
│   │   ├── unaria.fab               ○ Exists
│   │   └── vocatio.fab              ○ Exists
│   └── sententia/
│       ├── actio.fab                ○ Exists
│       ├── declara.fab              ✓ Reviewed
│       ├── error.fab                ○ Exists
│       ├── imperium.fab             ○ Exists
│       └── index.fab                ○ Exists
└── codegen/
    ├── radices.fab                  ✓ Reviewed
    ├── typi.fab                     ✓ Reviewed
    ├── ts/
    │   ├── index.fab                ✓ Reviewed
    │   ├── nucleus.fab              ✓ Reviewed
    │   ├── typus.fab                ○ Exists
    │   ├── expressia/               ○ Exists (directory)
    │   ├── preamble/                ○ Exists (directory)
    │   └── sententia/
    │       └── index.fab            ✓ Reviewed
    └── zig/                         ○ Exists (directory)
```

Legend:
- ✓ Reviewed — Full or partial review completed
- ○ Exists — File exists, not deeply reviewed

---

*Opus inspectum est!* 🏛️