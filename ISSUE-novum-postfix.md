# Issue: Add postfix `{...} novum Type` construction syntax

## Summary

Add postfix object construction syntax `{...} novum Type` as the standard way to construct class instances from object literals. This complements the existing `{...} qua Type` casting syntax and resolves ambiguity that causes LLMs (and humans) to misuse `qua` for construction.

## Motivation

### Current state
- `qua` is for typecasting: `{...} qua Type` emits `{...} as Type`
- `novum Type {...}` is prefix construction: emits `new Type({...})`
- `innatum` is for builtin construction only (lista, copia, tabula)

### Problem
1. **LLM confusion**: LLMs consistently use `{...} qua Type` when they want construction, not casting
2. **Semantic ambiguity**: `qua` (casting) looks identical to construction in source, but has different TypeScript semantics
3. **rivus compilation**: nanus compilers emit `{...} as Type` for `qua`, but classes need `new Type({...})` to work correctly with private fields
4. **faber magic**: faber silently transforms `{...} qua GenusName` into `new GenusName({...})` - this magic transformation is undocumented and prevents nanus from compiling rivus

### Solution
Add postfix construction syntax that parallels casting:
```faber
fixum loc = { linea: 1, columna: 1, index: 0 } novum Locus   # construction
fixum loc = { linea: 1, columna: 1, index: 0 } qua Locus     # casting (rare)
```

## Syntax

```
ObjectLiteral "novum" TypeName  →  new TypeName(ObjectLiteral)
ObjectLiteral "qua" TypeName    →  ObjectLiteral as TypeName
```

Both are postfix operators on object literals, making the distinction clear and learnable.

## Scope of changes

### 1. Lexer
- No changes needed - `novum` is already a keyword

### 2. Parser (all compilers)
After parsing an object literal, check for trailing `novum Type` in addition to `qua Type`.

**Files:**
- `fons/faber-ts/parser/` - faber parser
- `fons/rivus/parser/` - rivus parser
- `fons/nanus-ts/parser.ts` - nanus TypeScript
- `fons/nanus-go/parser.go` - nanus Go

### 3. AST
Add new node type or extend existing cast node:
```
PostfixConstruction {
    object: ObjectLiteral
    typeName: TypeName
}
```

Or reuse/extend existing `QUA` expression with a `isConstruction: boolean` flag.

### 4. Emitter (all compilers)
- `{...} novum Type` → `new Type({...})`
- `{...} qua Type` → `{...} as Type` (no change)

**Files:**
- `fons/faber-ts/codegen/ts/` - faber emitter
- `fons/rivus/codegen/ts/` - rivus emitter
- `fons/nanus-ts/emitter.ts` - nanus TypeScript
- `fons/nanus-go/emitter.go` - nanus Go

### 5. Remove faber magic
Remove the `genusNames` tracking and automatic `qua` → `new` transformation in faber.

**File:** `fons/faber-ts/codegen/ts/statements/genus.ts` (lines 33-34 and related)

### 6. Update rivus source
Replace `{...} qua GenusName` with `{...} novum GenusName` for all genus (class) types.

**Scope:** 225 replacements across ~59 genus types

**Top targets by frequency:**
| Genus | Count |
|-------|-------|
| Symbolum | 25 |
| SemanticErrorNuntius | 20 |
| Annotatio | 12 |
| UsitataFeatura | 11 |
| ModulusExportum | 9 |
| TypusAnnotatio | 8 |
| TypusParametrum | 8 |
| Various *Capacitas | 7 each |

**Keep as `qua`:** 157 patterns targeting `discretio` or `pactum` types (Expressia, Sententia, SemanticTypus, etc.)

### 7. Update golden tests
Add test cases for `{...} novum Type` syntax.

### 8. Documentation
- Update language reference
- Add examples showing `qua` vs `novum` distinction

## Migration strategy

1. **Phase 1**: Add `{...} novum Type` support to all parsers/emitters (backward compatible)
2. **Phase 2**: Update rivus source to use `novum` for genus construction
3. **Phase 3**: Remove faber's magic `qua` → `new` transformation
4. **Phase 4**: Deprecate prefix `novum Type {...}` syntax (optional, low priority)

## Verification

After all changes:
```bash
bun run build:nanus-ts
bun run build:nanus-go
bun run build:rivus -- -c nanus-ts   # Should pass
bun run build:rivus -- -c nanus-go   # Should pass
bun run golden -c nanus-ts
bun run golden -c nanus-go
bun run golden -c faber
```

## Assignee

TBD

## Labels

- `language-syntax`
- `breaking-change`
- `rivus`
- `nanus`

## Related

- Removes dependency on faber for rivus compilation
- Enables nanus as sole bootstrap compiler
- Unblocks faber deprecation
