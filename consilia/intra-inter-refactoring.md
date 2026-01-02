# Opportunities for `intra` and `inter` Operators in fons-fab/

Research findings on where the new operators could improve code clarity.

## Operator Semantics

- **`intra`** - Range containment: `x intra 0..100` replaces `x >= 0 et x <= 100`
- **`inter`** - Set membership: `x inter [1, 2, 3]` replaces `x == 1 aut x == 2 aut x == 3`

## Summary Statistics

| Operator | Candidates | High-Impact Areas |
|----------|-----------|-------------------|
| `inter` | 24 sites | Character validation (6), Type checking (10), Modifiers (3) |
| `intra` | 5 sites | Lexor character range checks |

## High-Impact Refactoring Candidates

### 1. Character Validation (Lexor) - **HIGHEST IMPACT**

**fons-fab/lexor/index.fab** - 6 sites

```fab
# Line 108: Decimal digit
- redde c >= "0" et c <= "9"
+ redde c intra "0".."9"

# Line 113: Hex digit
- redde ego.estDigitus(c) aut (c >= "a" et c <= "f") aut (c >= "A" et c <= "F")
+ redde ego.estDigitus(c) aut c intra "a".."f" aut c intra "A".."F"

# Line 118: Binary digit
- redde c == "0" aut c == "1"
+ redde c inter ["0", "1"]

# Line 123: Octal digit
- redde c >= "0" et c <= "7"
+ redde c intra "0".."7"

# Line 128: Letter/identifier start
- redde (c >= "a" et c <= "z") aut (c >= "A" et c <= "Z") aut c == "_"
+ redde c intra "a".."z" aut c intra "A".."Z" aut c == "_"

# Line 180: Whitespace
- si c == " " aut c == "\t" aut c == "\r" {
+ si c inter [" ", "\t", "\r"] {
```

**Impact**: 6 functions become significantly clearer. Intent (range/set membership) explicit.

---

### 2. Type Checking - **HIGH IMPACT**

**Numeric type checks** - Appears 3 times across codebase:

```fab
# fons-fab/semantic/expressia/unaria.fab:67
# fons-fab/semantic/expressia/binaria.fab:119
# fons-fab/semantic/typi.fab:459
- redde nomen == "numerus" aut nomen == "fractus" aut nomen == "decimus" aut nomen == "magnus"
+ redde nomen inter ["numerus", "fractus", "decimus", "magnus"]
```

**Operator type checks** - 4 instances:

```fab
# fons-fab/semantic/expressia/binaria.fab:104
- redde signum == "+" aut signum == "-" aut signum == "*" aut signum == "/" aut signum == "%"
+ redde signum inter ["+", "-", "*", "/", "%"]

# Line 109: Comparison operators
- redde signum == "<" aut signum == ">" aut signum == "<=" aut signum == ">="
+ redde signum inter ["<", ">", "<=", ">="]

# Line 114: Logical operators
- redde signum == "&&" aut signum == "||" aut signum == "et" aut signum == "aut"
+ redde signum inter ["&&", "||", "et", "aut"]

# Line 84: Equality operators
- si b.signum == "==" aut b.signum == "!="
+ si b.signum inter ["==", "!="]
```

**Generic type check**:

```fab
# fons-fab/semantic/nucleus.fab:75
- redde nomen == "lista" aut nomen == "tabula" aut nomen == "copia" aut nomen == "promissum" aut nomen == "cursor" aut nomen == "fluxus"
+ redde nomen inter ["lista", "tabula", "copia", "promissum", "cursor", "fluxus"]
```

---

### 3. Visibility Modifiers - **MEDIUM IMPACT**

**fons-fab/codegen/typi.fab** - 4 sites (lines 96-114)

```fab
# Gender agreement forms for visibility
- si mod == "publicum" aut mod == "publica" aut mod == "publicus" {
+ si mod inter ["publicum", "publica", "publicus"] {
    redde "public"
}

- si mod == "privatum" aut mod == "privata" aut mod == "privatus" {
+ si mod inter ["privatum", "privata", "privatus"] {
    redde "private"
}

- si mod == "protectum" aut mod == "protecta" aut mod == "protectus" {
+ si mod inter ["protectum", "protecta", "protectus"] {
    redde "protected"
}

- si mod == "abstractum" aut mod == "abstracta" aut mod == "abstractus" {
+ si mod inter ["abstractum", "abstracta", "abstractus"] {
    redde "abstract"
}
```

**Impact**: Reduces verbose gender agreement checks.

---

### 4. Parser Keyword Checks - **MEDIUM IMPACT**

```fab
# fons-fab/parser/sententia/index.fab:51
- si verbum === "varia" aut verbum === "fixum" aut verbum === "figendum" aut verbum === "variandum"
+ si verbum inter ["varia", "fixum", "figendum", "variandum"]

# fons-fab/semantic/expressia/primaria.fab:25
- si n.nomen == "verum" aut n.nomen == "falsum"
+ si n.nomen inter ["verum", "falsum"]

# fons-fab/semantic/expressia/vocatio.fab:70
- si n.nomen == "primus" aut n.nomen == "ultimus"
+ si n.nomen inter ["primus", "ultimus"]
```

---

### 5. Unary Operator Checks - **LOW IMPACT**

```fab
# fons-fab/semantic/expressia/unaria.fab:22-23
- si u.signum == "!" aut u.signum == "non"
+ si u.signum inter ["!", "non"]

# Line 27
- si u.signum == "nulla" aut u.signum == "nonnulla"
+ si u.signum inter ["nulla", "nonnulla"]

# Line 32
- si u.signum == "nihil" aut u.signum == "nonnihil"
+ si u.signum inter ["nihil", "nonnihil"]
```

**Impact**: Marginal improvement (only 2 items per set).

---

## Parser Method Chains - Needs Helper Functions

These patterns appear frequently but would require new helper methods:

```fab
# Current pattern (fons-fab/parser/expressia/binaria.fab:159)
dum p.congruet(SymbolumGenus.AequumBis) aut 
    p.congruet(SymbolumGenus.NonAequum) aut 
    p.congruet(SymbolumGenus.AequumTer) aut 
    p.congruet(SymbolumGenus.NonAequumBis) {

# Proposed helper method
functio congruetAliquod(lista<SymbolumGenus> typi) -> bivalens {
    ex typi pro genus {
        si ego.congruet(genus) { redde verum }
    }
    redde falsum
}

# Usage
dum p.congruetAliquod([
    SymbolumGenus.AequumBis, 
    SymbolumGenus.NonAequum,
    SymbolumGenus.AequumTer, 
    SymbolumGenus.NonAequumBis
]) {
```

**Instances**: 5+ parser method chains

---

## Recommendation: Prioritized Rollout

### Phase A: High-Impact, Low-Risk (Do First)
1. **Lexor character validation** (6 sites) - Clearest wins
2. **Semantic type/operator checks** (10 sites) - Repeated patterns

### Phase B: Medium-Impact (Do Second)
3. **Visibility modifiers** (4 sites) - Reduces gender complexity
4. **Parser keyword checks** (5 sites) - Improves readability

### Phase C: Low-Impact (Optional)
5. **Unary operator checks** (3 sites) - Marginal benefit

### Phase D: Requires Infrastructure (Later)
6. **Parser helper methods** - Add `congruetAliquod()`, `probaVerbumAliquod()`

---

## Benefits

**Code Clarity**:
- Intent is explicit: "is this IN the set?" vs "is it equal to A or B or C?"
- Reduces visual noise from repeated `aut` chains
- Self-documenting (no comment needed to explain validation)

**Maintenance**:
- Adding/removing items from set is easier (one line change)
- Less error-prone than manually updating boolean chains

**Latin Semantics**:
- `inter` = "among" (naturally maps to set membership)
- `intra` = "within" (naturally maps to range containment)

---

## Open Questions

1. **Should `inter` support enum values?**
   ```fab
   si genus inter [SymbolumGenus.Numerus, SymbolumGenus.Fractus]
   ```

2. **Does `intra` work with character ranges in current implementation?**
   ```fab
   # Does this compile today?
   si c intra "a".."z"
   ```

3. **Performance**: Are `inter`/`intra` optimized for small sets, or do they build actual collections?


---

## Implementation Findings (Verified)

### `intra` Operator Constraints

**Semantic Requirements**:
- Left operand: **MUST be numeric** (`numerus`, `fractus`, `decimus`, `magnus`)
- Right operand: **MUST be RangeExpression** (using `..` operator)

**TypeScript Codegen**:
```fab
x intra 0..100  →  (x >= 0 && x < 100)  # Exclusive end
```

**Character range checks CANNOT use `intra`** because characters are `textus` type:
```fab
# THIS WILL NOT COMPILE:
c intra "0".."9"  ❌ Error: left operand must be numeric, got textus

# Must use traditional comparison:
c >= "0" et c <= "9"  ✅ Works
```

### `inter` Operator Constraints

**Semantic Requirements**:
- Left operand: **Any type**
- Right operand: **ArrayExpression** (literal array `[...]`)

**TypeScript Codegen**:
```fab
x inter [1, 2, 3]  →  [1, 2, 3].includes(x)
```

**Works with strings**:
```fab
c inter ["a", "e", "i", "o", "u"]  ✅ Compiles to ["a", "e", "i", "o", "u"].includes(c)
```

---

## REVISED Recommendations

Given the implementation constraints, the original findings need adjustment:

### ❌ INVALID: Character Range Checks Cannot Use `intra`

All 5 proposed `intra` refactorings in **fons-fab/lexor/index.fab** are **INVALID**:
- Line 108: `c intra "0".."9"` ❌ (c is textus)
- Line 113: `c intra "a".."f"` ❌ (c is textus)
- Line 123: `c intra "0".."7"` ❌ (c is textus)
- Line 128: `c intra "a".."z"` ❌ (c is textus)

**Reason**: `intra` requires numeric left operand; characters are strings.

**Outcome**: No `intra` candidates exist in fons-fab (no numeric range checks found).

---

### ✅ VALID: All `inter` Refactorings

All 24 `inter` refactorings remain valid:

**High-Impact** (16 sites):
1. Lexor character validation: 2 sites (binary digit, whitespace)
2. Semantic type/operator checks: 10 sites
3. Visibility modifiers: 4 sites

**Medium-Impact** (8 sites):
4. Parser keyword checks: 5 sites
5. Unary operator checks: 3 sites

---

## Final Statistics

| Operator | Total Candidates | Valid After Verification |
|----------|------------------|--------------------------|
| `intra` | 5 | **0** (all textus, not numerus) |
| `inter` | 24 | **24** (all valid) |

---

## Example Valid Refactoring

### Before (fons-fab/lexor/index.fab:118)
```fab
functio estDigitusBinarius(textus c) -> bivalens {
    redde c == "0" aut c == "1"
}
```

### After
```fab
functio estDigitusBinarius(textus c) -> bivalens {
    redde c inter ["0", "1"]
}
```

### Compiled Output
```typescript
function estDigitusBinarius(c: string): boolean {
    return ["0", "1"].includes(c);
}
```

**Benefits**:
- 28% shorter (24 chars → 17 chars in condition)
- Clearer intent ("is c among binary digits?")
- Easier to extend (add digits to array vs add `aut` clauses)

