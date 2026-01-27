# Faber Glyph Codegen Target

Generates Unicode glyph representation from Faber AST. One concept, one glyph. Bidirectionally mappable with zero information loss.

## Design Principles

1. **One concept, one symbol** — No multi-character tokens. `===` becomes `≣`, not three characters.
2. **Visual stratification** — Distinct Unicode ranges create visual layers:
   - **Block characters** — Structure (delimiters)
   - **Math operators** — Operators (infix/prefix semantics)
   - **Geometric shapes** — Keywords (control flow, declarations)
   - **Braille** — Content (identifiers, literals)
   - **Punctuation glyphs** — Flow (terminators, separators)
3. **Trivial parsing** — Lexer is range checks. No lookahead, no ambiguity.
4. **Round-trip stability** — After canonicalization, `faber -> glyph -> faber -> ...` converges to a stable normal form.

## Usage

```faber
§ ex "rivus/codegen" importa generate

fixum glyphs = generate(corpus, "glyph")
```

## Architecture

```
glyph/
  index.fab      # Entry: generateGlyph(corpus) -> textus
  nucleus.fab    # GlyphGenerator state + comment formatting
  lexicon.fab    # Mapping tables (keywords, operators, delimiters, braille)
  typus.fab      # Type annotation serialization
  sententia.fab  # Statement handlers
  expressia.fab  # Expression handlers
```

## Identifier and Literal Mapping

All identifiers and literals map to **Braille** (U+2800–U+28FF).

Each byte maps directly: `byte → U+2800 + byte`

| Byte/ASCII | Code | Braille |
|------------|------|---------|
| `0`        | 48   | ⠰       |
| `9`        | 57   | ⠹       |
| `A`        | 65   | ⡁       |
| `Z`        | 90   | ⡚       |
| `a`        | 97   | ⡡       |
| `z`        | 122  | ⡺       |

## Delimiter Mapping

Delimiters map to **Block Characters** (U+2580–U+259F).

| Faber | Glyph | Unicode | Name |
|-------|-------|---------|------|
| `{`   | `▐`   | U+2590 | Right half block |
| `}`   | `▌`   | U+258C | Left half block |
| `(`   | `▝`   | U+259D | Upper right quadrant |
| `)`   | `▘`   | U+2598 | Upper left quadrant |
| `[`   | `▗`   | U+2597 | Lower right quadrant |
| `]`   | `▖`   | U+2596 | Lower left quadrant |
| `<`   | `▀`   | U+2580 | Upper half block (type params) |
| `>`   | `▄`   | U+2584 | Lower half block (type params) |
| `"`   | `▚`   | U+259A | Upper left + lower right |
| `'`   | `▞`   | U+259E | Upper right + lower left |

## Operator Mapping

Operators map to **Math Operators** (U+2200–U+22FF, U+2A00–U+2AFF).

### Arithmetic

| Faber | Glyph | Unicode |
|-------|-------|---------|
| `+`   | `⊕`   | U+2295 |
| `-`   | `⊖`   | U+2296 |
| `*`   | `⊛`   | U+229B |
| `/`   | `⊘`   | U+2298 |
| `%`   | `⊜`   | U+229C |

### Comparison

| Faber  | Glyph | Unicode |
|--------|-------|---------|
| `<`    | `≺`   | U+227A |
| `>`    | `≻`   | U+227B |
| `<=`   | `≼`   | U+227C |
| `>=`   | `≽`   | U+227D |
| `==`   | `≈`   | U+2248 |
| `===`  | `≣`   | U+2263 |
| `!=`   | `≠`   | U+2260 |

### Assignment

| Faber | Glyph | Unicode |
|-------|-------|---------|
| `=`   | `←`   | U+2190 |
| `+=`  | `↞`   | U+219E |
| `-=`  | `↢`   | U+21A2 |

## Keyword Mapping

Keywords map to **Geometric Shapes** and related symbol blocks.

### Declarations

| Faber     | Glyph | Unicode | Mnemonic |
|-----------|-------|---------|----------|
| `fixum`   | `≡`   | U+2261 | Identical/immutable |
| `varia`   | `≔`   | U+2254 | Definition |
| `functio` | `∫`   | U+222B | Integral (function shape) |
| `genus`   | `◎`   | U+25CE | Bullseye (class) |
| `pactum`  | `◌`   | U+25CC | Dotted circle (interface) |

### Control Flow

| Faber   | Glyph | Unicode | Mnemonic |
|---------|-------|---------|----------|
| `si`    | `↳`   | U+21B3 | Down-right arrow (if) |
| `secus` | `↲`   | U+21B2 | Down-left arrow (else) |
| `ergo`  | `∴`   | U+2234 | Therefore |
| `dum`   | `∞`   | U+221E | Infinity (while) |
| `ex`    | `∈`   | U+2208 | Element of (from) |
| `elige` | `⋔`   | U+22D4 | Pitchfork (switch) |
| `casu`  | `⌜`   | U+231C | Top-left corner (case) |

### Control Transfer

| Faber   | Glyph | Unicode | Mnemonic |
|---------|-------|---------|----------|
| `redde` | `⊢`   | U+22A2 | Turnstile (return) |
| `rumpe` | `⊗`   | U+2297 | Circled times (break) |
| `perge` | `↻`   | U+21BB | Clockwise arrow (continue) |

### Error Handling

| Faber    | Glyph | Unicode | Mnemonic |
|----------|-------|---------|----------|
| `tempta` | `◇`   | U+25C7 | Possibility (try) |
| `cape`   | `◆`   | U+25C6 | Filled diamond (catch) |
| `iace`   | `↯`   | U+21AF | Downwards zigzag (throw) |
| `mori`   | `⟂`   | U+27C2 | Perpendicular (panic) |

### Boolean and Logic

| Faber   | Glyph | Unicode | Mnemonic |
|---------|-------|---------|----------|
| `verum` | `⊤`   | U+22A4 | Top (true) |
| `falsum`| `⊥`   | U+22A5 | Bottom (false) |
| `nihil` | `∅`   | U+2205 | Empty set (null) |
| `et`    | `∧`   | U+2227 | Logical and |
| `aut`   | `∨`   | U+2228 | Logical or |

## Punctuation Mapping

| Faber | Glyph | Unicode |
|-------|-------|---------|
| `;`   | `⁏`   | U+204F |
| `,`   | `⸴`   | U+2E34 |
| `.`   | `·`   | U+00B7 |
| `:`   | `∶`   | U+2236 |
| `@`   | `※`   | U+203B |
| `#`   | `⌗`   | U+2317 |

## Example

### Faber Source

```faber
functio fibonacci(numerus n) fit numerus {
  si n <= 1 ergo redde n
  redde fibonacci(n - 1) + fibonacci(n - 2)
}
```

### Glyph Representation

```
∫ ⡦⡩⡢⡯⡮⡡⡣⡣⡩ ▝ ⡮⡵⡭⡥⡲⡵⡳ ⡮ ▘ → ⡮⡵⡭⡥⡲⡵⡳ ▐
  ↳ ⡮ ≼ ⠱ ∴ ⊢ ⡮
  ⊢ ⡦⡩⡢⡯⡮⡡⡣⡣⡩ ▝ ⡮ ⊖ ⠱ ▘ ⊕ ⡦⡩⡢⡯⡮⡡⡣⡣⡩ ▝ ⡮ ⊖ ⠲ ▘
▌
```

### Visual Decoding

```
∫              → functio (function declaration)
⡦⡩⡢⡯⡮⡡⡣⡣⡩    → "fibonacci" (braille identifier)
▝ ▘            → ( ) (parameter delimiters)
⡮⡵⡭⡥⡲⡵⡳      → "numerus" (type name)
⡮              → "n" (parameter name)
→              → fit (returns)
▐ ▌            → { } (block delimiters)
↳              → si (if)
≼              → <= (less or equal)
⠱              → "1" (number literal)
∴              → ergo (then)
⊢              → redde (return)
⊖              → - (subtraction)
⊕              → + (addition)
```

## Font Requirements

Requires a font with coverage for:
- Braille Patterns (U+2800–U+28FF)
- Block Elements (U+2580–U+259F)
- Mathematical Operators (U+2200–U+22FF)
- Supplemental Math Operators (U+2A00–U+2AFF)
- Arrows (U+2190–U+21FF)
- Geometric Shapes (U+25A0–U+25FF)

Recommended: JuliaMono, Iosevka, or Nerd Fonts variants.

## See Also

- `fons/glyph-go/README.md` - Canonical glyph specification
- `lexicon.fab` - Complete mapping tables
