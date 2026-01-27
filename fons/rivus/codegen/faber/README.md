# Faber Codegen Target

Pretty-printer that emits canonical Faber source code from AST. Pure serialization with no semantic transformation.

## Usage

```faber
§ ex "rivus/codegen" importa generate

fixum source = generate(corpus, "faber")
```

## Architecture

```
faber/
  index.fab      # Entry: generateFaber(corpus) -> textus
  nucleus.fab    # FaberGenerator state (indentation only)
  typus.fab      # Type annotation serialization
  sententia.fab  # All 35 statement types
  expressia.fab  # All 30 expression types
```

## Design

- **No semantic lowering** — No `inFlumina`, `inCursore` context flags
- **No type system translation** — Types pass through unchanged
- **No preamble generation** — No runtime imports
- **No HAL/norma rewriting** — Import paths preserved
- **Pure 1:1 serialization** — AST node → canonical text

## Formatting

- 2-space indentation
- Keywords emitted verbatim (`functio`, `si`, `redde`, etc.)
- Operators emitted verbatim (`+`, `-`, `==`, `et`, `aut`, etc.)
- Comments preserved (`#` for line, `/* */` for block)

## Round-trip

```faber
fixum ast1 = parse(source)
fixum emitted = generateFaber(ast1.corpus)
fixum ast2 = parse(emitted)
# ast1 ≡ ast2 (structurally equal)
```

## See Also

- `fons/rivus/codegen/glyph/` — Unicode glyph representation
- `fons/rivus/codegen/ts/` — TypeScript compilation target
