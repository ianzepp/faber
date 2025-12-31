# Bootstrap: Self-Hosted Faber Compiler (TypeScript Target)

Rewrite the Faber compiler in Faber, targeting TypeScript/Bun.

## Rationale

1. **Path of least resistance** — TS codegen is the most mature (632 test expectations)
2. **Same runtime** — Bun runs both current compiler and output
3. **No memory management** — GC handles allocation; no `cura`/`curata` complexity
4. **Faster iteration** — Fix issues in Faber, recompile, test immediately

## Current State

### Completed

| Module | Location | Lines | Status |
|--------|----------|-------|--------|
| AST types | `fons-fab/ast/` | ~700 | Complete (21 files) |
| Lexer | `fons-fab/lexor/` | ~760 | Complete |
| Keywords | `fons-fab/lexicon/` | ~135 | Complete |
| Parser (skeleton) | `fons-fab/parser/` | ~1,470 | In progress |

### Parser Progress

| Component | File(s) | Lines | Status |
|-----------|---------|-------|--------|
| Core state | `parser/nucleus.fab` | ~340 | Complete |
| Error catalog | `parser/errores.fab` | ~280 | Complete |
| Type parser | `parser/typus.fab` | ~165 | Complete |
| Public API | `parser/index.fab` | ~90 | Complete |
| Statement dispatch | `sententia/index.fab` | ~340 | Partial (stubs) |
| Action statements | `sententia/actio.fab` | ~115 | Complete |
| Error statements | `sententia/error.fab` | ~130 | Complete |
| Block/program | `sententia/massa.fab` | ~110 | Complete |
| Variable decls | `sententia/varia.fab` | ~220 | Complete |
| Expression entry | `expressia/index.fab` | ~70 | Complete |
| Binary operators | `expressia/binaria.fab` | ~360 | Complete |
| Unary/postfix | `expressia/unaria.fab` | ~340 | Complete |
| Primary | `expressia/primaria.fab` | ~340 | Complete |

### Remaining Parser Work

Statement parsers with TODO stubs:
- Import declarations (`ex ... importa`)
- Function declarations (`functio`)
- Type declarations (`typus`, `ordo`, `genus`, `pactum`, `discretio`)
- Control flow (`si`, `dum`, `ex...pro`, `de...pro`, `in`)
- Pattern matching (`elige`, `discerne`, `custodi`)
- Tests (`probandum`, `proba`, `praepara`)
- Entry points (`incipit`, `incipiet`, `cura`, `ad`)

### Remaining Modules

| Module | Source | Est. Lines | Notes |
|--------|--------|------------|-------|
| Parser (complete) | `fons/parser/index.ts` | ~3,500 more | Remaining statements |
| Semantic | `fons/semantic/` | ~2,600 | Type checking, scopes |
| Codegen (TS) | `fons/codegen/ts/` | ~2,000 | TS target only |
| CLI | `fons/cli.ts` | ~600 | Entry point |

## Bootstrap Strategy

### Phase 1: Parser (`fons-fab/parser/`) — IN PROGRESS

Port `fons/parser/index.ts` to Faber:

1. ✅ Create `genus Parser` with token stream state
2. ✅ Port parsing functions as methods
3. ✅ Refactor closures → struct methods
4. ⏳ Complete remaining statement parsers

The parser is organized into subdirectories:
- `parser/` — core infrastructure
- `parser/sententia/` — statement parsers
- `parser/expressia/` — expression parsers

### Phase 2: Semantic Analyzer (`fons-fab/semantic/`)

Port `fons/semantic/`:

1. Symbol tables using `tabula<textus, Symbol>`
2. Scope stack management
3. Type inference and checking

### Phase 3: TypeScript Codegen (`fons-fab/codegen/ts/`)

Port only the TS target from `fons/codegen/ts/`:

1. Statement generators
2. Expression generators
3. Type emission

Skip other targets (py, rs, cpp, zig, fab) — they can be added later.

### Phase 4: CLI (`fons-fab/cli.fab`)

Minimal CLI:

```faber
functio main(lista<textus> args) -> numerus {
    fixum source = lege()  // stdin
    fixum result = compile(source)
    scribe result          // stdout
    redde 0
}
```

### Phase 5: Integration

1. Compile `fons-fab/*.fab` with TS compiler → `opus/*.ts`
2. Run with Bun, verify it compiles test files correctly
3. Self-compile: use Faber compiler to compile itself
4. Verify round-trip: both compilers produce identical output

## Key Patterns

### Closure → Genus Refactoring

TypeScript closures become `genus` with methods:

```typescript
// TypeScript
function parse(tokens: Token[]) {
    let current = 0;
    function advance() { return tokens[current++]; }
    // ...
}
```

```faber
// Faber
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

All bootstrap code uses Latin identifiers:

| English | Latin | Usage |
|---------|-------|-------|
| `parse` | `resolvere` | Parse/resolve |
| `current` | `index` | Current position |
| `tokens` | `symbola` | Token list |
| `error` | `error` | Error (Latin origin) |
| `peek` | `specta` | Look without consuming |
| `advance` | `procede` | Move forward |
| `check` | `proba` | Check/test |
| `match` | `congruet` | Match and consume |
| `expect` | `expecta` | Require or error |
| `report` | `renuncia` | Report error |
| `synchronize` | `synchrona` | Error recovery |
| `left` | `sinister` | Left operand |
| `right` | `dexter` | Right operand |
| `operator` | `signum` | Operator sign |
| `body` | `corpus` | Block body |
| `expression` | `expressia` | Expression |
| `statement` | `sententia` | Statement |

### Result Types

Functions return result types instead of throwing:

```faber
genus ParserResultatum {
    Programma? programma
    lista<ParserError> errores
}

functio resolvere(lista<Symbolum> symbola) -> ParserResultatum {
    // ...
}
```

### File Organization

Keep files small (~150-350 lines) for maintainability:

```
parser/
├── index.fab              # Public API (~90 lines)
├── nucleus.fab            # Core Parser genus (~340 lines)
├── errores.fab            # Error codes (~280 lines)
├── typus.fab              # Type annotation parser (~165 lines)
├── sententia/             # Statement parsers
│   ├── index.fab          # Dispatcher (~340 lines)
│   ├── actio.fab          # Actions (~115 lines)
│   ├── error.fab          # Error handling (~130 lines)
│   ├── massa.fab          # Blocks (~110 lines)
│   └── varia.fab          # Variables (~220 lines)
└── expressia/             # Expression parsers
    ├── index.fab          # Entry point (~70 lines)
    ├── binaria.fab        # Binary ops (~360 lines)
    ├── unaria.fab         # Unary/postfix (~340 lines)
    └── primaria.fab       # Primaries (~340 lines)
```

## Lessons Learned

### Session 1: Parser Foundation

1. **Use generous comments** — English comments inside functions explain control flow since all identifiers are Latin. This makes the code accessible to humans while maintaining Latin naming.

2. **Forward declarations** — Faber doesn't have module imports yet, so forward-declare functions that will be defined in other files. This establishes the contract.

3. **Subdirectories help** — Breaking the parser into `sententia/` and `expressia/` subdirectories makes files easier to find than one giant file.

4. **Error catalog is essential** — Porting the error codes early (`errores.fab`) provides consistent error handling infrastructure.

5. **Start with expression parsers** — Expressions are more self-contained than statements. Getting precedence climbing working first provides a solid foundation.

6. **TODO stubs are fine** — The statement dispatcher can return placeholder `parseExpressiaSententia(p)` for unimplemented statements. This allows incremental progress.

7. **Section comments in long functions** — Use `// =========` dividers to organize long functions into logical sections.

8. **Type annotation parser is foundational** — Many parsers need type annotations, so port this early.

## Build Commands

```bash
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

| Phase | Scope | Est. Days | Status |
|-------|-------|-----------|--------|
| Parser | ~5,000 lines | 5-7 | ~30% complete |
| Semantic | ~2,600 lines | 3-4 | Not started |
| Codegen | ~2,000 lines | 3-4 | Not started |
| CLI | ~600 lines | 1 | Not started |
| Integration | Debug, iterate | 2-3 | Not started |
| **Total** | | **14-19 days** | |

Significantly faster than Zig bootstrap (~20-28 days) due to:
- No allocator threading
- Mature TS codegen
- Simpler error handling (exceptions work)
