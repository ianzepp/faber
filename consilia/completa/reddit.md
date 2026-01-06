---
status: completed
updated: 2026-01-06
note: reddit keyword fully implemented as syntactic sugar for ergo redde
implemented:
  - reddit keyword in lexicon
  - Parser support in all contexts (si, dum, ex/de loops, elige, discerne, incipit/incipiet, custodi)
  - All targets (ts, py, rs, cpp, zig)
  - Comprehensive test coverage
tests: 8 tests covering si, elige, loops, discerne, incipit, sin/secus chains
---

# Reddit Keyword - Therefore-Return Syntax

## Overview

The `reddit` keyword provides syntactic sugar for `ergo redde` - a single keyword meaning "therefore return this expression."

## Motivation

The most common pattern in `elige` statements and early-return functions is:

```fab
# Current (verbose)
elige nomen {
    casu "textus" { redde TEXTUS }
    casu "numerus" { redde NUMERUS }
}

# With ergo (still verbose)
elige nomen {
    casu "textus" ergo redde TEXTUS
    casu "numerus" ergo redde NUMERUS
}

# With reddit (optimal)
elige nomen {
    casu "textus" reddit TEXTUS
    casu "numerus" reddit NUMERUS
}
```

**LLM benefit**: `reddit` gives "a sense of finality" - the line clearly ends with a return, no block scanning needed.

## Etymology

**reddit** - Latin "it returns" (3rd person present indicative of _reddere_)

- Matches existing pattern: `fit/fiet/fiunt/fient` are 3rd person verb forms
- Authentic Latin conjugation (not invented syntax)
- Single token, clean parse
- 6 chars vs `ergo redde` (11 chars) - 45% shorter

## Grammar

Anywhere `ergo` accepts a statement, `reddit` accepts an expression and implicitly wraps it in a return:

```ebnf
# Before
ifStmt := 'si' expression (blockStmt | 'ergo' statement)
whileStmt := 'dum' expression (blockStmt | 'ergo' statement)
forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement)
eligeCase := 'casu' expression (blockStmt | 'ergo' statement)
variantCase := 'casu' IDENTIFIER bindingClause? (blockStmt | 'ergo' statement)
incipitStmt := 'incipit' (blockStmt | 'ergo' statement)
incipietStmt := 'incipiet' (blockStmt | 'ergo' statement)

# After
ifStmt := 'si' expression (blockStmt | 'ergo' statement | 'reddit' expression)
whileStmt := 'dum' expression (blockStmt | 'ergo' statement | 'reddit' expression)
forBinding := ('pro' | 'fit' | 'fiet') IDENTIFIER (blockStmt | 'ergo' statement | 'reddit' expression)
eligeCase := 'casu' expression (blockStmt | 'ergo' statement | 'reddit' expression)
variantCase := 'casu' IDENTIFIER bindingClause? (blockStmt | 'ergo' statement | 'reddit' expression)
incipitStmt := 'incipit' (blockStmt | 'ergo' statement | 'reddit' expression)
incipietStmt := 'incipiet' (blockStmt | 'ergo' statement | 'reddit' expression)
```

## Usage Examples

### 1. Elige Cases (Primary Use Case)

```fab
functio getType(textus nomen) fit TypeGenus {
    elige nomen {
        casu "textus" reddit TEXTUS
        casu "numerus" reddit NUMERUS
        casu "fractus" reddit FRACTUS
        casu "bivalens" reddit BIVALENS
        ceterum reddit IGNOTUM
    }
}
```

### 2. Early Returns in Si Statements

```fab
functio classify(numerus x) fit textus {
    si x < 0 reddit "negative"
    si x == 0 reddit "zero"
    reddit "positive"
}

functio divide(numerus a, numerus b) fit numerus? {
    si b == 0 reddit nihil
    reddit a / b
}
```

### 3. Sin/Secus Chains

```fab
functio grade(numerus score) fit textus {
    si score >= 90 reddit "A"
    sin score >= 80 reddit "B"
    sin score >= 70 reddit "C"
    sin score >= 60 reddit "D"
    secus reddit "F"
}
```

### 4. Loop Early Exit

```fab
functio findFirst(lista<numerus> items, numerus target) fit numerus? {
    ex items pro item {
        si item == target reddit item
    }
    reddit nihil
}

functio hasKey(tabula<textus, numerus> obj, textus key) fit bivalens {
    de obj pro k {
        si k == key reddit verum
    }
    reddit falsum
}
```

### 5. Discerne Cases

```fab
functio unwrap(Option<T> opt) fit T {
    discerne opt {
        casu Some pro value reddit value
        casu None ergo iace("Cannot unwrap None")
    }
}
```

### 6. Entry Point Exit Code

```fab
incipit reddit 0  # Success exit code

incipit {
    si args.longitudo() < 2 reddit 1  # Error exit
    processArgs(args)
    reddit 0
}
```

## Implementation

### Core Insight

**No AST, semantic, or codegen changes needed!** `reddit` is pure syntactic sugar that desugars during parsing:

```
reddit <expression>  →  ReddeStatement { argument: <expression> }
```

The parser transforms `reddit X` into a `ReddeStatement` AST node immediately, so the rest of the compiler sees normal return statements.

### Phase 1: Keyword Registration

#### fons/ (TypeScript Compiler)

**File**: `fons/lexicon/keywords.ts` (line ~97)

```typescript
{ latin: 'redde', meaning: 'return', category: 'control' },
{ latin: 'reddit', meaning: 'then return', category: 'control' },  // NEW
{ latin: 'custodi', meaning: 'guard', category: 'control' },
```

#### fons-fab/ (Bootstrap Compiler)

**File**: `fons-fab/lexicon/verba.fab` (line ~28)

```fab
casu "redde" { redde verum }
casu "reddit" { redde verum }  # NEW
casu "custodi" { redde verum }
```

### Phase 2: Parser Changes

Both compilers need identical changes in 8 locations. Pattern:

```typescript
// Current (supports ergo)
if (matchKeyword('ergo')) {
    const stmt = parseStatement();
    body = wrapInBlock(stmt);
}

// Add (support reddit)
else if (matchKeyword('reddit')) {
    const expr = parseExpression(); // Parse expression, not statement
    const returnStmt = makeReddeStatement(expr); // Wrap in return
    body = wrapInBlock(returnStmt);
}
```

#### Locations Requiring Changes

##### 1. Si Statement (if/else-if)

**fons/ file**: `fons/parser/index.ts` (~line 2257)

**Current**:

```typescript
if (matchKeyword('ergo')) {
    const stmtPos = peek().position;
    const stmt = parseStatement();
    consequent = { type: 'BlockStatement', body: [stmt], position: stmtPos };
}
```

**Add after**:

```typescript
else if (matchKeyword('reddit')) {
    const stmtPos = peek().position;
    const expr = parseExpression();
    const returnStmt: ReddeStatement = {
        type: 'ReddeStatement',
        argument: expr,
        position: stmtPos
    };
    consequent = { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
}
```

**fons-fab/ file**: `fons-fab/parser/sententia/*.fab` (si statement parser)

**Pattern**: Same logic in Faber syntax

##### 2. Dum Statement (while loop)

**fons/ file**: `fons/parser/index.ts` (~line 2315)

**Add**: Same `reddit` handling as si statement

##### 3. Ex...pro Statement (iteration)

**fons/ file**: `fons/parser/index.ts` (~line 2418)

**Add**: Same `reddit` handling

##### 4. De...pro Statement (key iteration)

**fons/ file**: `fons/parser/index.ts` (~line 2683)

**Add**: Same `reddit` handling

##### 5. Elige Cases (switch cases)

**fons/ file**: `fons/parser/index.ts` (~line 2764)

**Current**:

```typescript
function parseCasuBody(): BlockStatement {
    if (matchKeyword('ergo')) {
        const stmtPos = peek().position;
        const stmt = parseStatement();
        return { type: 'BlockStatement', body: [stmt], position: stmtPos };
    }
    return parseBlockStatement();
}
```

**Add**:

```typescript
function parseCasuBody(): BlockStatement {
    if (matchKeyword('ergo')) {
        const stmtPos = peek().position;
        const stmt = parseStatement();
        return { type: 'BlockStatement', body: [stmt], position: stmtPos };
    }
    if (matchKeyword('reddit')) {
        // NEW
        const stmtPos = peek().position;
        const expr = parseExpression();
        const returnStmt: ReddeStatement = {
            type: 'ReddeStatement',
            argument: expr,
            position: stmtPos,
        };
        return { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
    }
    return parseBlockStatement();
}
```

**fons-fab/ file**: `fons-fab/parser/sententia/fluxus.fab` (~line 67)

**Current**:

```fab
si p.probaVerbum("ergo") {
    p.procede()
    consequens = r.sententia()
}
```

**Add**:

```fab
si p.probaVerbum("ergo") {
    p.procede()
    consequens = r.sententia()
} sin p.probaVerbum("reddit") {  # NEW
    p.procede()
    fixum expr = r.expressia()
    fixum returnStmt = finge ReddeSententia {
        locus: p.specta(-1).locus,
        valor: expr
    } qua Sententia
    consequens = returnStmt
}
```

##### 6. Discerne Cases (pattern matching)

Same pattern - add `reddit` support alongside `ergo`

##### 7. Incipit (entry point)

**fons/ file**: `fons/parser/index.ts` (~line 3548)

**fons-fab/ file**: `fons-fab/parser/sententia/initus.fab` (~line 44)

Same pattern

##### 8. Incipiet (async entry point)

**fons/ file**: `fons/parser/index.ts` (~line 3586)

**fons-fab/ file**: `fons-fab/parser/sententia/initus.fab` (~line 81)

Same pattern

### Phase 3: Grammar Documentation

**File**: `grammatica/regimen.md`

Update all EBNF rules and examples to show `reddit` option.

### Phase 4: Testing

#### Parser Tests

**File**: `proba/statements/elige-reddit.yaml`

```yaml
name: 'elige with reddit cases'
source: |
    functio getType(textus name) fit numerus {
        elige name {
            casu "a" reddit 1
            casu "b" reddit 2
            ceterum reddit 0
        }
    }
expected: |
    function getType(name: string): number {
        if (name === "a") {
            return 1;
        } else if (name === "b") {
            return 2;
        } else {
            return 0;
        }
    }
```

**File**: `proba/statements/si-reddit.yaml`

```yaml
name: 'si with reddit'
source: |
    functio classify(numerus x) fit textus {
        si x < 0 reddit "negative"
        si x == 0 reddit "zero"
        reddit "positive"
    }
expected: |
    function classify(x: number): string {
        if (x < 0) {
            return "negative";
        }
        if (x === 0) {
            return "zero";
        }
        return "positive";
    }
```

**File**: `proba/statements/reddit-error.yaml`

```yaml
name: 'reddit at top level should error'
source: |
    reddit 42
error: 'reddit can only appear after si/dum/ex/de/elige/discerne/incipit'
```

#### Working Examples

**File**: `exempla/statements/elige/with-reddit.fab`

```fab
# Elige with reddit syntax
#
# casu <value> reddit <expression>

functio getHttpStatus(numerus code) fit textus {
    elige code {
        casu 200 reddit "OK"
        casu 201 reddit "Created"
        casu 204 reddit "No Content"
        casu 400 reddit "Bad Request"
        casu 401 reddit "Unauthorized"
        casu 403 reddit "Forbidden"
        casu 404 reddit "Not Found"
        casu 500 reddit "Internal Server Error"
        casu 502 reddit "Bad Gateway"
        casu 503 reddit "Service Unavailable"
        ceterum reddit "Unknown Status"
    }
}

functio classify(numerus x) fit textus {
    si x < 0 reddit "negative"
    si x == 0 reddit "zero"
    reddit "positive"
}

incipit {
    scribe getHttpStatus(200)
    scribe getHttpStatus(404)
    scribe getHttpStatus(999)

    scribe classify(-5)
    scribe classify(0)
    scribe classify(10)
}
```

**File**: `exempla/statements/si/early-return.fab`

```fab
# Early returns with reddit

functio divide(numerus a, numerus b) fit numerus? {
    si b == 0 reddit nihil
    reddit a / b
}

functio findFirst(lista<numerus> items, numerus target) fit numerus? {
    ex items pro item {
        si item == target reddit item
    }
    reddit nihil
}

incipit {
    scribe divide(10, 2)   # 5
    scribe divide(10, 0)   # nihil

    fixum nums = [1, 2, 3, 4, 5]
    scribe findFirst(nums, 3)  # 3
    scribe findFirst(nums, 9)  # nihil
}
```

### Phase 5: Refactoring Existing Code

After implementation, refactor fons-fab/ to use `reddit`:

**Files to refactor** (~11 elige statements, ~200 individual cases):

1. `fons-fab/semantic/nucleus.fab` (line 60) - 6 cases
2. `fons-fab/semantic/expressia/primaria.fab` (line 59) - 6 cases
3. `fons-fab/parser/sententia/varia.fab` (line 69) - 4 cases
4. `fons-fab/codegen/ts/typus.fab` (line 16) - 12 cases
5. `fons-fab/parser/nucleus.fab` (line 387) - 18 cases
6. `fons-fab/parser/nucleus.fab` (line 417) - 3 cases
7. `fons-fab/lexor/errores.fab` (line 29) - 6 cases
8. `fons-fab/parser/errores.fab` (line 108) - 42 cases
9. `fons-fab/lexicon/verba.fab` (line 14) - 94 cases
10. `fons-fab/lexicon/verba.fab` (line 152) - 31 cases
11. `fons-fab/lexicon/verba.fab` (line 212) - 6 cases

**Pattern**: Replace `casu X { redde Y }` with `casu X reddit Y`

**Example transformation**:

```fab
# Before
elige nomen {
    casu "textus" { redde TEXTUS }
    casu "numerus" { redde NUMERUS }
    casu "fractus" { redde FRACTUS }
}

# After
elige nomen {
    casu "textus" reddit TEXTUS
    casu "numerus" reddit NUMERUS
    casu "fractus" reddit FRACTUS
}
```

## Summary of Changes

### fons/ (TypeScript Compiler)

| File                       | Change                                 | Lines         |
| -------------------------- | -------------------------------------- | ------------- |
| `fons/lexicon/keywords.ts` | Add `reddit` keyword                   | +1            |
| `fons/parser/index.ts`     | Add `reddit` support in si statement   | +8            |
| `fons/parser/index.ts`     | Add `reddit` support in dum statement  | +8            |
| `fons/parser/index.ts`     | Add `reddit` support in ex...pro       | +8            |
| `fons/parser/index.ts`     | Add `reddit` support in de...pro       | +8            |
| `fons/parser/index.ts`     | Add `reddit` support in elige cases    | +8            |
| `fons/parser/index.ts`     | Add `reddit` support in discerne cases | +8            |
| `fons/parser/index.ts`     | Add `reddit` support in incipit        | +8            |
| `fons/parser/index.ts`     | Add `reddit` support in incipiet       | +8            |
| `grammatica/regimen.md`    | Update EBNF grammar rules              | ~20           |
| **Total**                  |                                        | **~85 lines** |

### fons-fab/ (Bootstrap Compiler)

| File                                   | Change                           | Lines         |
| -------------------------------------- | -------------------------------- | ------------- |
| `fons-fab/lexicon/verba.fab`           | Add `reddit` keyword             | +1            |
| `fons-fab/parser/sententia/fluxus.fab` | Add `reddit` in elige cases      | +8            |
| `fons-fab/parser/sententia/initus.fab` | Add `reddit` in incipit/incipiet | +16           |
| `fons-fab/parser/sententia/*`          | Add `reddit` in si/dum/ex/de     | ~32           |
| **Total**                              |                                  | **~57 lines** |

### Test Files

| File                                       | Purpose                | Lines          |
| ------------------------------------------ | ---------------------- | -------------- |
| `proba/statements/elige-reddit.yaml`       | Test elige with reddit | ~20            |
| `proba/statements/si-reddit.yaml`          | Test si with reddit    | ~20            |
| `proba/statements/reddit-error.yaml`       | Test error handling    | ~10            |
| `exempla/statements/elige/with-reddit.fab` | Working elige example  | ~30            |
| `exempla/statements/si/early-return.fab`   | Working si example     | ~25            |
| **Total**                                  |                        | **~105 lines** |

### Refactoring Existing Code

| File                            | Cases to Convert  |
| ------------------------------- | ----------------- |
| `fons-fab/lexicon/verba.fab`    | 94 + 31 + 6 = 131 |
| `fons-fab/parser/errores.fab`   | 42                |
| `fons-fab/parser/nucleus.fab`   | 18 + 3 = 21       |
| `fons-fab/semantic/*`           | 6 + 6 = 12        |
| `fons-fab/codegen/ts/typus.fab` | 12                |
| `fons-fab/lexor/errores.fab`    | 6                 |
| **Total**                       | **~224 cases**    |

## Implementation Order

1. **Phase 1**: Add keyword to both compilers (2 files, trivial)
2. **Phase 2**: Add parser support in fons/ (1 file, 8 locations, ~65 lines)
3. **Phase 3**: Add parser support in fons-fab/ (4-5 files, ~57 lines)
4. **Phase 4**: Update grammar documentation (~20 lines)
5. **Phase 5**: Add tests and examples (~105 lines)
6. **Phase 6**: Refactor existing fons-fab/ code (~224 cases)

**Estimated effort**:

- Core implementation: 2-3 hours
- Testing: 1 hour
- Refactoring existing code: 1-2 hours
- **Total: 4-6 hours**

## Benefits

1. **Cleaner syntax**: `casu "x" reddit Y` vs `casu "x" ergo redde Y`
2. **LLM clarity**: Single keyword signals "line ends with return"
3. **Reduced tokens**: 6 chars vs 11 (45% shorter)
4. **Consistent**: Works everywhere `ergo` does
5. **Authentic Latin**: Real verb conjugation (3rd person present), not invented
6. **Better diffs**: Single-line changes instead of block changes
7. **Easier scanning**: All cases align vertically without brace clutter

## Alternatives Considered

### `ergoredde` (compound)

- **Pros**: Explicit combination of existing keywords
- **Cons**: Not authentic Latin, visually cluttered

### `igitur` (synonym for ergo)

- **Pros**: Classical synonym for ergo
- **Cons**: Still needs `redde`, doesn't solve verbosity

### `reddo` (1st person "I return")

- **Pros**: Single word, authentic
- **Cons**: Breaks pattern (fit/fiet are 3rd person, not 1st)

### `reddendum` (gerundive "to be returned")

- **Pros**: Matches figendum/variandum pattern
- **Cons**: Too long (9 chars), unclear benefit

**Decision**: `reddit` (3rd person) matches the `fit/fiet/fiunt/fient` pattern and is the shortest authentic option.

## Future Enhancements

None planned - this feature is complete as specified.

## Related Documents

- `grammatica/regimen.md` - Control flow grammar
- `consilia/intra-inter-refactoring.md` - Previous refactoring effort
- `consilia/bootstrap-ts.md` - Bootstrap compiler progress
