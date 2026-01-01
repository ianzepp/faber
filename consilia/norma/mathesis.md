---
status: planned
targets: [ts, py, zig, rs, cpp]
note: Math functions as stdlib module, not intrinsics
updated: 2024-12
---

# Mathesis - Math Operations

## Overview

Mathematical functions and constants. Unlike I/O intrinsics (`_scribe`, etc.), math operations live in a stdlib module to maintain consistency with other stdlib modules.

## Usage

```fab
ex "norma/mathesis" importa pavimentum, radix, PI

fixum x = pavimentum(3.7)      # 3
fixum y = radix(16)            # 4
fixum area = PI * r * r
```

## Etymology

| Latin          | Meaning          | Maps To     |
| -------------- | ---------------- | ----------- |
| `mathesis`     | mathematics      | module name |
| `pavimentum`   | floor, pavement  | floor()     |
| `tectum`       | roof, ceiling    | ceil()      |
| `radix`        | root             | sqrt()      |
| `potentia`     | power            | pow()       |
| `absolutum`    | freed, absolute  | abs()       |
| `signum`       | sign, mark       | sign()      |
| `rotundum`     | rounded          | round()     |
| `truncatum`    | cut off          | trunc()     |
| `logarithmus`  | logarithm        | log()       |
| `exponens`     | rising out       | exp()       |
| `sinus`        | curve, fold      | sin()       |
| `cosinus`      | co-sine          | cos()       |
| `tangens`      | touching         | tan()       |
| `minimus`      | smallest         | min()       |
| `maximus`      | greatest         | max()       |
| `constringens` | binding together | clamp()     |

---

## Functions

### Rounding

| Faber           | Description       | TypeScript   | Python       | Zig      | Rust         |
| --------------- | ----------------- | ------------ | ------------ | -------- | ------------ |
| `pavimentum(x)` | Round toward -inf | `Math.floor` | `math.floor` | `@floor` | `f64::floor` |
| `tectum(x)`     | Round toward +inf | `Math.ceil`  | `math.ceil`  | `@ceil`  | `f64::ceil`  |
| `rotundum(x)`   | Round to nearest  | `Math.round` | `round`      | `@round` | `f64::round` |
| `truncatum(x)`  | Round toward zero | `Math.trunc` | `math.trunc` | `@trunc` | `f64::trunc` |

### Powers and Roots

| Faber              | Description      | TypeScript   | Python       | Zig            | Rust         |
| ------------------ | ---------------- | ------------ | ------------ | -------------- | ------------ |
| `radix(x)`         | Square root      | `Math.sqrt`  | `math.sqrt`  | `@sqrt`        | `f64::sqrt`  |
| `potentia(x, n)`   | x to the power n | `Math.pow`   | `math.pow`   | `std.math.pow` | `f64::powf`  |
| `logarithmus(x)`   | Natural log      | `Math.log`   | `math.log`   | `@log`         | `f64::ln`    |
| `logarithmus10(x)` | Log base 10      | `Math.log10` | `math.log10` | `@log10`       | `f64::log10` |
| `exponens(x)`      | e^x              | `Math.exp`   | `math.exp`   | `@exp`         | `f64::exp`   |

### Trigonometry

| Faber        | Description       | TypeScript | Python     | Zig    | Rust       |
| ------------ | ----------------- | ---------- | ---------- | ------ | ---------- |
| `sinus(x)`   | Sine (radians)    | `Math.sin` | `math.sin` | `@sin` | `f64::sin` |
| `cosinus(x)` | Cosine (radians)  | `Math.cos` | `math.cos` | `@cos` | `f64::cos` |
| `tangens(x)` | Tangent (radians) | `Math.tan` | `math.tan` | `@tan` | `f64::tan` |

### Absolute and Sign

| Faber          | Description     | TypeScript  | Python               | Zig    | Rust          |
| -------------- | --------------- | ----------- | -------------------- | ------ | ------------- |
| `absolutum(x)` | Absolute value  | `Math.abs`  | `abs`                | `@abs` | `f64::abs`    |
| `signum(x)`    | Sign (-1, 0, 1) | `Math.sign` | `math.copysign(1,x)` | custom | `f64::signum` |

### Min/Max/Clamp

| Faber                     | Description    | TypeScript                | Python                | Zig               | Rust         |
| ------------------------- | -------------- | ------------------------- | --------------------- | ----------------- | ------------ |
| `minimus(a, b)`           | Smaller of two | `Math.min`                | `min`                 | `@min`            | `f64::min`   |
| `maximus(a, b)`           | Larger of two  | `Math.max`                | `max`                 | `@max`            | `f64::max`   |
| `constringens(x, lo, hi)` | Clamp to range | `Math.min(Math.max(...))` | `max(lo, min(hi, x))` | `@max(@min(...))` | `f64::clamp` |

---

## Constants

| Faber | Description    | Value               |
| ----- | -------------- | ------------------- |
| `PI`  | Pi             | 3.14159265358979... |
| `E`   | Euler's number | 2.71828182845904... |
| `TAU` | 2 \* Pi        | 6.28318530717958... |

---

## Target Mappings

### TypeScript

```typescript
// ex "norma/mathesis" importa pavimentum, radix, PI
// becomes: (no import needed, Math is global)

Math.floor(x);
Math.sqrt(x);
Math.PI;
```

### Python

```python
# ex "norma/mathesis" importa pavimentum, radix, PI
import math

math.floor(x)
math.sqrt(x)
math.pi
```

### Zig

```zig
// ex "norma/mathesis" importa pavimentum, radix, PI
const std = @import("std");

@floor(x);
@sqrt(x);
std.math.pi;
```

### Rust

```rust
// ex "norma/mathesis" importa pavimentum, radix, PI
// (no import needed for f64 methods)

x.floor();
x.sqrt();
std::f64::consts::PI;
```

### C++

```cpp
// ex "norma/mathesis" importa pavimentum, radix, PI
#include <cmath>
#include <numbers>

std::floor(x);
std::sqrt(x);
std::numbers::pi;
```

---

## Design Decisions

### Why a module, not intrinsics?

1. **Consistency** - All other stdlib features (collections, time, crypto) are modules
2. **Discoverability** - `ex "norma/mathesis" importa` tells you what's available
3. **Target transparency** - Users know they're importing target-native functionality
4. **No magic** - Intrinsics appear from nowhere; imports are explicit

### Why not `numerus` module name?

`numerus` collides with the primitive type `numerus`. Using `mathesis` (Greek-derived Latin for "mathematics") is unambiguous.

### Why Latin function names?

Faber's design principle: semantic keywords over cryptic abbreviations. `pavimentum` (floor) and `tectum` (ceiling) are architectural metaphors that match the mathematical concept.

---

## Implementation Status

| Feature                 | TS  | Py  | Zig | Rust | C++ |
| ----------------------- | --- | --- | --- | ---- | --- |
| `pavimentum` (floor)    | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `tectum` (ceiling)      | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `radix` (sqrt)          | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `potentia` (pow)        | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `absolutum` (abs)       | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `signum` (sign)         | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `rotundum` (round)      | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `truncatum` (trunc)     | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `logarithmus` (log)     | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `logarithmus10` (log10) | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `exponens` (exp)        | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `sinus` (sin)           | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `cosinus` (cos)         | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `tangens` (tan)         | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `minimus` (min)         | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `maximus` (max)         | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `constringens` (clamp)  | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `PI` constant           | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `E` constant            | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `TAU` constant          | [ ] | [ ] | [ ] | [ ]  | [ ] |
