# Proba - Test Framework

YAML-driven cross-target codegen tests for the Faber compiler.

## Running Tests

```bash
bun test                          # Run all tests
bun test -t "binary"              # Run tests matching pattern
bun test -t "@ts"                 # Run only TypeScript target tests
bun test -t "@py"                 # Run only Python target tests
bun test -t "@zig"                # Run only Zig target tests
bun test -t "@rs"                 # Run only Rust target tests
bun test -t "@cpp"                # Run only C++ target tests
COVERAGE_DETAILS=1 bun test       # Show per-suite coverage breakdown
STRICT_COVERAGE=1 bun test        # Fail on missing target expectations
STRICT_COVERAGE="binary" bun test # Strict mode for matching tests only
```

## Directory Structure

```
proba/
├── runner.test.ts             # Test harness (loads YAML, runs against targets)
├── codegen/
│   ├── expressions/           # Expression codegen tests (call.yaml, binary.yaml, ...)
│   └── statements/            # Statement codegen tests (si.yaml, functio.yaml, ...)
├── norma/                     # Standard library tests
│   ├── aleator.yaml           # Random number generation
│   ├── copia.yaml             # Set methods
│   ├── lista.yaml             # List/array methods
│   ├── mathesis.yaml          # Math functions
│   └── tabula.yaml            # Map/dict methods
├── casus.yaml                 # Error handling (tempta/cape/iace)
├── curator.yaml               # Resource management (cura...fit)
├── fundamenta.yaml            # Primitives and literals
├── typi.yaml                  # Type system
└── ...
```

Suite names derive from file paths: `codegen/statements/si.yaml` becomes suite `codegen/statements/si`.

## Test File Format

Each YAML file contains an array of test cases:

```yaml
# Modern format (preferred)
- name: descriptive test name
  source: |
      fixum x = 1 + 2
  expect:
      ts: 'const x = (1 + 2);'
      py: 'x = (1 + 2)'
      rs: ['let x', '(1 + 2)']
      cpp:
          contains: ['const auto x']
          not_contains: ['var']
      zig:
          exact: 'const x = (1 + 2);'
  skip: [cpp] # optional: skip specific targets

# Legacy format (top-level expectations)
- name: test name
  source: 'fixum x = 1'
  ts: 'const x = 1;'
  py: 'x = 1'
```

### Input Fields

| Field    | Description                     |
| -------- | ------------------------------- |
| `source` | Faber source code to compile    |

### Compiler Skip Flags

| Field   | Description                             |
| ------- | --------------------------------------- |
| `faber` | Set to `false` to skip for faber tests  |
| `rivus` | Set to `false` to skip for rivus tests  |

Use these when a feature exists in one compiler but not the other.

### Expectation Formats

| Format                                    | Behavior                      |
| ----------------------------------------- | ----------------------------- |
| `string`                                  | Exact match (after trimming)  |
| `string[]`                                | All fragments must be present |
| `{ exact: string }`                       | Exact match                   |
| `{ contains: string[] }`                  | All fragments must be present |
| `{ not_contains: string[] }`              | Fragments must NOT be present |
| `{ contains: [...], not_contains: [...]}` | Combined inclusion/exclusion  |

### Targets

Tests can specify expectations for any combination of:

- `ts` - TypeScript
- `py` - Python
- `rs` - Rust
- `cpp` - C++
- `zig` - Zig
- `fab` - Canonical Faber (re-emit)

Missing targets are tracked in coverage reports. Use `skip: [target]` to explicitly exclude a target (won't count as missing).

## Error Tests (errata)

Test that invalid code produces expected errors:

```yaml
- name: undefined variable
  source: 'fixum x = unknownVar'
  errata:
      - 'Semantic errors'
      - 'Undefined variable'

- name: any tokenizer error
  source: 'fixum x = "unterminated'
  errata: true # any error is acceptable

- name: exact error message
  source: 'bad code'
  errata: "Parse errors: P001: Expected ')'"
```

### Errata Formats

| Format     | Behavior                             |
| ---------- | ------------------------------------ |
| `true`     | Any error is acceptable              |
| `string`   | Exact match on error message         |
| `string[]` | All fragments must appear in message |

Errata tests use strict compilation (tokenizer + parser + semantic errors all cause failure).

## How the Runner Works

1. **Discovery**: Recursively finds all `.yaml` files in `proba/`
2. **Parsing**: Loads each file as an array of test cases
3. **Grouping**: Creates a describe block per file, nested by target
4. **Execution**: For each test case:
    - Errata tests: Compile strictly, expect failure, validate error message
    - Normal tests: Compile leniently (ignores undefined vars in snippets), check output
5. **Coverage**: After all tests, prints missing expectations by target

### Test Execution Flow

```
tokenize(source) -> parse(tokens) -> analyze(program) -> generate(program, target)
                                                               |
                                               compare output to expectation
```

Lenient mode (normal tests) ignores semantic errors to allow snippet testing with undefined variables. Strict mode (errata tests) fails on any tokenizer, parse, or semantic error.

## Coverage Reports

After test runs, missing expectations are summarized:

```
COVERAGE REPORT: Tests missing target expectations
======================================================================

Missing by target:
  rs: 45 tests
  zig: 23 tests
  cpp: 12 tests

======================================================================
Total: 80 tests with incomplete coverage
======================================================================
```

Use `COVERAGE_DETAILS=1` for per-suite breakdown with specific test names.

## Adding New Tests

1. Find or create the appropriate YAML file based on what you're testing
2. Add a test case with `source` and `expect` block
3. Include expectations for all targets you want to verify
4. Use `skip` for targets that genuinely can't support the feature
5. Use `faber: false` or `rivus: false` for compiler-specific features
6. Run `bun test -t "your test name"` to verify

Example workflow:

```bash
# Add test to proba/codegen/expressions/call.yaml
# Run just that test
bun test -t "my new call test"

# Verify all call tests pass
bun test -t "call"

# Run full suite
bun test
```
