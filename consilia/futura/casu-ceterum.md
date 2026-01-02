# Switch Case Keywords: `casu`/`ceterum`

## Summary

Replace `si` with `casu` inside `elige` and `discerne` blocks, and replace `secus` with `ceterum` only inside `elige`, to eliminate keyword overloading.

## Motivation

Currently `si` is overloaded across multiple contexts:

| Context        | Usage                                | Meaning              |
| -------------- | ------------------------------------ | -------------------- |
| Conditional    | `si condition { }`                   | "if condition"       |
| Switch case    | `elige x { si "value" { } }`         | "if [matches] value" |
| Optional param | `si numerus depth`                   | "if [provided]"      |
| Pattern match  | `discerne x { si Variant ut v { } }` | "if [is] Variant"    |

This creates ambiguity for LLMs and humans parsing the syntax. The structural context differs even though all usages are semantically conditional.

## Proposed Syntax

### Before

```fab
elige status {
    si "pending" { ... }
    si "active" { ... }
    secus { ... }
}

discerne node {
    si BinaryExpr ut e { ... }
    si Literal ut l { ... }
}
```

### After

```fab
elige status {
    casu "pending" { ... }
    casu "active" { ... }
    ceterum { ... }
}

discerne node {
    casu BinaryExpr ut e { ... }
    casu Literal ut l { ... }
}
```

## Latin Grammar

**`casu`** — ablative singular of `casus` (4th declension noun meaning "fall/case/event")

- Ablative of circumstance: "in the case [of]"
- Usage: `casu "pending"` = "in the case of 'pending'"

**`ceterum`** — adverb meaning "for the rest, besides, otherwise"

- Derived from `ceterus` ("the rest/remaining")
- Distinct from `ceteri` (masculine plural, used for rest/spread patterns)

The pairing is grammatically sound: `casu` introduces specific matches (prepositional-style), `ceterum` handles "everything else" (adverb). This mirrors how `si`/`secus` work for boolean conditionals — different parts of speech serving different syntactic roles.

## Semantic Separation

After this change:

| Construct                  | Keywords          | Purpose        |
| -------------------------- | ----------------- | -------------- |
| `si`/`sin`/`secus`         | boolean condition | "if X is true" |
| `elige` + `casu`/`ceterum` | value match       | "in case of X" |
| `discerne` + `casu`        | variant match     | "in case of X" |

The optional parameter `si` remains unchanged — it appears in function signatures, not control flow blocks, so context is unambiguous.

## Scope of Changes

### TypeScript Compiler (`fons/`)

| File                               | Changes                                                          |
| ---------------------------------- | ---------------------------------------------------------------- |
| `lexicon/keywords.ts`              | Add `casu`, `ceterum` as keywords                                |
| `parser/ast.ts`                    | Rename `EligeSiCase` types if needed                             |
| `parser/index.ts`                  | Parse `casu`/`ceterum` instead of `si`/`secus` in elige/discerne |
| `parser/errors.ts`                 | Update error messages and hints                                  |
| `codegen/*/statements/elige.ts`    | Update case handling (6 targets)                                 |
| `codegen/*/statements/discerne.ts` | Update case handling (6 targets)                                 |
| `codegen/fab/generator.ts`         | Emit `casu` (and `ceterum` for elige only)                       |

### Tests (`proba/`)

| File                               | Changes        |
| ---------------------------------- | -------------- |
| `codegen/statements/elige.yaml`    | ~50 test cases |
| `codegen/statements/discerne.yaml` | ~22 test cases |

### Examples (`exempla/`)

| File                            | Changes |
| ------------------------------- | ------- |
| `statements/elige/*.fab`        | 3 files |
| `statements/discerne/basic.fab` | 1 file  |

### Self-Hosted Compiler (`fons-fab/`)

~28 files contain `secus`, but only those inside `elige` blocks change. Boolean `si`/`secus` pairs remain unchanged. Key files:

- `lexicon/verba.fab` — add keyword recognition
- `parser/sententia/fluxus.fab` — parse new keywords
- `parser/errores.fab` — update error hints
- `codegen/ts/sententia/index.fab` — update emission

### Documentation

- `GRAMMAR.md` — regenerate from parser
- `AGENTS.md` — update syntax examples
- `grammatica/*.md` — update relevant sections

## Estimated Effort

| Task                | Estimate     |
| ------------------- | ------------ |
| TypeScript compiler | ~1 hour      |
| Test YAML updates   | ~30 min      |
| Exempla updates     | ~15 min      |
| fons-fab updates    | ~1-2 hours   |
| Documentation       | ~30 min      |
| **Total**           | **~4 hours** |

## Risk

The main complexity is in `fons-fab/` — `secus` appears ~130 times across 28 files, but most are boolean `si`/`secus` pairs that should NOT change. Only those inside `elige { }` blocks become `ceterum`.

## Not Changing

- `si`/`sin`/`secus` in boolean conditionals — unchanged
- `si` as optional parameter modifier — unchanged
- `ceteri` as rest/spread keyword — unchanged (distinct from `ceterum`)
