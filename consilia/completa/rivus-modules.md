---
status: completed
updated: 2026-01-05
---

# Rivus Module Resolution

Rivus now supports local file import resolution for multi-file compilation. This document describes the implemented design based on faber's proven approach.

## Implementation Summary

All steps from the migration plan were completed:

| Step | Description | Status |
|------|-------------|--------|
| 0 | Plumb file path into CLI + semantic | ✓ |
| 1 | Add `ModulusExportum`, `ModulusResolutum` types | ✓ |
| 2 | Add file I/O extern declarations | ✓ |
| 3 | Implement `estLocaleImportum`, `resolveViaModuli` | ✓ |
| 4 | Implement `extraheExporta` | ✓ |
| 5 | Implement `resolveModulum` with cache/cycle | ✓ |
| 6 | Integrate `ImportaSententia` into `predeclare()` | ✓ |
| 7 | Fix parser to reject `ceteri` in imports | ✓ |
| 8 | Fix codegen to skip `norma` imports | ✓ |

**Key files created/modified:**
- `fons/rivus/semantic/modulus.fab` - new module resolution core
- `fons/rivus/semantic/systema.ts` - extern implementations for file I/O
- `fons/rivus/semantic/nucleus.fab` - added `viaIngressus`, cache fields to Analyzator
- `fons/rivus/semantic/index.fab` - integrated import predeclaration
- `fons/rivus/parser/sententia/declara.fab` - removed ceteri from imports
- `fons/rivus/codegen/ts/sententia/importa.fab` - skip norma imports
- `fons/rivus/cli.fab` - reads file path from stdin
- `scripta/rivus` - passes file path via stdin
- `scripta/build-rivus.ts` - injects extern implementations post-build

---

## Original Problem Statement

The exempla `statements/importa-local/main.fab` fails because rivus parses the `importa` statement but doesn't resolve symbols from the imported file:

```fab
# main.fab
ex "./utils" importa greet, ANSWER, Point

fixum textus message = greet("World")  # Error: undefined symbol 'greet'
```

The parser produces an `ImportaSententia` node, and the semantic layer has a stubbed handler:

```fab
# fons/rivus/semantic/sententia/index.fab:142-145
casu ImportaSententia {
    # Imports are processed in predeclaration phase
}
```

But no actual resolution occurs - the imported symbols are never added to scope.

## Current State Analysis

### What Rivus Already Has

1. **Two-phase semantic analysis** (`fons/rivus/semantic/index.fab`):
   - Phase 1: `predeclare()` - registers top-level names with placeholder types
   - Phase 2: `analyzeSententia()` - full body analysis

2. **Predeclaration infrastructure** handles: `FunctioDeclaratio`, `GenusDeclaratio`, `PactumDeclaratio`, `OrdoDeclaratio`, `DiscretioDeclaratio`, `TypusAliasDeclaratio`

3. **Import parsing** (`fons/rivus/parser/sententia/declara.fab:810-883`) - produces complete `ImportaSententia` nodes

4. **Symbol table** (`fons/rivus/semantic/scopus.fab`) with `Symbolum`, `SymbolumSpecies`, `definie()`, `quaere()`

### What's Missing

1. **File path context** - CLI reads from stdin, `analyze()` receives no path information
2. **Module resolution** - no code to load/parse/cache imported files
3. **Import symbol registration** - `predeclare()` has no `ImportaSententia` case

## Critical Blocker: Step 0

The current architecture cannot support relative imports:

```fab
# fons/rivus/cli.fab:21-54
incipiet {
    fixum source = lege  # stdin only - no file path
    # ...
    fixum semResult = analyze(parseResult.programma qua Programma)
    # No path context passed!
}
```

Compare to faber:

```typescript
// fons/faber/semantic/index.ts:193-198
export interface AnalyzeOptions {
    filePath?: string;
    moduleContext?: ModuleContext;
}

// fons/faber/cli.ts:226
const { errors: semanticErrors } = analyze(program, { filePath });
```

**Without a file path, `resolveViaModuli("./utils", ???)` cannot compute an absolute path.**

### Required Changes for Step 0

1. **Extend `Analyzator`** to store entry file path:

```fab
# fons/rivus/semantic/nucleus.fab
genus Analyzator {
    # ... existing fields ...
    textus? viaIngressus                    # entry file path (NEW)
    tabula<textus, ModulusResolutum> cache  # module cache (NEW)
    copia<textus> inProgressu               # cycle detection (NEW)
}
```

2. **Extend `analyze()` signature**:

```fab
# fons/rivus/semantic/index.fab
@ publica
functio analyze(Programma programma, textus? viaIngressus) -> SemanticResultatum {
    fixum a = novumAnalyzator()
    a.viaIngressus = viaIngressus
    # ...
}
```

3. **Update CLI** to pass file path (or add file-based entry point):

```fab
# Option A: Accept path as argument
fixum viaScripti = argumenta[0]
fixum source = legeScriptum(viaScripti)
fixum semResult = analyze(parseResult.programma qua Programma, viaScripti)

# Option B: Accept path via flag
# bun run rivus compile --file input.fab
```

## Implementation Plan

### File Structure

```
fons/rivus/semantic/
├── index.fab          # Entry point (modify)
├── nucleus.fab        # Analyzator state (modify)
├── modulus.fab        # Module resolution (NEW)
├── typi.fab           # Type system (existing)
└── scopus.fab         # Symbol table (existing)
```

### Types

```fab
# fons/rivus/semantic/modulus.fab

# Export information extracted from a module
@ publicum
genus ModulusExportum {
    textus nomen
    SymbolumSpecies species
    SemanticTypus typus
    Locus locus
}

# Resolved module with its exports
@ publicum
genus ModulusResolutum {
    tabula<textus, ModulusExportum> exporta
    textus via                              # absolute file path
    Programma? programma                    # parsed AST (for future multi-file codegen)
}

# Error result for module resolution
@ publicum
ordo ModulusError {
    NonInvenitur    # File not found
    ErrorLexoris    # Tokenization failed
    ErrorParsoris   # Parse failed
}
```

### Core Functions

```fab
# fons/rivus/semantic/modulus.fab

# Check if import source is a local file (not norma, not external package)
@ publica
functio estLocaleImportum(textus fons) -> bivalens {
    redde fons.initioCum("./") aut fons.initioCum("../")
}

# Check if import source is norma stdlib
@ publica
functio estNormaImportum(textus fons) -> bivalens {
    redde fons == "norma" aut fons.initioCum("norma/")
}

# Resolve import path to absolute filesystem path
# Returns nihil if file doesn't exist
@ publica
functio resolveViaModuli(textus fons, textus viaBasica) -> textus? {
    # 1. Get directory of importing file
    fixum directorium = viaParentis(viaBasica)

    # 2. Add .fab extension if missing
    varia via = fons
    si non via.finitCum(".fab") {
        via = via + ".fab"
    }

    # 3. Resolve to absolute path
    fixum viaAbsoluta = resolveViam(directorium, via)

    # 4. Check existence
    si non existitScriptum(viaAbsoluta) {
        redde nihil
    }

    redde viaAbsoluta
}

# Extract exports from a parsed program
# All top-level declarations are exported
@ publica
functio extraheExporta(Programma programma, textus via) -> ModulusResolutum {
    varia exporta = {} innatum tabula<textus, ModulusExportum>

    ex programma.corpus pro stmt {
        discerne stmt {
            casu FunctioDeclaratio ut f {
                exporta[f.nomen] = {
                    nomen: f.nomen,
                    species: SymbolumSpecies.Functio,
                    typus: functioTypus([], IGNOTUM, f.asynca, falsum),
                    locus: f.locus
                } qua ModulusExportum
            }
            casu GenusDeclaratio ut g {
                exporta[g.nomen] = {
                    nomen: g.nomen,
                    species: SymbolumSpecies.Genus,
                    typus: usitatumTypus(g.nomen, falsum),
                    locus: g.locus
                } qua ModulusExportum
            }
            casu PactumDeclaratio ut p {
                exporta[p.nomen] = {
                    nomen: p.nomen,
                    species: SymbolumSpecies.Pactum,
                    typus: usitatumTypus(p.nomen, falsum),
                    locus: p.locus
                } qua ModulusExportum
            }
            casu OrdoDeclaratio ut o {
                exporta[o.nomen] = {
                    nomen: o.nomen,
                    species: SymbolumSpecies.Ordo,
                    typus: usitatumTypus(o.nomen, falsum),
                    locus: o.locus
                } qua ModulusExportum
            }
            casu DiscretioDeclaratio ut d {
                exporta[d.nomen] = {
                    nomen: d.nomen,
                    species: SymbolumSpecies.TypusAlias,
                    typus: usitatumTypus(d.nomen, falsum),
                    locus: d.locus
                } qua ModulusExportum
            }
            casu TypusAliasDeclaratio ut t {
                exporta[t.nomen] = {
                    nomen: t.nomen,
                    species: SymbolumSpecies.TypusAlias,
                    typus: usitatumTypus(t.nomen, falsum),
                    locus: t.locus
                } qua ModulusExportum
            }
            casu VariaDeclaratio ut v {
                # Only simple identifiers, not destructuring patterns
                si v.nomen est Nomen {
                    fixum nomen = (v.nomen qua Nomen).nomen
                    exporta[nomen] = {
                        nomen: nomen,
                        species: SymbolumSpecies.Variabilis,
                        typus: IGNOTUM,
                        locus: v.locus
                    } qua ModulusExportum
                }
            }
        }
    }

    redde {
        exporta: exporta,
        via: via,
        programma: programma
    } qua ModulusResolutum
}

# Main entry point: resolve and load a local module
@ publica
functio resolveModulum(textus fons, Analyzator a) -> ModulusResolutum? {
    # 1. Must have entry path context
    si nihil a.viaIngressus {
        a.error("Cannot resolve local import without file context", { linea: 0, columna: 0, index: 0 } qua Locus)
        redde nihil
    }

    # 2. Resolve to absolute path
    fixum viaAbsoluta = resolveViaModuli(fons, a.viaIngressus qua textus)
    si nihil viaAbsoluta {
        redde nihil  # Caller reports ModuleNotFound
    }

    # 3. Check cache
    si nonnihil a.cache[viaAbsoluta] {
        redde a.cache[viaAbsoluta]
    }

    # 4. Check for cycles - return empty exports (JS/TS hoisting behavior)
    si a.inProgressu.habet(viaAbsoluta) {
        redde {
            exporta: {} innatum tabula<textus, ModulusExportum>,
            via: viaAbsoluta,
            programma: nihil
        } qua ModulusResolutum
    }

    # 5. Mark in progress
    a.inProgressu.adde(viaAbsoluta)

    # 6. Load and parse
    fixum sourceCode = legeScriptum(viaAbsoluta)
    fixum lexResult = lexare(sourceCode)
    si lexResult.errores.longitudo() > 0 {
        a.inProgressu.remove(viaAbsoluta)
        redde nihil  # Caller reports LexerError
    }

    fixum parseResult = resolvere(lexResult.symbola)
    si parseResult.errores.longitudo() > 0 aut nihil parseResult.programma {
        a.inProgressu.remove(viaAbsoluta)
        redde nihil  # Caller reports ParseError
    }

    # 7. Extract exports
    fixum modulus = extraheExporta(parseResult.programma qua Programma, viaAbsoluta)

    # 8. Cache before recursing (handles diamonds)
    a.cache[viaAbsoluta] = modulus

    # 9. Recursively resolve child imports (for cycle detection)
    ex (parseResult.programma qua Programma).corpus pro stmt {
        discerne stmt {
            casu ImportaSententia ut imp {
                si estLocaleImportum(imp.fons) {
                    # Temporarily update context for child resolution
                    fixum priorVia = a.viaIngressus
                    a.viaIngressus = viaAbsoluta
                    resolveModulum(imp.fons, a)
                    a.viaIngressus = priorVia
                }
            }
        }
    }

    # 10. Remove from in progress
    a.inProgressu.remove(viaAbsoluta)

    redde modulus
}
```

### Integration: Predeclaration

Extend `predeclare()` in `fons/rivus/semantic/index.fab`:

```fab
# Add case for ImportaSententia
casu ImportaSententia ut imp {
    # Skip norma imports (handled by intrinsics)
    si estNormaImportum(imp.fons) {
        # Norma symbols are defined via definieIntrinsica()
        # No additional work needed
    }
    # Handle local file imports
    sin estLocaleImportum(imp.fons) {
        fixum modulus = resolveModulum(imp.fons, a)

        si nihil modulus {
            a.error(scriptum("Module not found: §", imp.fons), imp.locus)
        } secus {
            # Wildcard import: ex "./x" importa * ut alias
            si imp.totum {
                si nonnihil imp.totumAlias {
                    # Define namespace alias
                    a.definie({
                        nomen: imp.totumAlias qua textus,
                        semanticTypus: IGNOTUM,
                        species: SymbolumSpecies.Variabilis,
                        mutabilis: falsum,
                        locus: imp.locus
                    } qua Symbolum)
                }
                # Also add all individual exports to scope
                ex modulus.exporta pro nomen, exportum {
                    a.definie({
                        nomen: exportum.nomen,
                        semanticTypus: exportum.typus,
                        species: exportum.species,
                        mutabilis: falsum,
                        locus: exportum.locus
                    } qua Symbolum)
                }
            } secus {
                # Named imports: ex "./x" importa foo, bar ut b
                ex imp.specificatores pro spec {
                    si spec.residuum {
                        # Rest in imports is a semantic error
                        a.error("'ceteri' is not valid in import specifiers", spec.locus)
                    } sin nonnihil modulus.exporta[spec.importatum] {
                        fixum exportum = modulus.exporta[spec.importatum] qua ModulusExportum
                        a.definie({
                            nomen: spec.locale,  # Use local alias if provided
                            semanticTypus: exportum.typus,
                            species: exportum.species,
                            mutabilis: falsum,
                            locus: spec.locus
                        } qua Symbolum)
                    } secus {
                        a.error(scriptum("'§' is not exported from '§'", spec.importatum, imp.fons), spec.locus)
                    }
                }
            }
        }
    }
    # External package imports pass through to codegen unchanged
}
```

### Parser Fix: Reject `ceteri` in Imports

The parser currently accepts `ceteri` (rest) in import specifiers, but GRAMMAR.md says "rest is only valid in destructuring contexts."

**Option A (Preferred):** Fix parser to reject `ceteri`:

```fab
# fons/rivus/parser/sententia/declara.fab:848-853
# REMOVE these lines:
varia residuum = falsum
si p.probaVerbum("ceteri") {
    residuum = verum
    p.procede()
}
```

**Option B (Fallback):** Keep parser, emit semantic error (shown in predeclare code above).

### Codegen Fix: Skip `norma` Imports

Rivus codegen currently emits all imports verbatim. Add norma check:

```fab
# fons/rivus/codegen/ts/sententia/index.fab
casu ImportaSententia ut imp {
    # Skip norma imports - these are compiler intrinsics
    si imp.fons == "norma" aut imp.fons.initioCum("norma/") {
        redde ""
    }

    # Emit other imports normally
    redde genImporta(imp.fons, imp.specificatores, imp.totum, imp.totumAlias, g)
}
```

### File I/O Dependencies

Module resolution requires file system access. Options:

**Option A (Recommended):** Add extern declarations for Bun/Node APIs:

```fab
# fons/rivus/semantic/systema.fab
externa functio legeScriptum(textus via) -> textus
externa functio existitScriptum(textus via) -> bivalens
externa functio viaParentis(textus via) -> textus
externa functio resolveViam(textus basis, textus relativum) -> textus
```

Codegen emits appropriate implementation:

```typescript
// TypeScript target
import { readFileSync, existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
const legeScriptum = (via: string) => readFileSync(via, 'utf-8');
const existitScriptum = existsSync;
const viaParentis = dirname;
const resolveViam = resolve;
```

**Option B:** Keep file I/O in faber (interim bootstrap). Rivus generates TypeScript that imports faber's `modules.ts`. Not truly self-hosted but unblocks progress.

## Design Decisions

### D1: Cycle Handling

**Decision:** Match faber's actual behavior - cycles return empty exports, no error.

Faber's `ModuleResult` type has a `'cycle'` error variant but the implementation returns `ok: true` with empty exports. This matches JS/TS import hoisting semantics.

```fab
si a.inProgressu.habet(viaAbsoluta) {
    # Cycle detected - return empty exports (not an error)
    redde {
        exporta: {} innatum tabula<textus, ModulusExportum>,
        via: viaAbsoluta,
        programma: nihil
    } qua ModulusResolutum
}
```

### D2: Type Information Depth

**Decision:** Names-only with placeholder types (`IGNOTUM`).

Full type resolution during module loading would require running semantic analysis on imported files. Instead, extract declaration names and assign placeholder types. The semantic analyzer will resolve actual types when imported symbols are used.

### D3: Export Visibility

**Decision:** All top-level declarations are exported (no `@publica` requirement for module exports).

This matches faber's behavior and TypeScript/ES6 semantics where named exports are explicit in the import statement.

## Migration Plan

| Step | Description | Dependency | Files |
|------|-------------|------------|-------|
| **0** | **Plumb file path into CLI + semantic** | None | `cli.fab`, `nucleus.fab`, `index.fab` |
| 1 | Add `ModulusExportum`, `ModulusResolutum` types | 0 | `modulus.fab` (new) |
| 2 | Add file I/O extern declarations | 0 | `systema.fab` (new) |
| 3 | Implement `estLocaleImportum`, `resolveViaModuli` | 2 | `modulus.fab` |
| 4 | Implement `extraheExporta` | 1 | `modulus.fab` |
| 5 | Implement `resolveModulum` with cache/cycle | 3, 4 | `modulus.fab` |
| 6 | Integrate `ImportaSententia` into `predeclare()` | 5 | `index.fab` |
| 7 | Fix parser to reject `ceteri` in imports | None | `declara.fab` |
| 8 | Fix codegen to skip `norma` imports | None | `sententia/index.fab` |
| 9 | Add test cases (cycles, diamonds, aliases) | 6 | `exempla/` |
| 10 | Verify self-compilation test | All | - |
| 11 | Verify `statements/importa-local` passes | 10 | - |

**Step 0 is a blocker** - no other work can proceed without file path context.

Steps 7 and 8 can be done in parallel with steps 1-6.

## Test Strategy

### Minimal Test Case

```
fons/exempla/statements/importa-local/
├── main.fab    # ex "./utils" importa greet, ANSWER, Point
└── utils.fab   # exports greet(), ANSWER, Point
```

### Additional Test Cases

| Test | Description |
|------|-------------|
| `importa-local/cycle-a.fab` | A imports B, B imports A (should work, empty exports on cycle) |
| `importa-local/diamond.fab` | A imports B+C, both import D (tests caching) |
| `importa-local/nested.fab` | A imports B, B imports C (tests recursion) |
| `importa-local/alias.fab` | `importa foo ut f` (tests alias handling) |
| `importa-local/wildcard.fab` | `importa * ut utils` (tests namespace import) |
| `importa-local/not-found.fab` | Import nonexistent file (tests error) |
| `importa-local/not-exported.fab` | Import symbol not in module (tests error) |

### Ultimate Test: Self-Compilation

The most valuable test is: **can rivus semantically analyze its own source tree?**

Rivus is a multi-file compiler:

```
fons/rivus/
├── cli.fab                          # imports semantic/index
├── semantic/
│   ├── index.fab                    # imports nucleus, modulus, typi, scopus
│   ├── modulus.fab (NEW)            # imports lexor, parser
│   └── ...
├── parser/
│   └── sententia/
│       └── declara.fab              # imports ../resolvitor, ../../ast/*
└── ...
```

If module resolution works correctly:
- Relative paths resolve (`../ast/...`, `./typi`)
- Diamond dependencies work (many files import `typi.fab`)
- Caching prevents re-parsing the same file
- No spurious `UndefinedVariable` errors

**If rivus can compile itself, the exempla cases are trivial.**

## Key Files

| File | Purpose |
|------|---------|
| `fons/faber/semantic/modules.ts` | Reference implementation (396 lines) |
| `fons/rivus/semantic/index.fab` | Entry point, `analyze()`, `predeclare()` |
| `fons/rivus/semantic/nucleus.fab` | `Analyzator` state |
| `fons/rivus/semantic/modulus.fab` | Module resolution (NEW) |
| `fons/rivus/parser/sententia/declara.fab:810-883` | `parseImportaSententia` |
| `fons/rivus/codegen/ts/sententia/importa.fab` | TS import codegen |
| `fons/exempla/statements/importa-local/` | Test case |

---

Opus perfectum est.

---

## Appendix: GPT-5.2 Peer Review Notes

<details>
<summary>First Review</summary>

### Reality check: Rivus already has the right "phase hook"

This proposal is directionally correct, but a few parts are stale relative to today's Rivus tree:

- Rivus _does_ have a semantic layer and a predeclaration pass already: `fons/rivus/semantic/index.fab` predeclares names (Phase 1) then analyzes bodies (Phase 2). Imports are explicitly treated as "handled in predeclaration" but are not yet implemented: `fons/rivus/semantic/sententia/index.fab`.
- Therefore, the cleanest integration point is **to extend `predeclare()` to process `ImportaSententia`** (and potentially also reject non-top-level imports if you want TS-parity).

Concretely: today, any identifier only introduced via `ex "./x" importa foo` will still trip `UndefinedVariable` during expression analysis (e.g. `fons/rivus/semantic/expressia/primaria.fab`). The symptom described in the doc matches this.

### A bigger blocker than the algorithm: where does `viaBasica` come from?

Your plan assumes the semantic layer knows "the importing file's path". But the current bootstrap CLI reads source from stdin and calls `analyze(programma)` with no path context: `fons/rivus/cli.fab`.

That means even a perfect `resolveViaModuli()` cannot be correct unless:

- The compiler entry point is file-based (the `bun run rivus compile <file.fab>` path), or
- You add an optional `viaIngressus`/`basePath` parameter to semantic analysis (or stash it in `Analyzator`).

So: **treat "plumb entry file path into semantic" as Step 0** for local imports. Otherwise module resolution only works in environments where the "current directory" coincidentally matches the importer's directory.

### Where module resolution should live (my vote)

I agree with Option B (semantic layer), but with a stronger rationale:

- Module resolution is needed for **semantic symbol availability** (prevent undefined symbol errors) and for **type availability** (eventually), so it belongs in semantic.
- Rivus already has the correct two-pass structure; modules fit naturally as a Phase 1 concern.

Recommended shape:

- Add `fons/rivus/semantic/modulus.fab` implementing a near-port of `fons/faber/semantic/modules.ts`.
- Extend `fons/rivus/semantic/index.fab:predeclare()` with an `ImportaSententia` branch that:
    - Detects norma vs local vs external
    - Loads local exports (or at minimum, defines imported names as `IGNOTUM`)
    - Emits `NotExportedFromModule` / `ModuleNotFound` errors as appropriate

### Cycles: decide whether you actually want an error

This doc says "match faber behavior" by returning empty exports on cycles. That's roughly what Faber's resolver does today: `fons/faber/semantic/modules.ts` returns `ok: true` with an empty export map when `inProgress` hits (it does _not_ surface a cycle error).

Two implications:

- If you truly want parity, **S013 `CircularImport` should probably not fire for module cycles** (or only for "hard" cycles you choose to forbid).
- If you _do_ want cycles to be an error in Rivus (stricter than Faber), that's defensible—but then the document shouldn't present "match Faber" as the goal.

Caveat lector: Faber's types mention `'cycle'` in `ModuleResult`, but the implementation currently treats cycles as "ok" rather than "error". Aligning expectations here will save you debugging time.

### Export extraction: names-only is fine, but watch namespace imports

I agree with your Recommendation Q3 (names-only / placeholder types) as the bootstrap-friendly move.

However, there are two practical wrinkles:

1. **Wildcard namespace import** (`ex "./utils" importa * ut utils`)

- The doc lists this as a test case.
- In Faber, codegen supports this, but semantic import binding for the alias is easy to miss. In your Rivus plan, make sure you define the alias symbol (probably as `IGNOTUM` or `objectum`) in addition to (or instead of) importing all members.

2. **Kind mapping**

Your sketch stores `genus` as strings like "functio" | "variabilis" etc. Rivus's symbol table wants `SymbolumSpecies` plus a `SemanticTypus`. You can still keep a lightweight export record, but you'll eventually want a mapping:

- `functio` → `SymbolumSpecies.Functio` + `functioTypus([...IGNOTUM], IGNOTUM/VACUUM, async, falsum)`
- `fixum/varia` → `SymbolumSpecies.Variabilis` + `IGNOTUM`
- `genus/pactum/ordo/typus/discretio` → a `Usitatum`/`Ordo`/`Genus` placeholder as needed

This is essentially what Faber's `extractExports()` does, just in Rivus's type system.

### Specifier grammar mismatch to flag now

Your document implicitly assumes import specifiers are `name` or `name ut alias` or `*`. That matches `GRAMMAR.md`.

But Rivus's parser currently allows `ceteri` in `parseImportaSententia()` (it shares the destructuring specifier shape): `fons/rivus/parser/sententia/declara.fab`.

From `GRAMMAR.md`, `ceteri` is intended to be destructuring-only ("rest is only valid in destructuring contexts"). If you don't fix the parser immediately, module resolution should at least treat `residuum` on imports as a semantic error (otherwise you'll generate nonsense TS).

### Don't forget `norma` parity (even if this doc is "local imports")

The baseline behavior in README/GRAMMAR is that `norma` imports are compiler intrinsics, not runtime imports.

- Faber TS codegen explicitly skips emitting `import ... from "norma"` and `"norma/*"`: `fons/faber/codegen/ts/statements/importa.ts`.
- Rivus TS codegen currently emits imports verbatim for all sources: `fons/rivus/codegen/ts/sententia/importa.fab`.

Even if you stage the work, it's worth calling out here because once Rivus can resolve local imports semantically, users will immediately start writing `ex norma importa ...` and expect it to behave like Faber.

### Test strategy: start with the bootstrap's own source tree

Besides `fons/exempla/statements/importa-local/`, the most valuable "real" test is: **can Rivus semantically analyze its own multi-file compiler source** without spurious `UndefinedVariable` errors?

That forces:

- Relative path resolution (`../ast/...`, `./typi`, etc.)
- Diamond-shaped dependency graphs (common in analyzer modules)
- Caching correctness (performance + avoiding repeated parse)

Once that works, the exempla cases become confidence checks rather than the only proof.

</details>

<details>
<summary>Second Review (Additional)</summary>

### Executive Summary

The proposal is well-researched and directionally correct. The algorithm (path resolution → file loading → export extraction → caching with cycle detection) is a solid match for faber's proven implementation. However, the implementation plan needs three critical corrections before beginning work:

1. **Predeclaration integration is already designed**: Rivus's `predeclare()` hook is correct—no separate phase needed
2. **Missing context plumbing**: The CLI passes no file path to `analyze()`, making relative imports impossible without changes
3. **Parser allows `ceteri` in imports**: Grammar says "rest is only valid in destructuring contexts" but parser accepts it

The document's Q3 (names-only exports) and Q1 (semantic layer placement) are both sound recommendations.

### Critical Blocker: Entry Point Doesn't Pass File Path

The plan's `resolveViaModuli()` function assumes `viaBasica` (base path) is available, but the current CLI architecture doesn't provide it.

**Required fix**: Extend `fons/rivus/semantic/index.fab:analyze()` to accept optional `viaIngressus` parameter, store it in `Analyzator`, and pass through to module resolution. CLI needs file-based entry point (`bun run rivus compile <file.fab>`) or to accept path parameter.

Without this, `resolveViaModuli()` can only work if:

- Current working directory happens to match importing file's directory, or
- You hardcode absolute paths (unacceptable for general use)

Treat "plumb entry file path into semantic" as Step 0—otherwise module resolution is fundamentally broken.

### Type Mapping Detail

The proposal's `ModulusExportum.genus` field stores strings ("functio", "variabilis", etc.). This works for lightweight tracking, but Rivus's symbol table expects `SymbolumSpecies` enum and `SemanticTypus`. You'll need a mapping function during export registration.

### Wildcard Imports Need Special Handling

The document mentions wildcard imports (`ex "./utils" importa * ut utils`) as a test case, but the algorithm needs to define two symbols:

1. **Individual exports**: Register each function/variable/type separately
2. **Namespace alias**: Register `utils` as `IGNOTUM` or `objectum` type

### Parser Grammar Mismatch

The document correctly notes that `ceteri` (rest) should be destructuring-only per GRAMMAR.md. However, Rivus's parser reuses the destructuring specifier shape for imports. This means `ex "./utils" importa ceteri rest` parses successfully but should be an error.

### Norma Import Parity is a prerequisite

The document focuses on local imports, but `norma` imports will be the next gap users hit. Current behavior mismatch:

- **Faber**: Skips `norma` imports entirely (codegen emits nothing)
- **Rivus**: Emits imports verbatim

### Test Strategy: Self-Compilation First

The most valuable test is: **Can Rivus semantically analyze its own source tree?**

If Rivus can compile itself without `UndefinedVariable` errors, the exempla cases are trivial.

### File I/O Strategy

Recommended approach: Option B with target-specific adapters (extern declarations for `readFileSync`, `existsSync`, etc.). Codegen emits appropriate implementation for each target.

### Conclusion

The proposal is 90% complete. The algorithm is sound, the data structures are appropriate, and the integration point is correct. The three gaps (CLI path plumbing, parser `ceteri` acceptance, codegen `norma` handling) are preconditions—not design flaws.

Fix Step 0 (CLI path) first. Without it, module resolution is impossible to test correctly.

</details>
