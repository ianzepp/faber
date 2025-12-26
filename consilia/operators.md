# Operators Reference

Complete reference of all operators in Faber Romanus.

## Precedence Table (Lowest to Highest)

| Level | Operators                                                           | Associativity | Description         |
| ----- | ------------------------------------------------------------------- | ------------- | ------------------- |
| 1     | `=`                                                                 | Right         | Assignment          |
| 2     | `? :`, `sic secus`                                                  | Right         | Ternary conditional |
| 3     | `vel`, `??`                                                         | Left          | Nullish coalescing  |
| 4     | `\|\|`, `aut`                                                       | Left          | Logical OR          |
| 5     | `&&`, `et`                                                          | Left          | Logical AND         |
| 6     | `==`, `!=`, `===`, `!==`, `est`, `non est`                          | Left          | Equality            |
| 7     | `<`, `>`, `<=`, `>=`                                                | Left          | Comparison          |
| 8     | `\|`                                                                | Left          | Bitwise OR          |
| 9     | `^`                                                                 | Left          | Bitwise XOR         |
| 10    | `&`                                                                 | Left          | Bitwise AND         |
| 11    | `<<`, `>>`                                                          | Left          | Bit shift           |
| 12    | `..`, `ante`, `usque`                                               | N/A           | Range               |
| 13    | `+`, `-`                                                            | Left          | Additive            |
| 14    | `*`, `/`, `%`                                                       | Left          | Multiplicative      |
| 15    | `!`, `-`, `~`, `non`, `nulla`, `nonnulla`, `negativum`, `positivum` | Right         | Unary prefix        |
| 16    | `ut`                                                                | Left          | Type cast           |
| 17    | `.`, `?.`, `!.`, `[]`, `()`                                         | Left          | Member access, call |

**Note:** Bitwise operators bind tighter than comparison (unlike C). This means `flags & MASK == 0` parses as `(flags & MASK) == 0`.

---

## Arithmetic Operators

| Operator    | Description              | Example | Result |
| ----------- | ------------------------ | ------- | ------ |
| `+`         | Addition / concatenation | `3 + 2` | `5`    |
| `-`         | Subtraction              | `3 - 2` | `1`    |
| `*`         | Multiplication           | `3 * 2` | `6`    |
| `/`         | Division                 | `6 / 2` | `3`    |
| `%`         | Modulo                   | `7 % 3` | `1`    |
| `-` (unary) | Negation                 | `-5`    | `-5`   |

**Not supported:** `++`, `--`, `+=`, `-=`, `*=`, `/=`, `**` (exponentiation)

---

## Comparison Operators

| Operator | Latin     | Description           | Example                  |
| -------- | --------- | --------------------- | ------------------------ |
| `==`     |           | Equality              | `x == y`                 |
| `!=`     |           | Inequality            | `x != y`                 |
| `===`    | `est`     | Strict equality       | `x === y`, `x est y`     |
| `!==`    | `non est` | Strict inequality     | `x !== y`, `x non est y` |
| `<`      |           | Less than             | `x < y`                  |
| `>`      |           | Greater than          | `x > y`                  |
| `<=`     |           | Less than or equal    | `x <= y`                 |
| `>=`     |           | Greater than or equal | `x >= y`                 |

---

## Logical Operators

| Operator | Latin | Description | Example               |
| -------- | ----- | ----------- | --------------------- |
| `&&`     | `et`  | Logical AND | `a && b`, `a et b`    |
| `\|\|`   | `aut` | Logical OR  | `a \|\| b`, `a aut b` |
| `!`      | `non` | Logical NOT | `!x`, `non x`         |

**Note:** Prefix `!` is deprecated in favor of `non` to avoid confusion with `!.` (non-null assertion).

---

## Bitwise Operators

| Operator | Description | Example        | Result |
| -------- | ----------- | -------------- | ------ |
| `&`      | Bitwise AND | `0x0F & 0x03`  | `0x03` |
| `\|`     | Bitwise OR  | `0x0F \| 0x30` | `0x3F` |
| `^`      | Bitwise XOR | `0x0F ^ 0x03`  | `0x0C` |
| `~`      | Bitwise NOT | `~0x0F`        | `-16`  |
| `<<`     | Left shift  | `1 << 4`       | `16`   |
| `>>`     | Right shift | `16 >> 2`      | `4`    |

**Design decision:** No Latin keywords for bitwise operators. These are low-level operations that don't map naturally to spoken language.

**Not supported:** `>>>` (unsigned right shift), `&=`, `|=`, `^=`, `<<=`, `>>=`

---

## Nullish and Coalescing

| Operator | Latin | Description        | Example             |
| -------- | ----- | ------------------ | ------------------- |
| `??`     | `vel` | Nullish coalescing | `x ?? y`, `x vel y` |

**Nullish vs Falsy:** `vel` only triggers on `nihil`, not on `0`, `""`, or `falsum`.

```faber
0 vel 5           // 0 (not nihil)
"" vel "default"  // "" (not nihil)
nihil vel 5       // 5
```

---

## Member Access

| Operator | Description        | Example                  |
| -------- | ------------------ | ------------------------ |
| `.`      | Property access    | `user.name`              |
| `?.`     | Optional chaining  | `user?.address?.city`    |
| `!.`     | Non-null assertion | `user!.name`             |
| `[]`     | Computed access    | `items[0]`, `map["key"]` |

---

## Type Operators

| Operator | Latin | Description         | Example        |
| -------- | ----- | ------------------- | -------------- |
| `ut`     |       | Type cast           | `x ut textus`  |
| `est`    |       | Type check (future) | `x est textus` |

**Note:** `est` for type checking is not yet implemented. Currently `est` is an alias for `===`.

---

## Range Operators

| Operator | Latin   | Description    | Inclusive |
| -------- | ------- | -------------- | --------- |
| `..`     |         | Range          | Exclusive |
|          | `ante`  | Range (before) | Exclusive |
|          | `usque` | Range (up to)  | Inclusive |
|          | `per`   | Step modifier  | N/A       |

```faber
0..10           // 0, 1, 2, ..., 9
0 ante 10       // 0, 1, 2, ..., 9
0 usque 10      // 0, 1, 2, ..., 10
0..10 per 2     // 0, 2, 4, 6, 8
```

---

## Spread and Rest

| Keyword  | Description     | Example                  |
| -------- | --------------- | ------------------------ |
| `sparge` | Spread elements | `[sparge a, sparge b]`   |
| `ceteri` | Rest parameters | `functio f(ceteri args)` |

```faber
// Array spread
fixum combined = [sparge first, sparge second]

// Object spread
fixum merged = { sparge defaults, timeout: 5000 }

// Call spread
add(sparge nums)

// Rest parameters
functio sum(ceteri lista<numerus> nums) -> numerus
```

---

## Conditional (Ternary)

| Syntax                          | Description      |
| ------------------------------- | ---------------- |
| `condition ? then : else`       | Standard ternary |
| `condition sic then secus else` | Latin ternary    |

```faber
fixum max = a > b ? a : b
fixum max = a > b sic a secus b
```

---

## Special Unary Operators

| Operator | Latin       | Description   | Example       |
| -------- | ----------- | ------------- | ------------- |
|          | `nulla`     | Is null/empty | `nulla x`     |
|          | `nonnulla`  | Has content   | `nonnulla x`  |
|          | `negativum` | Is negative   | `negativum x` |
|          | `positivum` | Is positive   | `positivum x` |

These expand to inline checks:

```faber
nulla x        // x == null || x.length === 0 || ...
nonnulla x     // x != null && x.length > 0 && ...
negativum x    // x < 0
positivum x    // x > 0
```

---

## Assignment

| Operator | Description |
| -------- | ----------- |
| `=`      | Assignment  |

**Not supported:** `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`

---

## Target Mappings

### Bitwise Operators

| Faber | TypeScript | Python | Zig  | C++  | Rust |
| ----- | ---------- | ------ | ---- | ---- | ---- |
| `&`   | `&`        | `&`    | `&`  | `&`  | `&`  |
| `\|`  | `\|`       | `\|`   | `\|` | `\|` | `\|` |
| `^`   | `^`        | `^`    | `^`  | `^`  | `^`  |
| `~`   | `~`        | `~`    | `~`  | `~`  | `!`  |
| `<<`  | `<<`       | `<<`   | `<<` | `<<` | `<<` |
| `>>`  | `>>`       | `>>`   | `>>` | `>>` | `>>` |

**Note:** Rust uses `!` for bitwise NOT instead of `~`.

### Logical Operators

| Faber          | TypeScript | Python | Zig   | C++    |
| -------------- | ---------- | ------ | ----- | ------ |
| `&&` / `et`    | `&&`       | `and`  | `and` | `&&`   |
| `\|\|` / `aut` | `\|\|`     | `or`   | `or`  | `\|\|` |
| `!` / `non`    | `!`        | `not`  | `!`   | `!`    |

### Nullish Coalescing

| Faber        | TypeScript | Python                      | Zig      |
| ------------ | ---------- | --------------------------- | -------- |
| `vel` / `??` | `??`       | `x if x is not None else y` | `orelse` |

### Type Cast

| Faber    | TypeScript | Python | Zig         | C++                 |
| -------- | ---------- | ------ | ----------- | ------------------- |
| `x ut T` | `x as T`   | N/A    | `@as(T, x)` | `static_cast<T>(x)` |

---

## Not Yet Implemented

| Feature             | Description                        | Status        |
| ------------------- | ---------------------------------- | ------------- |
| `sed /pattern/`     | Regex literals                     | Design only   |
| `est` (type check)  | `x est textus` → typeof/instanceof | Design only   |
| Binary literals     | `0b1010`                           | Not supported |
| `>>>`               | Unsigned right shift               | Not planned   |
| Compound assignment | `+=`, `-=`, etc.                   | Not planned   |
