# Bootstrap: Self-Hosted Faber Compiler (TypeScript Target)

Rewrite the Faber compiler in Faber, targeting TypeScript/Bun.

## Rationale

1. **Path of least resistance** — TS codegen is the most mature (632 test expectations)
2. **Same runtime** — Bun runs both current compiler and output
3. **No memory management** — GC handles allocation; no `cura`/`curata` complexity
4. **Faster iteration** — Fix issues in Faber, recompile, test immediately

## Current State

**No blocking issues remain.** All previously identified blockers have been resolved:

- ✅ Mutual recursion → solved with `pactum Resolvitor`
- ✅ Discretio instantiation → solved with `finge` keyword
- ✅ Function hoisting → solved with two-pass semantic analysis
- ✅ Do-while loops → implemented as `fac { } dum condition`
- ✅ AST as discretio → unified `Expressia` (24 variants) and `Sententia` (31 variants)

### Compiles Successfully

| Module               | Location                 | Files | Status                     |
| -------------------- | ------------------------ | ----- | -------------------------- |
| AST types            | `fons-fab/ast/`          | 6     | Complete (discretio-based) |
| Lexer                | `fons-fab/lexor/`        | 2     | Complete                   |
| Keywords             | `fons-fab/lexicon/`      | 1     | Complete                   |
| Parser errors        | `parser/errores.fab`     | 1     | Complete                   |
| Parser core          | `parser/nucleus.fab`     | 1     | Complete                   |
| Resolvitor interface | `parser/resolvitor.fab`  | 1     | Complete                   |
| Type parser          | `parser/typus.fab`       | 1     | Complete                   |
| Parser entry         | `parser/index.fab`       | 1     | Complete (Parsator wired)  |
| Statement dispatch   | `sententia/index.fab`    | 1     | Complete                   |
| Declarations         | `sententia/declara.fab`  | 1     | Complete                   |
| Control flow         | `sententia/imperium.fab` | 1     | Complete                   |
| Pattern matching     | `sententia/fluxus.fab`   | 1     | Complete                   |
| Entry/resources      | `sententia/initus.fab`   | 1     | Complete                   |
| Action statements    | `sententia/actio.fab`    | 1     | Complete                   |
| Error statements     | `sententia/error.fab`    | 1     | Complete                   |
| Block/program        | `sententia/massa.fab`    | 1     | Complete                   |
| Variable decls       | `sententia/varia.fab`    | 1     | Complete                   |
| Expression entry     | `expressia/index.fab`    | 1     | Complete                   |
| Binary operators     | `expressia/binaria.fab`  | 1     | Complete                   |
| Unary/postfix        | `expressia/unaria.fab`   | 1     | Complete                   |
| Primary expressions  | `expressia/primaria.fab` | 1     | Complete                   |

**All 27 fons-fab files compile successfully (~6,259 lines).**

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

### Remaining Parser Work

**Fully implemented:**

- `parser/index.fab` - Parsator (Resolvitor implementation), entry point
- `parser/typus.fab` - Type annotations with generics, nullable, array shorthand
- `expressia/binaria.fab` - Full precedence chain (assignment → ternary → logical → comparison → arithmetic)
- `expressia/unaria.fab` - Prefix operators (non, -, ~, cede, novum), postfix (call, member, qua)
- `expressia/primaria.fab` - Literals, identifiers, ego, arrays, objects, lambdas, grouped expressions
- `sententia/index.fab` - Statement dispatcher routing to all parsers
- `sententia/declara.fab` - functio, genus, pactum, ordo, discretio, typus, importa
- `sententia/imperium.fab` - si/sin/secus, dum, ex...pro, de...pro
- `sententia/actio.fab` - redde, rumpe, perge, iace, scribe/vide/mone
- `sententia/error.fab` - tempta/cape/demum, fac...dum, adfirma
- `sententia/massa.fab` - Block parsing, program parsing
- `sententia/varia.fab` - Variable declarations, destructuring
- `sententia/fluxus.fab` - elige (switch), discerne (pattern match), custodi (guard)
- `sententia/initus.fab` - incipit/incipiet (entry points), cura (resources), ad (dispatch)

**Not yet implemented:**

- Testing: `probandum`, `proba`, `praepara` (not needed for bootstrap)

### Remaining Modules

| Module  | Source             | Est. Lines | Notes          |
| ------- | ------------------ | ---------- | -------------- |
| Codegen | `fons/codegen/ts/` | ~2,000     | TS target only |
| CLI     | `fons/cli.ts`      | ~600       | Entry point    |

## Bootstrap Strategy

### Phase 1: Parser (`fons-fab/parser/`) — COMPLETE

1. ✅ Create `genus Parser` with token stream state
2. ✅ Create `pactum Resolvitor` for mutual recursion
3. ✅ Implement `Parsator` (Resolvitor implementation)
4. ✅ Restructure AST as `discretio` variants with `finge`
5. ✅ Expression parsers (binary, unary, postfix, primary, objects, lambdas)
6. ✅ Statement dispatcher and all statement parsers
7. ✅ Pattern matching (elige, discerne, custodi), entry points (incipit, incipiet), resources (cura, ad)

**27 files, all compiling successfully.**

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

**17 files (~2,000 lines), all compiling successfully.**

Note: Module resolution (`modules.ts`) skipped — requires file I/O not yet in Faber.

### Phase 3: TypeScript Codegen (`fons-fab/codegen/ts/`)

Port only the TS target from `fons/codegen/ts/`:

1. Statement generators
2. Expression generators
3. Type emission

Skip other targets (py, rs, cpp, zig, fab) — they can be added later.

### Phase 4: CLI (`fons-fab/cli.fab`)

Minimal CLI:

```fab
functio main(lista<textus> args) -> numerus {
    fixum source = lege()  # stdin
    fixum result = compile(source)
    scribe result          # stdout
    redde 0
}
```

### Phase 5: Integration

1. Compile `fons-fab/*.fab` with TS compiler → `opus/*.ts`
2. Run with Bun, verify it compiles test files correctly
3. Self-compile: use Faber compiler to compile itself
4. Verify round-trip: both compilers produce identical output

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

## Build Commands

```bash
# Check all parser files compile
for f in fons-fab/parser/**/*.fab; do bun run faber check "$f"; done

# Compile bootstrap to TypeScript
bun run faber compile fons-fab/**/*.fab -t ts -o opus/

# Run compiled compiler
bun opus/cli.ts < input.fab > output.ts

# Self-compile (once working)
bun opus/cli.ts < fons-fab/**/*.fab > opus2/
diff -r opus/ opus2/  # Should be identical
```

## Success Criteria

1. `fons-fab/` compiles to valid TypeScript
2. Compiled compiler passes existing test suite
3. Self-compilation produces identical output
4. No runtime dependencies beyond Bun

## Timeline

| Phase       | Scope        | Est. Days      | Status      |
| ----------- | ------------ | -------------- | ----------- |
| Parser      | ~6,259 lines | 5-7            | ✅ Complete |
| Semantic    | ~2,000 lines | 3-4            | ✅ Complete |
| Codegen     | ~2,000 lines | 3-4            | Not started |
| CLI         | ~600 lines   | 1              | Not started |
| Integration | Debug, iter  | 2-3            | Not started |
| **Total**   |              | **14-19 days** |             |

## Next Steps

1. **Phase 3: TypeScript Codegen** — Port `fons/codegen/ts/` to Faber
2. **Phase 4: CLI** — Create `fons-fab/cli.fab` entry point
