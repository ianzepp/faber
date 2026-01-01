---
status: planned
targets: [ts, py, zig, rs, cpp]
note: Random number generation as stdlib module
updated: 2024-12
---

# Aleator - Random Number Generation

## Overview

Random number generation: floats, integers, bytes, UUIDs, and collection operations.

## Usage

```fab
ex "norma/aleator" importa fractus, numerus, elige

fixum f = fractus()              # 0.0 <= f < 1.0
fixum n = numerus(1, 100)        # 1 <= n <= 100
fixum item = elige(items)        # random element from list
```

## Etymology

| Latin     | Meaning              | Maps To               |
| --------- | -------------------- | --------------------- |
| `aleator` | dice player, gambler | module name           |
| `fractus` | broken (fraction)    | random float 0-1      |
| `inter`   | between              | random int in range   |
| `octeti`  | bytes                | random bytes          |
| `uuid`    | (borrowed)           | UUID v4               |
| `selige`  | choose, select       | random pick from list |
| `misce`   | mix, shuffle         | shuffle (copy)        |
| `semen`   | seed                 | set RNG seed          |

---

## Functions

### Basic Generation

| Faber             | Description                  | Returns   |
| ----------------- | ---------------------------- | --------- |
| `fractus()`       | Random float in [0, 1)       | `fractus` |
| `inter(min, max)` | Random integer in [min, max] | `numerus` |
| `octeti(n)`       | n random bytes               | `octeti`  |
| `uuid()`          | UUID v4 string               | `textus`  |

### Collection Operations

| Faber           | Description              | Returns |
| --------------- | ------------------------ | ------- |
| `selige(lista)` | Random element from list | `T`     |
| `misce(lista)`  | Shuffled copy of list    | `T[]`   |

### Seeding

| Faber      | Description  | Returns  |
| ---------- | ------------ | -------- |
| `semen(n)` | Seed the RNG | `vacuum` |

---

## Target Mappings

### TypeScript

```typescript
// ex "norma/aleator" importa fractus, numerus, uuid

Math.random(); // fractus()
Math.floor(Math.random() * (max - min + 1)) + min; // numerus(min, max)
crypto.randomUUID(); // uuid()
```

### Python

```python
# ex "norma/aleator" importa fractus, numerus, uuid
import random
import uuid as uuid_mod

random.random()                    # fractus()
random.randint(min, max)           # numerus(min, max)
str(uuid_mod.uuid4())              # uuid()
random.choice(lista)               # elige(lista)
random.sample(lista, len(lista))   # misce(lista)
random.seed(n)                     # semen(n)
```

### Zig

```zig
// ex "norma/aleator" importa fractus, numerus
const std = @import("std");

// Requires RNG state initialization
var prng = std.rand.DefaultPrng.init(seed);
const random = prng.random();

random.float(f64);                          // fractus()
random.intRangeAtMost(i64, min, max);       // numerus(min, max)
random.bytes(&buffer);                       // octeti(n)
```

### Rust

```rust
// ex "norma/aleator" importa fractus, numerus, uuid
use rand::Rng;
use uuid::Uuid;

rand::random::<f64>();                      // fractus()
rand::thread_rng().gen_range(min..=max);    // numerus(min, max)
Uuid::new_v4().to_string();                 // uuid()
```

### C++

```cpp
// ex "norma/aleator" importa fractus, numerus
#include <random>

std::random_device rd;
std::mt19937 gen(rd());
std::uniform_real_distribution<> dis(0.0, 1.0);

dis(gen);                                    // fractus()
std::uniform_int_distribution<>(min, max)(gen); // numerus(min, max)
```

---

## Design Decisions

### Why `fractus()` not `numerus()`?

`numerus` already means "integer" in Faber. For a random float, `fractus` (fraction) better describes the 0-1 range output. Also avoids confusion with `aleator.numerus(min, max)` which returns an integer.

### Why module, not intrinsic?

1. **Target variance** - Random APIs differ significantly across targets (seedable vs not, CSPRNG vs PRNG)
2. **State management** - Zig requires explicit RNG state; intrinsics can't handle this cleanly
3. **Consistency** - Matches `mathesis` and other stdlib modules

### Seeding considerations

Not all targets support seeding:

- **TypeScript**: `Math.random()` is not seedable (need external lib)
- **Python**: `random.seed()` works
- **Zig**: Explicit PRNG state
- **Rust**: `rand::SeedableRng`
- **C++**: `std::mt19937` constructor

For unseeded random, the default target behavior is used. `semen()` may emit a warning or no-op on targets that don't support seeding.

### CSPRNG vs PRNG

For `octeti()` and `uuid()`, cryptographically secure randomness is preferred:

- **TypeScript**: `crypto.getRandomValues()`
- **Python**: `secrets` module
- **Rust**: `rand::rngs::OsRng`
- **Zig**: `std.crypto.random`

For `fractus()` and `numerus()`, fast PRNG is acceptable.

---

## Implementation Status

| Feature             | TS  | Py  | Zig | Rust | C++ |
| ------------------- | --- | --- | --- | ---- | --- |
| `fractus()` (float) | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `numerus(min, max)` | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `octeti(n)` (bytes) | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `uuid()` (UUID v4)  | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `elige(lista)`      | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `misce(lista)`      | [ ] | [ ] | [ ] | [ ]  | [ ] |
| `semen(n)` (seed)   | [ ] | [ ] | [ ] | [ ]  | [ ] |
