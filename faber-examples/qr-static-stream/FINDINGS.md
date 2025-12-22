# QR Static Stream - Compiler Findings

Port of `qr-static-stream` Python project to Faber Romanus.

## Status: Working

The QR static stream algorithm now compiles and runs correctly.

```bash
# Compile norma.fab (stdlib) + qr_static.fab, then run
(bun run src/cli.ts compile src/stdlib/norma.fab && \
 bun run src/cli.ts compile faber-examples/qr-static-stream/qr_static.fab | grep -v "^import") \
 > /tmp/qr.ts && bun run /tmp/qr.ts
```

Output:
```
Original pattern:
#.#.
.#.#
#.#.
.#.#
Encoded into 4 frames
Frame 0: [random noise]
Frame 1: [random noise]
Frame 2: [random noise]
Frame 3: [computed frame]
Recovered pattern:
#.#.
.#.#
#.#.
.#.#
```

## Issues Resolved During Port

### 1. Array Literals

Added `ArrayExpression` to parser and codegen.

### 2. Arrow Return Type Syntax

Changed from `functio Textus foo()` to `functio foo() -> Textus`.

### 3. Standard Library (norma.fab)

Created stdlib written in Faber itself:
- `series(n)` — range-like sequence 0..n-1
- `seriesAb(start, end, step)` — range with bounds
- `fortuitus()` — random number
- `pavimentum(n)` — floor
- Math utilities: `absolutus`, `minimus`, `maximus`, etc.

### 4. Intrinsics

Added target-specific intrinsics that norma.fab builds on:
- I/O: `scribe`, `vide`, `mone`, `lege`
- Math: `_fortuitus`, `_pavimentum`, `_tectum`, `_radix`, `_potentia`

### 5. For-of vs For-in

Latin `ex collection pro x` → JS `for (x of collection)` (values)
Latin `in collection pro x` → JS `for (x in collection)` (keys)

## Language Features Exercised

- Array literals and nested arrays
- Function declarations with return types
- For-of loops over arrays
- While loops with counters
- Conditionals (si/aliter)
- Method calls (push, length)
- Member access with computed indices (a[i][j])
- Arithmetic and modulo operations
- Imports from norma stdlib
