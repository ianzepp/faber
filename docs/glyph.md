# Faber Unicode Glyph Reference

Faber promotes ASCII operators and Latin keywords to Unicode glyphs, eliminating lexer lookahead and producing visually compact, unambiguous source code. Both glyph and keyword/ASCII forms are accepted by the compiler — the formatter (`forma-rs`) can normalize to either style.

**Design principle:** These glyphs match what programming ligature fonts (Fira Code, JetBrains Mono, Cascadia Code) already render. Faber makes the source of truth match what developers already see.

---

## Core Operators (Implemented — #387)

Foundational operator promotions that eliminate all multi-character operator lookahead.

| Glyph | Unicode | Name | Replaces | Meaning |
|-------|---------|------|----------|---------|
| `←` | U+2190 | Leftwards arrow | `=` | Assignment |
| `≡` | U+2261 | Identical to | `==` | Equality |
| `≠` | U+2260 | Not equal to | `!=` | Inequality |
| `≤` | U+2264 | Less-than or equal | `<=` | Less-than-or-equal |
| `≥` | U+2265 | Greater-than or equal | `>=` | Greater-than-or-equal |
| `→` | U+2192 | Rightwards arrow | `->` | Return type arrow |

```faber
functio factorial(numerus n) → numerus {
    si n ≤ 1 {
        redde 1
    }
    redde n * factorial(n - 1)
}
```

---

## Range Operators (#406)

Replace the `..` exclusive range and `usque` inclusive range with single-codepoint glyphs. Visually intuitive: two dots = exclusive, three dots = inclusive.

| Glyph | Unicode | Name | Replaces | Meaning |
|-------|---------|------|----------|---------|
| `‥` | U+2025 | Two dot leader | `..` | Exclusive range (up to but not including) |
| `…` | U+2026 | Horizontal ellipsis | `usque` | Inclusive range (all the way through) |

```faber
itera pro 0‥5 fixum i { }    # 0, 1, 2, 3, 4
itera pro 0…5 fixum i { }    # 0, 1, 2, 3, 4, 5
```

**Keyword aliases:** `ante` and `usque` may be kept as verbose alternatives.

---

## Compound Assignment Operators (#404)

Circled operator glyphs read as "apply and store" — visually distinct from bare arithmetic operators.

| Glyph | Unicode | Name | Replaces | Meaning |
|-------|---------|------|----------|---------|
| `⊕` | U+2295 | Circled plus | `+=` | Add-assign |
| `⊖` | U+2296 | Circled minus | `-=` | Subtract-assign |
| `⊛` | U+229B | Circled asterisk | `*=` | Multiply-assign |
| `⊘` | U+2298 | Circled division slash | `/=` | Divide-assign |
| `⊜` | U+229C | Circled equals | `&=` | Bitwise AND-assign |
| `⊚` | U+229A | Circled ring | `\|=` | Bitwise OR-assign |

```faber
varia numerus x ← 10
x ⊕ 5      # x += 5
x ⊖ 10     # x -= 10
x ⊛ 2      # x *= 2
x ⊘ 3      # x /= 3
```

---

## Bitwise Operators (#405)

No collision with logical operators — Faber uses `et`/`aut` keywords for logical AND/OR.

| Glyph | Unicode | Name | Replaces | Meaning |
|-------|---------|------|----------|---------|
| `∧` | U+2227 | Logical and | `&` | Bitwise AND |
| `∨` | U+2228 | Logical or | `\|` | Bitwise OR |
| `⊻` | U+22BB | Xor | `^` | Bitwise XOR |
| `¬` | U+00AC | Not sign | `~` | Bitwise NOT |
| `≪` | U+226A | Much less-than | `sinistratum` | Left shift |
| `≫` | U+226B | Much greater-than | `dextratum` | Right shift |

```faber
fixum numerus a ← 0xFF
scribe a ∧ 0x0F     # bitwise AND
scribe a ∨ 0x01     # bitwise OR
scribe a ⊻ 0xAA     # bitwise XOR
scribe ¬a            # bitwise NOT
scribe a ≪ 4         # left shift
scribe a ≫ 4         # right shift
```

**Also removed:** `&`, `|`, `^`, `~`, `&&`, `||`, `sinistratum`, `dextratum` can be dropped from the lexer entirely.

---

## Branching — `si` / `sin` / `secus` (#407)

**Status: Speculative.** First keyword-to-glyph promotion — no parsing benefit, pure visual compactness.

| Glyph | Unicode | Name | Replaces | Mnemonic |
|-------|---------|------|----------|----------|
| `↳` | U+21B3 | Downwards arrow with tip rightwards | `si` (if) | Branch into |
| `↔` | U+2194 | Left-right arrow | `sin` (else-if) | Pivot to another condition |
| `↲` | U+21B2 | Downwards arrow with tip leftwards | `secus` (else) | Catch-all / fallback |

```faber
↳ x ≤ 5 {
    scribe "small"
} ↔ x ≤ 10 {
    scribe "medium"
} ↲ {
    scribe "big"
}
```

---

## Switching — `elige` / `discerne` / `casu` (#408)

**Status: Speculative.** Pairs with #407 — `↲` (secus/else) serves as the default/catch-all across all branching constructs.

| Glyph | Unicode | Name | Replaces | Mnemonic |
|-------|---------|------|----------|----------|
| `⋔` | U+22D4 | Pitchfork | `elige` (switch) | Branch into many |
| `⋈` | U+22C8 | Bowtie / natural join | `discerne` (match) | Match and destructure |
| `⌜` | U+231C | Top-left corner | `casu` (case) | Case branch marker |

```faber
⋔ status {
    ⌜ "pending" { scribe "waiting" }
    ⌜ "active" { scribe "running" }
    ↲ { scribe "unknown" }
}

⋈ event {
    ⌜ Click { scribe "clicked" }
    ⌜ Hover { scribe "hovered" }
}
```

**Unified branching family:**
```
↳  ↔  ↲       if / else-if / else
⋔  ⌜  ↲       switch / case / default
⋈  ⌜  ↲       match / case / default
```

---

## Loops — `dum` / `itera` / `rumpe` / `perge` (#409)

**Status: Speculative.** `↻` / `↺` are a natural pair — same shape, opposite direction.

| Glyph | Unicode | Name | Replaces | Mnemonic |
|-------|---------|------|----------|----------|
| `∞` | U+221E | Infinity | `dum` (while) | The loop itself |
| `↻` | U+21BB | Clockwise arrow | `itera` (for-each) | Go around the collection |
| `⊗` | U+2297 | Circled X | `rumpe` (break) | Stop / halt |
| `↺` | U+21BA | Counter-clockwise arrow | `perge` (continue) | Skip back to top |

```faber
∞ x > 0 {
    x ⊖ 1
}

↻ ex items fixum item {
    ↳ item ≤ 0 { ↺ }
    ↳ item ≡ 99 { ⊗ }
    scribe item
}
```

---

## Error Handling — `tempta` / `cape` / `iace` / `mori` (#410)

**Status: Speculative.** The `◇` / `◆` pairing is particularly elegant — try is the open diamond (uncertain), catch is the filled diamond (captured).

| Glyph | Unicode | Name | Replaces | Mnemonic |
|-------|---------|------|----------|----------|
| `◇` | U+25C7 | White diamond | `tempta` (try) | Uncertain outcome |
| `◆` | U+25C6 | Black diamond | `cape` (catch) | Outcome captured |
| `↯` | U+21AF | Downwards zigzag arrow | `iace` (throw) | Disruption / lightning |
| `⟂` | U+27C2 | Perpendicular | `mori` (panic) | Dead stop |

```faber
◇ {
    scribe "attempting"
    ↯ "something broke"
}
◆ err {
    scribe "caught:", err
}

⟂ "unrecoverable"
```

---

## Type Operations — `qua` / `innatum` / `novum` (#411)

**Status: Speculative.** All three are type transformation operations — take data and apply a type.

| Glyph | Unicode | Name | Replaces | Mnemonic |
|-------|---------|------|----------|----------|
| `⇢` | U+21E2 | Rightwards dashed arrow | `qua` (type cast) | "Becomes" / transforms into |
| `⊡` | U+22A1 | Squared dot | `innatum` (native construct) | Concrete / materialized |
| `⊙` | U+2299 | Circled dot | `novum` (new instance) | Bring into existence / seed |

```faber
fixum x ← value ⇢ textus        # cast
fixum items ← [] ⊡ lista<textus> # native construct
fixum rect ← ⊙ Rectangle {      # new instance
    width: 10,
    height: 5
}
```

---

## Complete Glyph Table

All 38 glyphs in one place, sorted by category:

### Operators (concrete)
| Glyph | Unicode | Replaces | Category |
|-------|---------|----------|----------|
| `←` | U+2190 | `=` | Assignment |
| `≡` | U+2261 | `==` | Comparison |
| `≠` | U+2260 | `!=` | Comparison |
| `≤` | U+2264 | `<=` | Comparison |
| `≥` | U+2265 | `>=` | Comparison |
| `→` | U+2192 | `->` | Arrow |
| `‥` | U+2025 | `..` | Range |
| `…` | U+2026 | `usque` | Range |
| `⊕` | U+2295 | `+=` | Compound assignment |
| `⊖` | U+2296 | `-=` | Compound assignment |
| `⊛` | U+229B | `*=` | Compound assignment |
| `⊘` | U+2298 | `/=` | Compound assignment |
| `⊜` | U+229C | `&=` | Compound assignment |
| `⊚` | U+229A | `\|=` | Compound assignment |
| `∧` | U+2227 | `&` | Bitwise |
| `∨` | U+2228 | `\|` | Bitwise |
| `⊻` | U+22BB | `^` | Bitwise |
| `¬` | U+00AC | `~` | Bitwise |
| `≪` | U+226A | `sinistratum` | Bitwise shift |
| `≫` | U+226B | `dextratum` | Bitwise shift |

### Keywords (speculative)
| Glyph | Unicode | Replaces | Category |
|-------|---------|----------|----------|
| `↳` | U+21B3 | `si` | Branching |
| `↔` | U+2194 | `sin` | Branching |
| `↲` | U+21B2 | `secus` | Branching |
| `⋔` | U+22D4 | `elige` | Switching |
| `⋈` | U+22C8 | `discerne` | Switching |
| `⌜` | U+231C | `casu` | Switching |
| `∞` | U+221E | `dum` | Loops |
| `↻` | U+21BB | `itera` | Loops |
| `⊗` | U+2297 | `rumpe` | Loops |
| `↺` | U+21BA | `perge` | Loops |
| `◇` | U+25C7 | `tempta` | Error handling |
| `◆` | U+25C6 | `cape` | Error handling |
| `↯` | U+21AF | `iace` | Error handling |
| `⟂` | U+27C2 | `mori` | Error handling |
| `⇢` | U+21E2 | `qua` | Type operations |
| `⊡` | U+22A1 | `innatum` | Type operations |
| `⊙` | U+2299 | `novum` | Type operations |

---

## Implementation

Both glyph and Latin/ASCII forms are accepted permanently. The lexer maps both to the same `TokenKind`:

```rust
'←' => TokenKind::Assign,
"si" | "↳" => TokenKind::Si,
```

The parser matches on `TokenKind`, not spelling — no parser changes needed for dual-form support. The formatter (`forma-rs`, #292) can normalize to glyph or keyword form based on preference.

## Tracking

Epic: #412 — Unicode glyph promotion for operators and keywords
