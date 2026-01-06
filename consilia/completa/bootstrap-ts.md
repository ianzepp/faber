---
status: completed
updated: 2026-01-06
note: Historical document describing bootstrap phases. Project completed and renamed to "Rivus"
see: fons/rivus/CHECKLIST.md for current implementation status
---

# Bootstrap: Self-Hosted Faber Compiler (TypeScript Target)

Rewrite the Faber compiler in Faber, targeting TypeScript/Bun.

## Status Summary (2026-01-06)

**COMPLETED.** The bootstrap compiler "Rivus" is now at `fons/rivus/` with 107 .fab source files (originally 51 in this document). TypeScript target fully implemented with 741 tests passing (100%). Output location: `opus/rivus/fons/ts/` (108 TypeScript files). Zig target partially implemented. CLI: `bun run build:rivus` to build, `bun run rivus compile <file.fab>` to use.

This document describes the historical development phases. Paths reference the original `fons-fab/` location which has been renamed to `fons/rivus/`.

## Current State (Historical - Phase 1)

| Module    | Location             |  Files |      Lines |
| --------- | -------------------- | -----: | ---------: |
| AST       | `fons-fab/ast/`      |      6 |      1,112 |
| Lexicon   | `fons-fab/lexicon/`  |      1 |        221 |
| Lexor     | `fons-fab/lexor/`    |      2 |        853 |
| Parser    | `fons-fab/parser/`   |     18 |      4,134 |
| Semantic  | `fons-fab/semantic/` |     17 |      2,844 |
| Codegen   | `fons-fab/codegen/`  |      6 |      1,859 |
| CLI       | `fons-fab/cli.fab`   |      1 |         67 |
| **Total** |                      | **51** | **11,090** |

### Bootstrap Progress

**✅ Phase 0 Complete - Syntax Modernization** (commits 5ea0b64, 3eb1309, 49fd9d0)

- ✅ Added keywords: `casu`, `ceterum`, `intra`, `inter`, `finge`
- ✅ Converted ~85 elige/discerne statements across 24 files
- ✅ All 51 fons-fab files compile successfully
- ✅ Verification: `bun run faber compile fons-fab/cli.fab` succeeds

**✅ Phase 1 Complete - TypeScript Compilation**

- ✅ TS codegen: lower `tabula[key]` to `Map.get/set` (b92ef64)
- ✅ Align fons-fab field names with AST, fix scope handling (ef1d259)
- ✅ All 51 files compile with zero TypeScript errors (20b19ed)

```bash
cd opus/bootstrap && npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler cli.ts
# Exit code: 0
```

**P2 - Comment Preservation** (deferred)

- ✅ AST has `notaePrae`/`notaePost` fields (fons-fab/ast/radix.fab)
- ✅ Comment formatting stubs added to fons-fab/codegen/typi.fab
- ✅ TsGenerator has notaePrae()/notaePost() helper methods
- ✅ genSententia() wraps output with comment helpers (currently return "")
- ⏳ Full implementation requires string methods (contains, split, trim) - deferred to post-bootstrap

### Out of Scope for Bootstrap

- **Stdlib registries** (lista/tabula/copia) - Not needed for TS target (native Array/Map/Set)
- **Morphological parsing** (lexicon declensions/conjugations) - Keyword lookup sufficient
- **Preamble templates** - Only needed for non-TS targets
- **Unit tests** - YAML integration tests provide coverage
- **Other codegen targets** (Zig, Rust, Fab) - Add after TS self-compiles

## Bootstrap Phases

### Phase 0: Syntax Modernization ✅ COMPLETE

**Goal**: Update fons-fab source to use modern syntax so fons/ compiler can parse it

**Tasks**:

1. ✅ Add `casu`, `ceterum`, `intra`, `inter` keywords to `fons-fab/lexicon/verba.fab`
2. ✅ Update `fons-fab/lexor/` to tokenize new keywords
3. ✅ Replace `si` → `casu` in all `elige`/`discerne` blocks (~85 statements)
4. ✅ Replace `secus` → `ceterum` in `elige` default cases
5. ✅ Verify: `bun run faber compile fons-fab/cli.fab` succeeds

**Results**: 24 files modified, ~537 lines changed (commits 5ea0b64, 3eb1309, 49fd9d0)

### Phase 1: TypeScript Compilation ✅ COMPLETE

**Goal**: All 51 files compile to TypeScript without type errors

**Tasks**:

1. ✅ Lower `tabula[key]` indexing to `Map.get/set` (b92ef64)
2. ✅ Fix import placement — imports hoisted to file level (ef1d259)
3. ✅ Align AST field names across fons-fab (ef1d259)
4. ✅ Add `@ publica` visibility to interface methods
5. ✅ Verify: `cd opus/bootstrap && npx tsc --noEmit cli.ts` passes

**Results**: 51 files, 0 TypeScript errors

### Phase 2: Self-Compilation (Current)

**Goal**: Bootstrap compiler can compile itself to TypeScript

**Tasks**:

1. ✅ Compile fons-fab with fons: all 51 files in opus/bootstrap/
2. ⏳ Run bootstrap compiler: `bun opus/bootstrap/cli.ts compile <test.fab>`
3. ⏳ Compare output with fons compiler (functional equivalence)
4. ⏳ Self-compile: bootstrap compiles fons-fab, output matches

### Phase 3: YAML Test Verification

**Goal**: Confirm feature coverage parity with fons compiler

**Tasks**:

1. Run proba/\*_/_.yaml tests through both compilers
2. Compare generated code (semantically equivalent, whitespace may differ)
3. Identify any missing features or edge cases
4. Document known differences

### Phase 4: Fab Codegen Target

**Goal**: True self-hosting (Faber → Faber)

**Tasks**:

1. Implement `fons-fab/codegen/fab/` target
2. Self-compile fons-fab with bootstrap compiler to Faber
3. Verify output is identical to fons-fab source (canonical formatting)
4. Switch to using fons-fab as primary compiler

### Phase 5: Additional Targets

**Goal**: Add Zig and Rust codegen for post-bootstrap goals

**Tasks**:

1. Implement `fons-fab/codegen/zig/` target + runtime
2. Implement `fons-fab/codegen/rs/` target
3. Run YAML tests to verify correctness
4. Remove fons/codegen/py/ and fons/codegen/cpp/ from original compiler

## Build Commands

```bash
# Current: Compile single file for testing (Phase 0)
bun run faber compile fons-fab/cli.fab -o /tmp/test.ts

# Phase 1: Compile all fons-fab files to opus/bootstrap/
for f in fons-fab/**/*.fab; do
  outfile="opus/bootstrap/${f#fons-fab/}"; outfile="${outfile%.fab}.ts"
  mkdir -p "$(dirname "$outfile")"
  bun run faber compile "$f" -o "$outfile"
done

# Phase 1: TypeScript type-check
cd opus/bootstrap && npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler cli.ts

# Phase 2: Run compiled compiler
bun opus/bootstrap/cli.ts compile input.fab -o output.ts

# Phase 2: Self-compile verification
bun opus/bootstrap/cli.ts compile fons-fab/cli.fab -o /tmp/cli2.ts
diff opus/bootstrap/cli.ts /tmp/cli2.ts  # Should be identical

# Phase 3: YAML test verification
bun run faber compile proba/fundamenta.yaml -o /tmp/old.ts
bun opus/bootstrap/cli.ts compile proba/fundamenta.yaml -o /tmp/new.ts
diff /tmp/old.ts /tmp/new.ts
```

## Key Patterns

### Resolvitor (Mutual Recursion)

The `pactum Resolvitor` interface breaks circular dependencies between expression and statement parsers:

```fab
pactum Resolvitor {
    functio parser() -> Parser
    functio expressia() -> Expressia
    functio sententia() -> Sententia
}

functio parseCondicio(Resolvitor r) -> Expressia {
    fixum p = r.parser()
    si p.congruetVerbum("sic") {
        fixum consequens = r.expressia()  # Cross-module call
    }
}
```

### Discretio Construction

Use `finge` to construct tagged union variants:

```fab
functio parseBinaria(Resolvitor r) -> Expressia {
    redde finge Binaria { locus: l, signum: s, sinister: a, dexter: b } qua Expressia
}
```

### Discerne Binding: `ut` vs `pro`

- `casu Variant ut v { v.field }` — binds whole variant, access fields via `v.`
- `casu Variant pro a, b { }` — extracts fields positionally (fragile, avoid)

### Scriptum for String Formatting

Avoid method call return type bugs by using `scriptum()`:

```fab
# Good
fixum line = scriptum("{}if ({}) {{", g.ind(), cond)

# Bad (method return type may be wrong)
fixum line = g.ind() + "if (" + cond + ") {"
```

Use `{{` and `}}` for literal braces.

## Latin Naming Reference

| English | Latin     | English    | Latin     |
| ------- | --------- | ---------- | --------- |
| parse   | resolvere | left       | sinister  |
| current | index     | right      | dexter    |
| tokens  | symbola   | operator   | signum    |
| peek    | specta    | body       | corpus    |
| advance | procede   | expression | expressia |
| match   | congruet  | statement  | sententia |
| expect  | expecta   | report     | renuncia  |

## Success Criteria (Phase 2 Complete)

1. ✅ fons-fab/ source uses modern syntax (casu/ceterum/intra/inter)
2. ✅ fons-fab/ compiles to valid TypeScript (type-checks pass)
3. ✅ Module system works (correct import statements generated)
4. ✅ Comments preserved in generated TypeScript
5. ✅ Bootstrap compiler can compile itself
6. ✅ Self-compilation produces functionally equivalent output
7. ✅ No runtime dependencies beyond Bun

## Success Criteria (Full Bootstrap - Phase 4 Complete)

8. ✅ Fab codegen target implemented
9. ✅ fons-fab self-compiles to Faber with canonical formatting
10. ✅ YAML test suite passes with bootstrap compiler
11. ✅ Bootstrap compiler becomes primary compiler

## Design Decisions

### Target Languages (Post-Bootstrap)

| Target | Keep | Rationale                                               |
| ------ | ---- | ------------------------------------------------------- |
| TS     | ✅   | Bootstrap, web, primary target                          |
| Zig    | ✅   | Native, systems, explicit memory                        |
| Rust   | ✅   | Native alternative, WASM, `de`/`in` aligns with borrows |
| Fab    | ✅   | Self-hosting, canonical formatting                      |
| Python | ❌   | Dynamic mismatch, heavy maintenance burden              |
| C++    | ❌   | No audience, no compelling differentiator               |

After bootstrap: remove `fons/codegen/py/` and `fons/codegen/cpp/`.
