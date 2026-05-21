# Testing

Faber testing has two parts:

- language syntax for defining suites and cases with `probandum` and `proba`
- the package command `faber test`, which compiles the generated Rust test crate and runs it through Cargo

## Language Syntax

### Cases

`proba` defines one test case.

```fab
proba "arithmetic passes" {
    adfirma 1 + 1 ≡ 2
}
```

### Suites

`probandum` groups cases and nested suites. Suite-level setup and teardown apply to contained cases.

```fab
probandum "math suite" {
    praepara omnia {
        fixum setup_value ← 1
    }

    proba "nested case" {
        adfirma setup_value ≡ 1
    }
}
```

Setup and teardown blocks use:

- `praepara` for before-each setup,
- `praeparabit` for async before-each setup,
- `postpara` for after-each teardown,
- `postparabit` for async after-each teardown,
- `omnia` when the setup or teardown applies to all cases in the suite.

## Test Modifiers

The current testing surface recognizes these `proba` modifiers:

| Verbum | Meaning | Current behavior |
| ------ | ------- | ---------------- |
| `omitte` | skip | Compiles to an ignored Rust test with a reason string |
| `futurum` | todo | Compiles to an ignored Rust test with a reason string |
| `solum` | focus | Selects the test in default `faber test` runs when any focused tests exist |
| `tag` | label | Used by `faber test --tag` |
| `requirit` | requires | Parsed and stored in metadata for later phases |
| `solum_in` | only in | Parsed and stored in metadata for later phases |

Other parsed modifiers such as `temporis`, `metior`, `repete`, and `fragilis` are also preserved in HIR metadata, but `faber test` does not enforce them yet.

## `faber test`

`faber test` is the package test entry point.

```bash
faber test [path]
faber test [path] [filter]
faber test [path] --name <name>
faber test [path] --suite <suite-path>
faber test [path] --tag <tag>
faber test [path] --ignored
faber test [path] --include-ignored
```

Behavior:

- Faber generates the Rust crate under `target/faber/`.
- Cargo test artifacts stay under the package `target/` directory, not `target/faber/target/`.
- All tests are still generated and compiled even when execution is narrowed.
- Selection changes execution by emitting Rust `#[ignore = "..."]` reasons for deselected tests.
- `--name` matches the original Faber test name, not the generated Rust function name.
- `--suite` matches the full Faber suite path joined with `/`.
- `--tag` matches a test's `tag` modifier.
- If any tests are marked `solum` and no explicit selector is given, default runs focus on the `solum` tests.
- Combined selectors use AND semantics.
- `--ignored` and `--include-ignored` map to the Rust harness and are mutually exclusive.
- Because selection is implemented with generated Rust ignore reasons, `--ignored` and `--include-ignored` also affect selection-ignored tests.

Cargo test still prints the generated Rust test names, so current output may show `proba_...` identifiers even though selection is expressed in Faber terms.

## Coverage

Faber line coverage is not implemented yet. Coverage reported against generated Rust under `target/faber/` is not the same as Faber source coverage; future coverage work needs Faber source mapping or Faber-native instrumentation.

## Fixtures

Repeatable smoke fixtures live under `examples/exempla/proba/packages/`:

- `passing`
- `failing`
- `ignored`
- `suite`
- `solum`
- `selectors`
- `selection-failure`

These fixtures are intentionally small and are used by the phase plan's command smokes.
