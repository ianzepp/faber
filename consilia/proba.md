---
status: implemented
targets: [ts]
updated: 2024-12
---

# Proba: Test Syntax for Faber

Test framework syntax for self-hosted compiler testing in rivus/.

## Implementation Status

| Feature              | Status   | Notes                         |
| -------------------- | -------- | ----------------------------- |
| `probandum` suites   | Done     | Nested suites supported       |
| `proba` test cases   | Done     | Basic test declarations       |
| `proba omitte`       | Done     | Skip with reason              |
| `proba futurum`      | Done     | Todo with reason              |
| `cura ante/post`     | Done     | Setup/teardown blocks         |
| `cura ... omnia`     | Done     | Before/after all              |
| `adfirma` assertions | Done     | Already existed               |
| Table-driven tests   | Phase 2  | `proba ex [...] pro { }`      |
| TypeScript codegen   | Done     | describe/test/beforeEach/etc. |
| Python codegen       | Not Done | —                             |
| Zig codegen          | Not Done | —                             |
| Rust codegen         | Not Done | —                             |
| C++ codegen          | Not Done | —                             |

## Keywords

| Construct   | Syntax                              | JS Equivalent             |
| ----------- | ----------------------------------- | ------------------------- |
| Suite       | `probandum "name" { }`              | `describe()`              |
| Test        | `proba "name" { }`                  | `test()`                  |
| Skip        | `proba omitte "reason" "name" { }`  | `test.skip()`             |
| Todo        | `proba futurum "reason" "name" { }` | `test.todo()`             |
| Before each | `cura ante { }`                     | `beforeEach()`            |
| Before all  | `cura ante omnia { }`               | `beforeAll()`             |
| After each  | `cura post { }`                     | `afterEach()`             |
| After all   | `cura post omnia { }`               | `afterAll()`              |
| Assertion   | `adfirma expr`                      | `expect(expr).toBe(true)` |

## Etymology

- **probandum** - gerundive of `probare` (to test/prove): "that which must be tested"
- **proba** - imperative of `probare`: "test!" / "prove!"
- **omitte** - imperative of `omittere`: "skip!" / "omit!"
- **futurum** - neuter noun: "the future" / "pending"
- **cura** - noun: "care, concern" — resource management keyword
- **ante** - preposition: "before"
- **post** - preposition: "after" (used as identifier after `cura`)
- **omnia** - neuter plural: "all things"

## Syntax

### Test Suites

```fab
probandum "Tokenizer" {
    proba "parses integers" { ... }
    proba "parses floats" { ... }
}
```

Suites are optional. Prefer flat `proba` with descriptive names over deep nesting.

### Assertions

Use `adfirma` with existing expression operators:

```fab
proba "equality" {
    adfirma result est "expected"
    adfirma result non est nihil
}

proba "unary checks" {
    adfirma value nonnulla          # not null
    adfirma count positivum         # > 0
    adfirma balance negativum       # < 0
    adfirma items nulla             # empty/null
}

proba "comparisons" {
    adfirma longitudo(arr) est 5
    adfirma age > 0
    adfirma age < 150
}
```

No special matcher API - assertions use the same operators as regular code.

### Failure Messages

`adfirma` auto-captures expression values on failure:

```fab
adfirma result est 42
# Failure: result (41) non est 42

adfirma result est 42, "parsed value"
# Failure: parsed value: 41 non est 42
```

Optional second argument provides a label. Both sides of comparisons are shown in output.

### Skip and Todo

```fab
proba omitte "blocked by issue #42" "test name" {
    # skipped - not executed
}

proba futurum "needs new parser feature" "test name" {
    # todo - marked pending
}
```

Modifier comes after `proba`, then reason string, then test name string.

### Setup and Teardown

```fab
probandum "Database" {
    cura ante omnia { db = connect() }   # once before all tests
    cura ante { db.reset() }             # before each test
    cura post { db.rollback() }          # after each test
    cura post omnia { db.close() }       # once after all tests

    proba "inserts" { ... }
    proba "updates" { ... }
}
```

Uses `cura` (care) keyword for resource management, consistent with the general
resource management design in `consilia/cura.md`. The `ante`/`post` timing
specifier follows, with optional `omnia` for "all" vs "each" semantics.

Design note: `post` is recognized as an identifier (not keyword) after `cura`
to avoid conflicts with method names like `post()` for HTTP operations.

### Async Tests

Tests are async by default at the wrapper level. Use normal async syntax inside:

```fab
proba "fetches data" {
    fixum result = cede fetch("/api")
    adfirma result.status est 200
}

proba "with destructuring" {
    ex fetch("/api") figendum { status, data }
    adfirma status est 200
    adfirma data nonnulla
}
```

No `futura proba` needed - `cede` and `figendum` just work.

## Grammar

```ebnf
probandumDecl := 'probandum' STRING '{' probandumBody '}'
probandumBody := (curaBlock | probandumDecl | probaStmt)*

probaStmt := 'proba' probaModifier? STRING blockStmt
probaModifier := 'omitte' STRING | 'futurum' STRING

curaBlock := 'cura' ('ante' | 'post') 'omnia'? blockStmt
```

## Code Generation

### TypeScript Output

```fab
probandum "Calculator" {
    cura ante omnia { db = connect() }
    cura ante { x = 0 }

    proba "adds numbers" {
        adfirma 1 + 1 est 2
    }

    proba omitte "broken" "needs fix" { }

    cura post { cleanup() }
    cura post omnia { db.close() }
}
```

Generates:

```typescript
describe('Calculator', () => {
    beforeAll(() => {
        db = connect();
    });
    beforeEach(() => {
        x = 0;
    });
    test('adds numbers', () => {
        if (!(1 + 1 === 2)) {
            throw new Error('Assertion failed: ((1 + 1) === 2)');
        }
    });
    test.skip('broken: needs fix', () => {});
    afterEach(() => {
        cleanup();
    });
    afterAll(() => {
        db.close();
    });
});
```

## Phase 2: Table-Driven Tests

Not yet implemented. Planned syntax:

```fab
proba "parse" ex [
    { ingressus: "42",  exitus: 42 },
    { ingressus: "-7",  exitus: -7 },
    { ingressus: "0",   exitus: 0 },
] pro { ingressus, exitus } {
    adfirma parse(ingressus) est exitus
}
```

`proba ... ex ... pro` iterates over test cases with destructuring. The test
runner expands this into N individual tests at compile time.

### Constraints

- Table expressions in `proba ex` must be compile-time literals
- Cannot use runtime variables for test case generation
- All unrolling happens during compilation, not execution

## Open Design Questions

### Stdlib vs Inline Harness

Two approaches for emitting test infrastructure:

**Inline**: Emit harness code directly into each test file.

- Pro: Zero dependencies, self-contained single file output
- Con: Duplicated boilerplate across test files

**Static stdlib**: Ship a `faber/proba` module per target.

- Pro: Clean test files, single source of truth, updatable without recompile
- Con: Additional distribution/installation step

**Recommendation**: Start with inline emission for simplicity. Extract to stdlib when harness needs grow beyond basic run/catch/report (e.g., timing, filtering, parallel execution, watch mode).

### Test Discovery

How does the test runner find tests?

- **Explicit**: Compiler emits a manifest or main() that lists all tests
- **Convention**: Test files named `*.proba.fab` or in `probae/` directory
- **Attribute**: Mark test files with a pragma or declaration

Current design assumes explicit - the compiler generates the runner as part of output.

### Async Test Timeout

Tests are async by default. Should there be a default timeout?

```fab
proba "slow operation" tempora 5000 {  # 5 second timeout?
    cede slowOperation()
}
```

Or leave timeout to the harness/runner level rather than per-test syntax?

### Failure Continuation

Current model: first `adfirma` failure throws, test stops.

Alternative: collect all failures, report at end. This would require a different codegen strategy. Probably not worth it for v1.
