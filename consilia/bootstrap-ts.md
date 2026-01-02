# Bootstrap: Self-Hosted Faber Compiler (TypeScript Target)

Rewrite the Faber compiler in Faber, targeting TypeScript/Bun.

## Rationale

1. **Path of least resistance** — TS codegen is the most mature (632 test expectations)
2. **Same runtime** — Bun runs both current compiler and output
3. **No memory management** — GC handles allocation; no `cura`/`curata` complexity
4. **Faster iteration** — Fix issues in Faber, recompile, test immediately

## Current State

**Phase 4 (Integration) in progress.** All 51 files compile to TypeScript (~11,090 lines total). TypeScript type-checking has 299 errors remaining.

### Solved Blockers

- ✅ Mutual recursion → solved with `pactum Resolvitor`
- ✅ Discretio instantiation → solved with `finge` keyword
- ✅ Function hoisting → solved with two-pass semantic analysis
- ✅ Do-while loops → implemented as `fac { } dum condition`
- ✅ AST as discretio → unified `Expressia` (24 variants) and `Sententia` (31 variants)
- ✅ Discretio return type mismatch → fixed in semantic analyzer
- ✅ Discerne binding types → fixed with `si Variant ut x` syntax (binds whole variant); 652 occurrences migrated from `pro` to `ut`
- ✅ Missing exports → added `@ publicum` to `discretio` and `ordo` declarations
- ✅ Incorrect enum names → fixed `SymbolumGenus` members (e.g., `Semicolon` → `PunctumColon`)
- ✅ Stdlib method names → fixed to match registry (e.g., `.iunge()` → `.coniunge()`, `.numerus()` → `.longitudo()`)

### Active Workarounds

- ⚠️ Method call return types → workaround with `scriptum()` formatting
- ⚠️ Nullable parameters → use `ignotum` type instead of `Type?`

### Module Status

| Module    | Location             |  Files |      Lines | Status                        |
| --------- | -------------------- | -----: | ---------: | ----------------------------- |
| AST       | `fons-fab/ast/`      |      6 |      1,112 | ✅ Complete (discretio-based) |
| Lexicon   | `fons-fab/lexicon/`  |      1 |        221 | ✅ Complete                   |
| Lexor     | `fons-fab/lexor/`    |      2 |        853 | ✅ Complete                   |
| Parser    | `fons-fab/parser/`   |     18 |      4,134 | ✅ Complete                   |
| Semantic  | `fons-fab/semantic/` |     17 |      2,844 | ✅ Complete                   |
| Codegen   | `fons-fab/codegen/`  |      6 |      1,859 | ✅ Complete                   |
| CLI       | `fons-fab/cli.fab`   |      1 |         67 | ✅ Complete                   |
| **Total** |                      | **51** | **11,090** | **All files compile**         |

### TypeScript Type-Check Status

**299 errors remaining** (down from 358). Compile to `opus/bootstrap/` and check with:

```bash
cd opus/bootstrap && npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler cli.ts
```

#### Error Categories

1. **Missing exports (~24 errors):** Some `genus` and `ordo` declarations need `@ publicum`
    - `genus Resolvitor` in `parser/resolvitor.fab`
    - `ordo ParserErrorCodice` in `parser/errores.fab`
    - `ordo LexorErrorCodice` in `lexor/errores.fab`

2. **Private method accessibility (~22 errors):** Methods on `Analyzator` class are private but called from other files
    - `intraScopum`, `exiScopum`, `error` in `semantic/nucleus.fab`

3. **Discriminated union narrowing (~70+ errors):** Pattern matching with `discerne`/`si...ut` doesn't narrow TypeScript types correctly
    - Properties like `signum`, `locus`, `corpus` show as "does not exist on type 'never'"

4. **Missing tabula method (~4 errors):** `habet` method not in `fons/codegen/tabula.ts` registry
    - Should map to `.has()`

5. **Import placement (~5 errors):** Imports generated inside functions instead of at module top

### Resolvitor Pattern

The mutual recursion problem between expression and statement parsers was solved using a `pactum Resolvitor` interface:

```fab
pactum Resolvitor {
    functio parser() -> Parser
    functio expressia() -> Expressia
    functio sententia() -> Sententia
    functio massa() -> MassaSententia
    functio adnotatio() -> TypusAnnotatio
}
```

Parsing functions receive `Resolvitor r` and call `r.expressia()`, `r.sententia()` etc. for cross-module parsing. The concrete `Parsator` genus in `parser/index.fab` implements this interface and wires up all parsing functions.

### Discretio Variant Construction

Use `finge` to construct discretio variants:

```fab
discretio Expressia {
    Binaria { Locus locus, textus signum, Expressia sinister, Expressia dexter }
    Littera { Locus locus, LitteraGenus species, textus crudus }
    # ...
}

functio parseBinaria(Resolvitor r) -> Expressia {
    # ...
    redde finge Binaria { locus: l, signum: s, sinister: a, dexter: b } qua Expressia
}
```

### Parser (Complete)

All 18 parser files compile successfully:

- `parser/index.fab` - Parsator (Resolvitor implementation), entry point
- `parser/nucleus.fab` - Core Parser genus with token stream state
- `parser/errores.fab` - Error codes and reporting
- `parser/resolvitor.fab` - Resolvitor pactum for mutual recursion
- `parser/typus.fab` - Type annotations with generics, nullable, array shorthand
- `expressia/index.fab` - Expression entry point
- `expressia/binaria.fab` - Full precedence chain (assignment → ternary → logical → comparison → arithmetic)
- `expressia/unaria.fab` - Prefix operators (non, -, ~, cede, novum), postfix (call, member, qua)
- `expressia/primaria.fab` - Literals, identifiers, ego, arrays, objects, lambdas, grouped expressions
- `sententia/index.fab` - Statement dispatcher routing to all parsers
- `sententia/declara.fab` - functio, genus, pactum, ordo, discretio, typus, importa
- `sententia/imperium.fab` - si/sin/secus, dum, ex...pro, de...pro
- `sententia/actio.fab` - redde, rumpe, perge, iace, scribe/vide/mone
- `sententia/error.fab` - tempta/cape/demum, fac...dum, adfirma
- `sententia/fluxus.fab` - elige (switch), discerne (pattern match), custodi (guard)
- `sententia/initus.fab` - incipit/incipiet (entry points), cura (resources), ad (dispatch)
- `sententia/massa.fab` - Block parsing, program parsing
- `sententia/varia.fab` - Variable declarations, destructuring
- `sententia/varia.fab` - Variable declarations, destructuring
- `sententia/fluxus.fab` - elige (switch), discerne (pattern match), custodi (guard)
- `sententia/initus.fab` - incipit/incipiet (entry points), cura (resources), ad (dispatch)

**Not yet implemented:**

- Testing: `probandum`, `proba`, `praepara` (not needed for bootstrap)

### Semantic (17 files, all compile)

All 17 semantic files compile successfully.

Files:

- `index.fab`, `nucleus.fab`, `errores.fab`, `scopus.fab`, `typi.fab`, `resolvitor.fab`
- `expressia/`: `index.fab`, `alia.fab`, `binaria.fab`, `primaria.fab`, `unaria.fab`, `vocatio.fab`
- `sententia/`: `index.fab`, `actio.fab`, `declara.fab`, `error.fab`, `imperium.fab`

Note: Module resolution (`modules.ts`) skipped — requires file I/O not yet in Faber.

### Codegen (6 files, complete)

| File                             | Status | Notes                                   |
| -------------------------------- | ------ | --------------------------------------- |
| `codegen/typi.fab`               | ✅     | RequiredFeatures, CodegenOptions, utils |
| `codegen/ts/nucleus.fab`         | ✅     | TsGenerator genus with state            |
| `codegen/ts/typus.fab`           | ✅     | Latin → TS type mapping                 |
| `codegen/ts/expressia/index.fab` | ✅     | All 24 expression types handled         |
| `codegen/ts/sententia/index.fab` | ✅     | Uses scriptum() for all formatting      |
| `codegen/ts/index.fab`           | ✅     | Public API entry point                  |

### CLI (1 file, complete)

| File      | Status | Notes                       |
| --------- | ------ | --------------------------- |
| `cli.fab` | ✅     | Minimal stdin→stdout driver |

### Remaining Work

Fix TypeScript type-check errors (299 remaining). See "TypeScript Type-Check Status" section above.

## Bootstrap Strategy

### Phase 1: Parser (`fons-fab/parser/`) — COMPLETE

1. ✅ Create `genus Parser` with token stream state
2. ✅ Create `pactum Resolvitor` for mutual recursion
3. ✅ Implement `Parsator` (Resolvitor implementation)
4. ✅ Restructure AST as `discretio` variants with `finge`
5. ✅ Expression parsers (binary, unary, postfix, primary, objects, lambdas)
6. ✅ Statement dispatcher and all statement parsers
7. ✅ Pattern matching (elige, discerne, custodi), entry points (incipit, incipiet), resources (cura, ad)

**18 files (4,134 lines), all compiling successfully.**

### Phase 2: Semantic Analyzer (`fons-fab/semantic/`) — COMPLETE

Port `fons/semantic/`:

1. ✅ Type system (`typi.fab`) — SemanticTypus discretio, constructors, utilities
2. ✅ Error catalog (`errores.fab`) — Error codes and message functions
3. ✅ Scope management (`scopus.fab`) — Symbol tables, scope chain
4. ✅ Analyzer state (`nucleus.fab`) — Analyzator genus with scope/error state
5. ✅ Resolvitor interface (`resolvitor.fab`) — Breaks circular imports
6. ✅ Expression resolution (`expressia/`) — Type inference for all expressions
7. ✅ Statement analysis (`sententia/`) — Type checking for all statements
8. ✅ Entry point (`index.fab`) — Main `analyze()` function

**17 files (2,844 lines), all compile successfully.**

### Phase 3: TypeScript Codegen (`fons-fab/codegen/ts/`) — COMPLETE

Port only the TS target from `fons/codegen/ts/`:

1. ✅ Generator class with state (`depth`, `inGenerator`, `inFlumina`, etc.) — `nucleus.fab`
2. ✅ Type emission (Latin → TypeScript type mapping) — `typus.fab`
3. ✅ Expression dispatcher — `expressia/index.fab`
4. ✅ Statement dispatcher — `sententia/index.fab` (uses scriptum() workaround)
5. ✅ Public API entry point — `index.fab`

**6 files (1,859 lines), all compile successfully.**

**Architecture decisions:**

- **Keep current file structure** — One file per node type mirrors the TS codebase
- **Drop `semi` parameter** — Hardcode semicolons; no Faber code uses configurable semicolons
- **Simplify `RequiredFeatures`** — Keep: `lista`, `tabula`, `copia`, `flumina`, `decimal`, `regex`. Drop Python/C++ specific fields.
- **Port as-is first** — Refactor after bootstrap works, not before
- **Use `ignotum` for nullable params** — Faber doesn't support `Type?` in parameter position, use `ignotum` and cast inside function

**Target languages (post-bootstrap):**

| Target | Keep | Rationale                                                                   |
| ------ | ---- | --------------------------------------------------------------------------- |
| TS     | ✅   | Bootstrap, web, primary target                                              |
| Zig    | ✅   | Native, systems, explicit memory                                            |
| Rust   | ✅   | Native alternative, WASM, ownership model aligns with Faber's `de`/`in`     |
| Fab    | ✅   | Self-hosting, canonical formatting                                          |
| Python | ❌   | Dynamic mismatch, maintenance burden, 7+ special fields in RequiredFeatures |
| C++    | ❌   | No audience, no compelling differentiator                                   |

After bootstrap: remove `fons/codegen/py/` and `fons/codegen/cpp/`.

### Phase 4: CLI (`fons-fab/cli.fab`) — COMPLETE

Minimal stdin→stdout compiler driver. **67 lines.**

### Phase 5: Integration — IN PROGRESS

1. ✅ Compile `fons-fab/*.fab` with TS compiler → `opus/bootstrap/*.ts`
2. 🔄 Fix TypeScript type-check errors (299 remaining)
3. ⬚ Run with Bun, verify it compiles test files correctly
4. ⬚ Self-compile: use Faber compiler to compile itself
5. ⬚ Verify round-trip: both compilers produce identical output

## Key Patterns

### Resolvitor Pattern

Mutual recursion solved via interface:

```fab
ex "./resolvitor" importa Resolvitor, Expressia

functio parseCondicio(Resolvitor r) -> Expressia {
    fixum p = r.parser()
    fixum test = parseAut(r)

    si p.congruetVerbum("sic") {
        fixum consequens = r.expressia()  # Cross-module call via Resolvitor
        # ...
    }
}
```

### Closure → Genus Refactoring

TypeScript closures become `genus` with methods:

```typescript
// TypeScript
function parse(tokens: Token[]) {
    let current = 0;
    function advance() {
        return tokens[current++];
    }
}
```

```fab
# Faber
genus Parser {
    lista<Symbolum> symbola
    numerus index

    functio procede() -> Symbolum {
        fixum s = ego.symbola[ego.index]
        ego.index = ego.index + 1
        redde s
    }
}
```

### Latin Naming

| English      | Latin       | Usage                  |
| ------------ | ----------- | ---------------------- |
| `parse`      | `resolvere` | Parse/resolve          |
| `current`    | `index`     | Current position       |
| `tokens`     | `symbola`   | Token list             |
| `peek`       | `specta`    | Look without consuming |
| `advance`    | `procede`   | Move forward           |
| `check`      | `proba`     | Check/test             |
| `match`      | `congruet`  | Match and consume      |
| `expect`     | `expecta`   | Require or error       |
| `report`     | `renuncia`  | Report error           |
| `left`       | `sinister`  | Left operand           |
| `right`      | `dexter`    | Right operand          |
| `operator`   | `signum`    | Operator sign          |
| `body`       | `corpus`    | Block body             |
| `expression` | `expressia` | Expression             |
| `statement`  | `sententia` | Statement              |

### File Organization

```
parser/
├── index.fab              # Public API, Parsator (Resolvitor impl)
├── nucleus.fab            # Core Parser genus
├── errores.fab            # Error codes
├── resolvitor.fab         # Resolvitor pactum
├── typus.fab              # Type annotation parser
├── sententia/             # Statement parsers
│   ├── index.fab          # Dispatcher
│   ├── actio.fab          # redde, rumpe, perge, iace, scribe
│   ├── declara.fab        # functio, genus, pactum, ordo, discretio, typus, importa
│   ├── error.fab          # tempta/cape/demum, fac, adfirma
│   ├── fluxus.fab         # elige, discerne, custodi
│   ├── imperium.fab       # si, dum, ex...pro, de...pro
│   ├── initus.fab         # incipit, incipiet, cura, ad
│   ├── massa.fab          # Blocks, program
│   └── varia.fab          # Variable declarations
└── expressia/             # Expression parsers
    ├── index.fab          # Entry point
    ├── binaria.fab        # Full precedence chain
    ├── unaria.fab         # Prefix and postfix
    └── primaria.fab       # Terminals (literals, identifiers, objects, lambdas)
```

## Lessons Learned

### Session 1: Parser Foundation

1. **Use generous comments** — English comments inside functions explain control flow since all identifiers are Latin.
2. **Subdirectories help** — Breaking the parser into `sententia/` and `expressia/` makes files easier to find.
3. **Error catalog is essential** — Porting error codes early provides consistent infrastructure.
4. **TODO stubs are fine** — Allows incremental progress.

### Session 2: Module Imports

1. **Local imports now work** — `ex "./path" importa Type, func` resolves relative `.fab` files.
2. **Remove Zig-specific patterns** — Stripped `curata alloc` from lexor.

### Session 3: Resolvitor Pattern

1. **Pactum solves circular deps** — The `pactum Resolvitor` pattern cleanly separates interface from implementation.
2. **`finge` enables discretio AST** — With `finge ... qua Expressia|Sententia`, the bootstrap can return real discretio variants.
3. **Two-pass semantic analysis** — Functions can now be called before definition (forward references work).
4. **`fac...dum` for do-while** — Use `fac { body } dum condition` for loops that execute at least once.
5. **Keywords as identifiers** — Keywords like `typus`, `genus` can be used as variable/field names.
6. **Keyword-as-identifier has multiple code paths** — There are three places that decide whether a keyword can be an identifier: (1) `parseVariaDeclaration()` uses `parseIdentifierOrKeyword()`, (2) field/param declarations use `parseIdentifierOrKeyword()`, (3) `parsePrimary()` has its own hardcoded `statementKeywords` blocklist. This is an architectural smell — a cleaner design would have a single `isContextualKeyword(kw)` function that all sites consult. For now, just ensure the blocklist only contains true statement-starting keywords (like `si`, `dum`, `redde`), not contextual keywords that are only meaningful within specific constructs (like `cape`/`demum` which only matter inside `tempta`/`fac`).

### Session 4: AST Restructure

1. **Single-file discretio** — Consolidated AST into `expressia.fab` (24 variants) and `sententia.fab` (31 variants).
2. **Supporting types stay as genus** — `CapeClausula`, `Parametrum`, `ObiectumProprietas` etc. are reusable building blocks.
3. **Parser integration verified** — Updated `actio.fab` and `primaria.fab` to use `finge`, generates correct tagged unions.
4. **Generated code** — `finge Littera { ... } qua Expressia` produces `{ tag: 'Littera', ... }` in TypeScript.

### Session 5: Parser Complete

1. **Avoid `typus` as variable name** — The keyword `typus` (used for type aliases) confuses the parser when used as a variable. Use `adnotatioTypus` or similar instead.
2. **Object literals and lambdas** — `parseObiectumExpressia()` supports shorthand, computed keys, spread. `parseLambdaExpressia()` handles params and return types.
3. **All control flow complete** — elige (switch), discerne (pattern match), custodi (guard), si/sin/secus chains all implemented.
4. **Entry points and resources** — incipit/incipiet for program entry, cura for resource management with automatic cleanup.
5. **27 files total** — Parser phase complete with full coverage of Faber syntax needed for self-hosting.

### Session 6: Codegen Started

1. **Method call return types broken** — `g.ind()` where `ind() -> textus` incorrectly infers as `numerus`. Workaround: use `scriptum()` for all string formatting instead of concatenation with method calls.
2. **Discerne bindings typed as unknown** — Pattern matching `si Variant pro a, b, c { }` gives bindings type `unknown`, not field types. Workaround: cast bindings explicitly `a qua textus`.
3. **No nullable function parameters** — `Type?` syntax only valid for fields and return types, not parameters. Use `ignotum` type and cast inside function.
4. **Disabled prettier** — Removed archived prettier import from CLI; `faber format` returns "not implemented" for now.
5. **Use scriptum() everywhere** — Instead of string concatenation like `g.ind() + "if (" + cond + ")"`, use `scriptum("{}if ({})", g.ind(), cond)`. This avoids the method call return type bug entirely and produces cleaner code.
6. **Scriptum brace escaping** — Use `{{` for literal `{` and `}}` for literal `}`. Example: `scriptum("{{ {} }}", val)` produces `{ value }`. This is needed when generating JS object literals or destructuring.

### Session 7: Bootstrap Verification

1. **Actual file count is 49** — Not 27 as previously documented. Includes 6 AST, 1 lexicon, 2 lexor, 18 parser, 17 semantic, 5 codegen.
2. **Total ~10,644 lines** — Previous estimates were understated.
3. **All 49 files compile** — Discretio return type bug was fixed.
4. **Discretio return type bug (fixed)** — Functions returning `finge Variant {...} qua Discretio` were failing type checking. The semantic analyzer now correctly recognizes that `Discretio` and `discretio Discretio` are the same type.
5. **Missing `ts/index.fab`** — Codegen entry point not yet created.

### Session 8: Discerne Binding Semantics

1. **`pro` vs `ut` clarified** — `si Variant pro x, y` extracts fields positionally (x=first field, y=second). `si Variant ut v` binds the whole variant for `v.field` access.
2. **Bootstrap was using `pro` incorrectly** — 652 occurrences of `si Variant pro x { x.field }` needed to be `si Variant ut x { x.field }`.
3. **Codegen updated** — `discerne.ts` now correctly generates `const x = e.fieldName;` for `pro` bindings when type info is available, falls back to `const { x } = e;` (assumes binding name matches field name) when type info is unavailable.
4. **All fons-fab files migrated** — Changed `pro` to `ut` for single-binding patterns that access fields on the binding.

### Session 9: TypeScript Type-Checking

1. **All 51 files now compile** — Added `ts/index.fab` and `cli.fab` to complete the bootstrap source.
2. **TypeScript type-checking started** — Compiling to `opus/bootstrap/` and running `tsc --noEmit` reveals 299 type errors.
3. **Export annotations needed** — `discretio` and `ordo` declarations need `@ publicum` to generate TypeScript `export`.
4. **Enum member names must match exactly** — `SymbolumGenus.Semicolon` doesn't exist; the enum defines `PunctumColon`. Latin consistency matters.
5. **Stdlib method names from registry** — Collection methods like `.iunge()` don't exist; must use registry names (`.coniunge()`). Check `fons/codegen/lista.ts` and `fons/codegen/tabula.ts` for valid method names.
6. **Discriminated union narrowing issue** — TypeScript's type narrowing with `discerne`/`si...ut` doesn't work as expected; many "property does not exist on type 'never'" errors suggest the codegen for pattern matching needs review.
7. **Private method accessibility** — Methods defined in a `genus` are private by default; external callers get "property is private" errors. May need `@ publicum` on methods or architectural changes.

## Build Commands

```bash
# Compile all fons-fab files to opus/bootstrap/
for f in fons-fab/**/*.fab; do
  outfile="opus/bootstrap/${f#fons-fab/}"; outfile="${outfile%.fab}.ts"
  mkdir -p "$(dirname "$outfile")"
  bun run faber compile "$f" -o "$outfile"
done

# TypeScript type-check (299 errors remaining)
cd opus/bootstrap && npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler cli.ts

# Run compiled compiler (once type-checking passes)
bun opus/bootstrap/cli.ts < input.fab > output.ts

# Self-compile (once working)
bun opus/bootstrap/cli.ts < fons-fab/**/*.fab > opus2/
diff -r opus/bootstrap/ opus2/  # Should be identical
```

## Success Criteria

1. `fons-fab/` compiles to valid TypeScript
2. Compiled compiler passes existing test suite
3. Self-compilation produces identical output
4. No runtime dependencies beyond Bun

## Timeline

| Phase     | Actual Lines | Status                        |
| --------- | -----------: | ----------------------------- |
| AST       |        1,112 | ✅ Complete (6 files)         |
| Lexicon   |          221 | ✅ Complete (1 file)          |
| Lexor     |          853 | ✅ Complete (2 files)         |
| Parser    |        4,134 | ✅ Complete (18 files)        |
| Semantic  |        2,844 | ✅ Complete (17 files)        |
| Codegen   |        1,859 | ✅ Complete (6 files)         |
| CLI       |           67 | ✅ Complete (1 file)          |
| **Total** |   **11,090** | **51/51 files compile to TS** |

## Next Steps

1. **Fix TypeScript type-check errors** — 299 errors remaining (see error categories above)
2. **Integration testing** — Run bootstrap compiler, verify it produces correct output
3. **Self-compilation** — Compile fons-fab with itself, verify round-trip

## Design Decisions Log

### 2026-01-01: Target Language Reduction

**Decision:** Drop Python and C++ as codegen targets. Keep TS, Zig, Rust, Fab.

**Rationale:**

- **Python:** Dynamic typing fights Faber's static model. Heavy maintenance burden (7+ Python-specific fields in `RequiredFeatures`, special syntax handling everywhere). No clear audience — Python users write Python.
- **C++:** No compelling use case. "Because it exists" isn't a roadmap.
- **Rust stays:** Shares borrowing semantics with Zig (aligns with `de`/`in` prepositions), has mindshare, provides WASM path.

**Impact:** Reduces target count from 6 to 4. Every language feature now costs 4x instead of 6x implementation effort. `RequiredFeatures` can drop ~10 Python-specific fields.

### 2026-01-01: Codegen Architecture

**Decision:** Port TS codegen as-is. No refactoring before bootstrap.

**Rationale:**

- Current design works. It's not elegant but it's mechanical.
- Refactoring in TypeScript before bootstrap is wasted effort — the refactored code would need porting anyway.
- Once bootstrap works, refactor in Faber itself (dog-fooding).

**Specific changes:**

- Drop `semi` parameter (hardcode `true` for TS)
- Simplify `RequiredFeatures` to: `lista`, `tabula`, `copia`, `flumina`, `decimal`, `regex`
- Accept dispatch switch duplication for now — Faber's `discerne` will clean it up post-bootstrap
