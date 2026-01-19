# Faber Glyph Format

An alternative representation of Faber source code using Unicode symbols. One concept, one glyph. Bidirectionally mappable with zero information loss.

## Design Principles

1. **One concept, one symbol** — No multi-character tokens. `===` becomes `≣`, not three characters.
2. **Visual stratification** — Four distinct Unicode ranges create visual layers:
   - **Block characters** — Structure (delimiters)
   - **Math operators** — Semantics (keywords)
   - **Braille** — Content (identifiers, literals)
   - **Punctuation glyphs** — Flow (terminators, separators)
3. **Trivial parsing** — Lexer is range checks. No lookahead, no ambiguity.
4. **Round-trip fidelity** — Faber → Glyph → Faber preserves everything.

## Formatting Rules

- Blocks always have spaces around them: `▝ ⡮ ▘` not `▝⡮▘`
- Braille tokens have no internal spaces: `⡦⡩⡢⡯⡮⡡⡣⡣⡩` not `⡦ ⡩ ⡢...`
- Math operators and punctuation follow normal token spacing

---

## Identifier and Literal Mapping

All identifiers and literals map to **Braille** (U+2800–U+28FF).

Each ASCII byte maps directly: `char_code → U+2800 + char_code`

| ASCII | Code | Braille |
|-------|------|---------|
| `0`   | 48   | ⠰       |
| `9`   | 57   | ⠹       |
| `A`   | 65   | ⡁       |
| `Z`   | 90   | ⡚       |
| `a`   | 97   | ⡡       |
| `z`   | 122  | ⡺       |

String contents remain as braille sequences. The quote delimiters use block characters (see below).

---

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
| `<`   | `▀`   | U+2580 | Upper half block (in type params) |
| `>`   | `▄`   | U+2584 | Lower half block (in type params) |
| `"`   | `▚`   | U+259A | Upper left + lower right |
| `'`   | `▞`   | U+259E | Upper right + lower left |

---

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
| `++`  | `⧺`   | U+29FA |
| `--`  | `⧻`   | U+29FB |

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
| `!==`  | `≢`   | U+2262 |

### Assignment

| Faber | Glyph | Unicode | Name |
|-------|-------|---------|------|
| `=`   | `←`   | U+2190 | Leftwards arrow |
| `+=`  | `↞`   | U+219E | Leftwards two headed arrow |
| `-=`  | `↢`   | U+21A2 | Leftwards arrow with tail |
| `*=`  | `↩`   | U+21A9 | Leftwards arrow with hook |
| `/=`  | `↫`   | U+21AB | Leftwards arrow with loop |
| `&=`  | `↤`   | U+21A4 | Leftwards arrow from bar |
| `\|=` | `↜`   | U+219C | Leftwards wave arrow |

### Bitwise

| Faber | Glyph | Unicode |
|-------|-------|---------|
| `&`   | `⊓`   | U+2293 |
| `\|`  | `⊔`   | U+2294 |
| `^`   | `⊻`   | U+22BB |
| `~`   | `∼`   | U+223C |

### Logical (symbol form)

| Faber  | Glyph | Unicode |
|--------|-------|---------|
| `&&`   | `⋀`   | U+22C0 |
| `\|\|` | `⋁`   | U+22C1 |
| `!`    | `¬`   | U+00AC |

### Other

| Faber | Glyph | Unicode | Note |
|-------|-------|---------|------|
| `..`  | `‥`   | U+2025 | Two dot leader (range) |
| `->`  | `→`   | U+2192 | Rightwards arrow |
| `=>`  | `⇒`   | U+21D2 | Rightwards double arrow |

Note:
- Optional chaining (`?.`, `?[`, `?(`) is two tokens: `⸮` (optional marker) + delimiter
- Non-null assertion (`!.`, `![`, `!(`) is two tokens: `¡` (non-null marker) + delimiter
- Nullish coalescing (`??`) is the keyword `vel` → `⁇`
- Logical not (`!expr`) is `¬` (distinct from non-null `¡`)

---

## Punctuation Mapping

| Faber | Glyph | Unicode |
|-------|-------|---------|
| `;`   | `⁏`   | U+204F |
| `,`   | `⸴`   | U+2E34 |
| `.`   | `·`   | U+00B7 |
| `:`   | `∶`   | U+2236 |
| `?`   | `⸮`   | U+2E2E | Reversed question mark (optional marker) |
| `!`   | `¡`   | U+00A1 | Inverted exclamation (non-null marker) |
| `@`   | `※`   | U+203B | Reference mark (annotations) |
| `#`   | `⌗`   | U+2317 | Viewdata square (comments) |
| `§`   | `§`   | unchanged (import sigil) |

---

## Keyword Mapping

Keywords map to **Math Operators** (U+2200–U+22FF).

### Declarations

| Faber       | Glyph | Unicode | Mnemonic |
|-------------|-------|---------|----------|
| `fixum`     | `≡`   | U+2261 | Identical/immutable |
| `varia`     | `≔`   | U+2254 | Definition |
| `figendum`  | `⫢`   | U+2AE2 | Vertical bar + triple turnstile (const + await) |
| `variandum` | `⫤`   | U+2AE4 | Vertical bar + double left turnstile (let + await) |
| `functio`   | `∫`   | U+222B | Integral (function shape) |
| `typus`     | `⊷`   | U+22B7 | Image of (type alias) |
| `ordo`      | `⊞`   | U+229E | Boxed plus (enum) |
| `abstractus`| `⊟`   | U+229F | Boxed minus (abstract) |

### Type/Class Family

| Faber       | Glyph | Unicode | Mnemonic |
|-------------|-------|---------|----------|
| `pactum`    | `◌`   | U+25CC | Dotted circle (interface — abstract) |
| `genus`     | `◎`   | U+25CE | Bullseye (class — concrete structure) |
| `ego`       | `◉`   | U+25C9 | Fisheye (self — solid core) |
| `novum`     | `⦿`   | U+29BF | Circled bullet (new — create instance) |
| `qua`       | `⦶`   | U+29B6 | Circled vertical bar (cast/transform) |
| `innatum`   | `⦵`   | U+29B5 | Circle with horizontal bar (native type) |

### Tagged Union Family

| Faber       | Glyph | Unicode | Mnemonic |
|-------------|-------|---------|----------|
| `discretio` | `⦻`   | U+29BB | Circle with X (define variants) |
| `finge`     | `⦺`   | U+29BA | Circle divided (construct variant) |
| `discerne`  | `⦼`   | U+29BC | Circled rotated division (pattern match) |

### Class Members

| Faber    | Glyph | Unicode | Mnemonic |
|----------|-------|---------|----------|
| `sub`    | `⊏`   | U+228F | Square image of (subtype) |
| `implet` | `⊒`   | U+2292 | Square original of (implements) |
| `generis`| `⊺`   | U+22BA | Intercalate (static/class-level) |
| `nexum`  | `⊸`   | U+22B8 | Multimap (bound/property) |

### Control Flow

| Faber     | Glyph | Unicode | Mnemonic |
|-----------|-------|---------|----------|
| `si`      | `∃`   | U+2203 | Exists (if) |
| `sin`     | `∄`   | U+2204 | Not exists (else-if) |
| `secus`   | `∁`   | U+2201 | Complement (else/otherwise) |
| `ergo`    | `∴`   | U+2234 | Therefore |
| `dum`     | `∞`   | U+221E | Infinity (while) |
| `ex`      | `∈`   | U+2208 | Element of (from) |
| `de`      | `∋`   | U+220B | Contains (in/of) |
| `pro`     | `∀`   | U+2200 | For all |
| `elige`   | `⋔`   | U+22D4 | Pitchfork (switch) |
| `casu`    | `↳`   | U+21B3 | Down-right arrow (case) |
| `ceterum` | `↲`   | U+21B2 | Down-left arrow (default) |
| `custodi` | `⊧`   | U+22A7 | Guard |
| `fac`     | `⊡`   | U+22A1 | Boxed empty (do block) |

### Control Transfer

| Faber   | Glyph | Unicode | Mnemonic |
|---------|-------|---------|----------|
| `redde` | `⊢`   | U+22A2 | Turnstile (return) |
| `reddit`| `⊣`   | U+22A3 | Reverse turnstile (inline return) |
| `rumpe` | `⊗`   | U+2297 | Circled times (break/cancel) |
| `perge` | `↻`   | U+21BB | Clockwise arrow (continue) |

### Error Handling

| Faber    | Glyph | Unicode | Mnemonic |
|----------|-------|---------|----------|
| `tempta` | `◇`   | U+25C7 | Possibility (try) |
| `cape`   | `◆`   | U+25C6 | Filled diamond (catch) |
| `demum`  | `◈`   | U+25C8 | Diamond in diamond (finally) |
| `iace`   | `↯`   | U+21AF | Downwards zigzag (throw) |
| `iacit`  | `⤋`   | U+290B | Downwards triple arrow (inline throw) |
| `mori`   | `⟂`   | U+27C2 | Perpendicular (panic/halt) |
| `moritor`| `⫫`   | U+2AEB | Double up tack (inline panic) |
| `adfirma`| `⊩`   | U+22A9 | Forces (assert) |

### Async

| Faber    | Glyph | Unicode | Mnemonic |
|----------|-------|---------|----------|
| `cede`   | `⋆`   | U+22C6 | Star (await) |
| `futura` | `⊶`   | U+22B6 | Original of (async marker) |
| `fit`    | `→`   | U+2192 | Returns sync |
| `fiet`   | `⇢`   | U+21E2 | Returns async |
| `fiunt`  | `⇉`   | U+21C9 | Yields (generator) |
| `fient`  | `⇶`   | U+21F6 | Yields async |

### Boolean and Logic

| Faber   | Glyph | Unicode | Mnemonic |
|---------|-------|---------|----------|
| `verum` | `⊤`   | U+22A4 | Top (true) |
| `falsum`| `⊥`   | U+22A5 | Bottom (false) |
| `nihil` | `∅`   | U+2205 | Empty set (null) |
| `et`    | `∧`   | U+2227 | Logical and |
| `aut`   | `∨`   | U+2228 | Logical or |
| `non`   | `¬`   | U+00AC | Negation |
| `vel`   | `⁇`   | U+2047 | Double question mark (nullish ??) |
| `est`   | `≟`   | U+225F | Questioned equal (is/type check) |

### Type Conversions

| Faber       | Glyph | Unicode | Mnemonic |
|-------------|-------|---------|----------|
| `numeratum` | `⌊`   | U+230A | Left floor (parse to integer) |
| `fractatum` | `⌈`   | U+2308 | Left ceiling (parse to float) |
| `textatum`  | `≋`   | U+224B | Triple tilde (to string) |
| `bivalentum`| `⊼`   | U+22BC | NAND (to boolean) |

### Parameters

| Faber   | Glyph | Unicode | Mnemonic |
|---------|-------|---------|----------|
| `de`    | `∋`   | U+220B | Contains (read param) |
| `in`    | `⊳`   | U+22B3 | Contains as normal subgroup (mutate param) |
| `ex`    | `∈`   | U+2208 | Element of (consume param) |
| `si`    | `⸮`   | U+2E2E | Reversed question mark (optional param) |
| `ceteri`| `⋯`   | U+22EF | Midline horizontal ellipsis (rest) |
| `sparge`| `⋰`   | U+22F0 | Up right diagonal ellipsis (spread) |
| `ut`    | `↦`   | U+21A6 | Rightwards arrow from bar (alias) |

### Imports

| Faber    | Glyph | Unicode | Mnemonic |
|----------|-------|---------|----------|
| `importa`| `⊲`   | U+22B2 | Import |
| `ut`     | `↦`   | U+21A6 | As/maps to |

### Output

| Faber   | Glyph | Unicode | Mnemonic |
|---------|-------|---------|----------|
| `scribe`| `⊝`   | U+229D | Circled dash (log) |
| `vide`  | `⋱`   | U+22F1 | Diagonal ellipsis (debug) |
| `mone`  | `⋮`   | U+22EE | Vertical ellipsis (warn) |

### Ranges

| Faber  | Glyph | Unicode | Mnemonic |
|--------|-------|---------|----------|
| `ante` | `≺`   | U+227A | Before |
| `usque`| `≼`   | U+227C | Up to |
| `per`  | `⊹`   | U+22B9 | Step by |
| `intra`| `∈`   | U+2208 | Within |
| `inter`| `≬`   | U+226C | Between |

### Bitwise Keywords

| Faber        | Glyph | Unicode | Mnemonic |
|--------------|-------|---------|----------|
| `sinistratum`| `⋘`   | U+22D8 | Left shift |
| `dextratum`  | `⋙`   | U+22D9 | Right shift |

### Testing

| Faber       | Glyph | Unicode | Mnemonic |
|-------------|-------|---------|----------|
| `probandum` | `⊬`   | U+22AC | Does not prove (test suite) |
| `proba`     | `⫞`   | U+2ADE | Short left tack (test case) |
| `praepara`  | `⊰`   | U+22B0 | Precedes under relation (before) |
| `postpara`  | `⊱`   | U+22B1 | Succeeds under relation (after) |
| `omitte`    | `⦸`   | U+29B8 | Circled reverse solidus (skip/banned) |

### Entry Points

| Faber     | Glyph | Unicode | Mnemonic |
|-----------|-------|---------|----------|
| `incipit` | `⟙`   | U+27D9 | Large down tack (begin sync — tall, complete) |
| `incipiet`| `⫟`   | U+2ADF | Short up tack (begin async — incomplete) |

### Resource Management

| Faber  | Glyph | Unicode | Mnemonic |
|--------|-------|---------|----------|
| `cura` | `⦾`   | U+29BE | Circled white bullet (managed scope) |
| `arena`| `—`   | — | UNASSIGNED (conflict with ordo) |
| `page` | `—`   | — | UNASSIGNED (conflict with abstractus) |

---

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
  ∃ ⡮ ≼ ⠱ ∴ ⊢ ⡮
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
∃              → si (if)
≼              → <= (less or equal)
⠱              → "1" (number literal)
∴              → ergo (then)
⊢              → redde (return)
⊖              → - (subtraction)
⊕              → + (addition)
```

---

## Implementation Notes

### Lexer Strategy

```
if (0x2800 <= cp <= 0x28FF) → identifier/literal token
if (0x2580 <= cp <= 0x259F) → delimiter token
if (0x2200 <= cp <= 0x22FF) → keyword or operator token
else → punctuation or whitespace
```

### Codegen

Glyph output is a post-parse serialization. The AST remains unchanged; only the printer differs.

### Font Requirements

Requires a font with coverage for:
- Braille Patterns (U+2800–U+28FF)
- Block Elements (U+2580–U+259F)
- Mathematical Operators (U+2200–U+22FF)
- Supplemental Math Operators (U+2A00–U+2AFF)
- Arrows (U+2190–U+21FF)
- Supplemental Arrows-A (U+27F0–U+27FF)
- Supplemental Arrows-B (U+2900–U+297F)
- Geometric Shapes (U+25A0–U+25FF) for try/catch diamonds

Recommended: JuliaMono, Iosevka, or Nerd Fonts variants.
