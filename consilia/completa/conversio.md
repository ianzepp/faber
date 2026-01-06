# Conversio - Type Conversion Operators

## Summary

Add postfix conversion operators for transforming values between primitive types: `numeratum`, `fractatum`, `textatum`, `bivalentum`.

## Motivation

Faber currently lacks string-to-number parsing and primitive type conversion:

```fab
# No way to do this today
fixum input = "42"
fixum n = ???  # parseInt equivalent?
```

The existing cast operators don't help:
- `qua` is a type assertion (no runtime conversion)
- `innatum` constructs native collection types

We need runtime type conversion that maps cleanly to each target language's idioms.

## Syntax

Postfix conversion keywords using Latin perfect passive participles:

```fab
expression 'numeratum'              # convert to numerus (panics on failure)
expression 'numeratum' 'vel' expr   # convert with fallback
expression 'fractatum'              # convert to fractus
expression 'textatum'               # convert to textus (infallible)
expression 'bivalentum'             # convert to bivalens (truthiness)
```

## Examples

```fab
# String to number (panics on parse failure)
fixum n = "42" numeratum
fixum f = "3.14" fractatum

# With fallback (vel = "or else")
fixum n = "bad" numeratum vel 0        # fallback to 0
fixum f = "bad" fractatum vel 0.0      # fallback to 0.0
fixum n = "bad" numeratum vel nihil    # returns numerus? (nullable)

# Sized integer types
fixum n = "42" numeratum<i32>          # signed 32-bit
fixum n = "42" numeratum<u64>          # unsigned 64-bit
fixum big = "12345678901234" numeratum<magnus>  # bigint

# Radix (base) for parsing using predefined types
fixum hex = "ff" numeratum<i32, Hex>   # 255 (hex)
fixum bin = "101" numeratum<u8, Bin>   # 5 (binary)
fixum oct = "777" numeratum<i32, Oct>  # 511 (octal)

# Number to string (infallible)
fixum s = 42 textatum                  # "42"
fixum s = 3.14 textatum                # "3.14"
fixum s = verum textatum               # "verum"

# Truthiness conversion (follows nonnulla semantics)
fixum b = 0 bivalentum                 # falsum (zero)
fixum b = 1 bivalentum                 # verum (non-zero)
fixum b = "" bivalentum                # falsum (empty)
fixum b = "hello" bivalentum           # verum (non-empty)
fixum b = [] bivalentum                # falsum (empty collection)
fixum b = [1, 2] bivalentum            # verum (non-empty collection)
fixum b = nihil bivalentum             # falsum (null)

# Chaining (left-associative)
fixum s = "42" numeratum textatum      # "42" -> 42 -> "42"

# With vel (standard left-associativity)
fixum n = x numeratum vel 0 vel 1      # ((x numeratum vel 0) vel 1)

# Object destructuring with conversion
fixum config = { port: "8080", timeout: "invalid", debug: "1" }
ex config fixum port numeratum, timeout numeratum vel 30 ut maxWait, debug bivalentum
# port = 8080, maxWait = 30 (fallback), debug = verum

# Array destructuring with conversion
fixum args = ["9000", "60"]
fixum [port numeratum, timeout numeratum vel 30] = args
```

## Etymology

The `-atum` suffix is the neuter perfect passive participle in Latin, meaning "having been made into X":

| Keyword | Derivation | Meaning |
|---------|------------|---------|
| `numeratum` | numerus + -atum | "made into a number" |
| `fractatum` | fractus + -atum | "made into a fraction" |
| `textatum` | textus + -atum | "made into text" |
| `bivalentum` | bivalens + -um | "made into a boolean" |

This follows Faber's pattern of adjectives/modifiers appearing after the noun (e.g., `genus Foo publicum`).

## Grammar

```ebnf
cast := call ('qua' typeAnnotation
            | 'innatum' typeAnnotation
            | conversionOp ('vel' expression)?)*

conversionOp := ('numeratum' | 'fractatum') typeParams?
              | 'textatum'
              | 'bivalentum'

typeParams := '<' typeAnnotation (',' radixType)? '>'

radixType := 'Dec' | 'Hex' | 'Oct' | 'Bin'
```

The `vel` clause is part of the conversion production, binding to the immediately following expression. Any subsequent `vel` is the binary nullish-coalescing operator at lower precedence.

For `numeratum` and `fractatum`, optional type parameters specify:
1. First parameter: target subtype (`i32`, `u64`, `magnus`, etc.)
2. Second parameter: radix type (`Dec` = decimal, `Hex` = hexadecimal, `Bin` = binary, `Oct` = octal)

**Note on destructuring:** The examples show conversion operators in destructuring patterns (e.g., `port numeratum`). This requires extending the destructuring grammar to support conversion as part of binding patterns.

## Codegen

### numeratum (string/value to integer)

| Faber | TypeScript | Python | Rust | C++ | Zig |
|-------|------------|--------|------|-----|-----|
| `"42" numeratum` | `parseInt("42", 10)` | `int("42")` | `"42".parse::<i64>().unwrap()` | `std::stoll("42")` | `std.fmt.parseInt(i64, "42", 10) catch unreachable` |
| `"42" numeratum vel 0` | `parseInt("42", 10) \|\| 0` | `int("42") if "42".isdigit() else 0` | `"42".parse::<i64>().unwrap_or(0)` | `[&]{ try { return std::stoll("42"); } catch(...) { return 0; } }()` | `std.fmt.parseInt(i64, "42", 10) catch 0` |
| `"42" numeratum<i32>` | `parseInt("42", 10)` | `int("42")` | `"42".parse::<i32>().unwrap()` | `std::stoi("42")` | `std.fmt.parseInt(i32, "42", 10) catch unreachable` |
| `"ff" numeratum<i32, Hex>` | `parseInt("ff", 16)` | `int("ff", 16)` | `i32::from_str_radix("ff", 16).unwrap()` | `std::stoi("ff", nullptr, 16)` | `std.fmt.parseInt(i32, "ff", 16) catch unreachable` |
| `"big" numeratum<magnus>` | `BigInt("big")` | `int("big")` | `"big".parse::<num_bigint::BigInt>().unwrap()` | `boost::multiprecision::cpp_int("big")` | `std.math.big.Int.init(...)` |

**Radix Values:**
| Faber | Base |
|-------|------|
| `Dec` | 10 |
| `Hex` | 16 |
| `Oct` | 8 |
| `Bin` | 2 |

### fractatum (string/value to float)

| Faber | TypeScript | Python | Rust | C++ | Zig |
|-------|------------|--------|------|-----|-----|
| `"3.14" fractatum` | `parseFloat("3.14")` | `float("3.14")` | `"3.14".parse::<f64>().unwrap()` | `std::stod("3.14")` | `std.fmt.parseFloat(f64, "3.14") catch unreachable` |
| `"3.14" fractatum vel 0.0` | `parseFloat("3.14") \|\| 0.0` | `float("3.14") if ... else 0.0` | `"3.14".parse::<f64>().unwrap_or(0.0)` | `...` | `std.fmt.parseFloat(f64, "3.14") catch 0.0` |

### textatum (any to string)

| Faber | TypeScript | Python | Rust | C++ | Zig |
|-------|------------|--------|------|-----|-----|
| `42 textatum` | `String(42)` | `str(42)` | `42.to_string()` | `std::to_string(42)` | `std.fmt.allocPrint(alloc, "{}", .{42})` |
| `verum textatum` | `String(true)` | `str(True)` | `true.to_string()` | `"verum"` | `"verum"` |

Note: `textatum` is infallible - no `vel` clause needed.

### bivalentum (any to boolean)

Follows `nonnulla` semantics: returns `verum` if the value is non-null AND non-empty/non-zero.

| Faber | TypeScript | Python | Rust | C++ | Zig |
|-------|------------|--------|------|-----|-----|
| `0 bivalentum` | `Boolean(0)` | `bool(0)` | `0 != 0` | `static_cast<bool>(0)` | `0 != 0` |
| `42 bivalentum` | `Boolean(42)` | `bool(42)` | `42 != 0` | `static_cast<bool>(42)` | `42 != 0` |
| `"" bivalentum` | `Boolean("")` | `bool("")` | `!s.is_empty()` | `!s.empty()` | `s.len != 0` |
| `"hi" bivalentum` | `Boolean("hi")` | `bool("hi")` | `!s.is_empty()` | `!s.empty()` | `s.len != 0` |
| `[] bivalentum` | `arr.length > 0` | `bool([])` | `!v.is_empty()` | `!v.empty()` | `v.items.len != 0` |
| `nihil bivalentum` | `x != null` | `x is not None` | `x.is_some()` | `x.has_value()` | `x != null` |
| `obj bivalentum` | `obj != null` | `obj is not None` | `obj.is_some()` | `obj.has_value()` | `obj != null` |

Note: `bivalentum` is infallible - no `vel` clause needed. Semantics match the existing `nonnulla` unary operator. For user-defined objects, `bivalentum` returns `verum` if non-null (same as `nonnulla`).

## Semantics

### Type Compatibility

| Source Type | numeratum | fractatum | textatum | bivalentum |
|-------------|-----------|-----------|----------|------------|
| `textus` | parse | parse | identity | non-empty |
| `numerus` | identity | widen | format | non-zero |
| `fractus` | truncate | identity | format | non-zero |
| `bivalens` | 0/1 | 0.0/1.0 | "verum"/"falsum" | identity |
| `genus` (object) | error | error | format | non-null |

### Error Handling

Without `vel`:
- Parse failure panics (like Rust's `.unwrap()`)
- Compile error for invalid source types

With `vel`:
- Parse failure returns the fallback value
- `vel nihil` returns nullable type (`numerus?`, `fractus?`)
- Fallback type must match result type

### Precedence

Conversion operators are at the same precedence level as `qua` and `innatum` (high, in the cast production). They are left-associative:

```fab
x numeratum vel 0 vel 1
# Parses as: ((x numeratum vel 0) vel 1)
# First vel is conversion fallback, second is binary nullish-coalesce
```

## Comparison with Existing Operators

| Operator | Purpose | Runtime Effect |
|----------|---------|----------------|
| `qua` | Type assertion | None - compile-time only |
| `innatum` | Native collection construction | Creates proper native instance |
| `numeratum` etc. | Type conversion | Runtime parsing/formatting |

## AST Nodes

```typescript
export interface ConversionExpression extends BaseNode {
    type: 'ConversionExpression';
    expression: Expression;
    conversion: 'numeratum' | 'fractatum' | 'textatum' | 'bivalentum';
    targetType?: TypeAnnotation;  // e.g., i32, u64, magnus
    radix?: 'Dec' | 'Hex' | 'Oct' | 'Bin';
    fallback?: Expression;  // present if 'vel' clause
}
```

## Implementation Checklist

### Phase 1: Keywords
- [x] `fons/faber/lexicon/keywords.ts` — Add conversion operator keywords (~line 280, Prepositions section):
  ```typescript
  { latin: 'numeratum', meaning: 'to number', category: 'operator' },
  { latin: 'fractatum', meaning: 'to float', category: 'operator' },
  { latin: 'textatum', meaning: 'to string', category: 'operator' },
  { latin: 'bivalentum', meaning: 'to boolean', category: 'operator' },
  { latin: 'Hex', meaning: 'hexadecimal radix', category: 'type' },
  { latin: 'Oct', meaning: 'octal radix', category: 'type' },
  { latin: 'Bin', meaning: 'binary radix', category: 'type' },
  { latin: 'Dec', meaning: 'decimal radix', category: 'type' },
  ```

### Phase 2: AST
- [x] `fons/faber/parser/ast.ts` — Add `ConversionExpression` interface (~line 2304, after `InnatumExpression`):
  ```typescript
  export interface ConversionExpression extends BaseNode {
      type: 'ConversionExpression';
      expression: Expression;
      conversion: 'numeratum' | 'fractatum' | 'textatum' | 'bivalentum';
      targetType?: TypeAnnotation;
      radix?: 'Dec' | 'Hex' | 'Oct' | 'Bin';
      fallback?: Expression;
  }
  ```
- [x] `fons/faber/parser/ast.ts` — Add `ConversionExpression` to `Expression` union type

### Phase 3: Parser
- [x] `fons/faber/parser/index.ts` — Extend `parseQuaExpression()` (~line 4869, after `innatum` case):
  ```typescript
  else if (matchKeyword('numeratum') || matchKeyword('fractatum') ||
           matchKeyword('textatum') || matchKeyword('bivalentum')) {
      const conversion = previous().value;
      let targetType: TypeAnnotation | undefined;
      let radix: string | undefined;
      let fallback: Expression | undefined;

      if ((conversion === 'numeratum' || conversion === 'fractatum') && match('<')) {
          targetType = parseTypeAnnotation();
          if (match(',')) {
              radix = consume(['Hex', 'Oct', 'Bin', 'Dec']).value;
          }
          consume('>');
      }

      if (matchKeyword('vel')) {
          fallback = parseUnary();
      }

      expr = {
          type: 'ConversionExpression',
          expression: expr,
          conversion,
          targetType,
          radix,
          fallback,
          position: expr.position
      };
  }
  ```

### Phase 4: Semantic Analysis
- [x] `fons/faber/semantic/index.ts` — Add `ConversionExpression` case in `resolveExpression()` (~line 834):
  - Resolve source expression type
  - Validate source → target conversion is valid (see Type Compatibility table)
  - If `fallback` present, validate fallback type matches result type
  - If `fallback` is `nihil`, mark result type as nullable
  - Set `node.resolvedType`

### Phase 5: Codegen — TypeScript (primary target)
- [x] `fons/faber/codegen/ts/expressions/conversio.ts` — Create new file with `genConversionExpression()`
- [x] `fons/faber/codegen/ts/generator.ts` — Add dispatch case (~line 318)
- [x] TypeScript codegen mapping:
  | Faber | TypeScript |
  |-------|------------|
  | `x numeratum` | `parseInt(x, 10)` |
  | `x numeratum vel f` | `parseInt(x, 10) \|\| f` |
  | `x numeratum<i32, Hex>` | `parseInt(x, 16)` |
  | `x fractatum` | `parseFloat(x)` |
  | `x textatum` | `String(x)` |
  | `x bivalentum` | `Boolean(x)` or `x != null && x !== '' && x !== 0` |

### Phase 6: Codegen — Other Targets

Each target requires two changes:
1. Create `expressions/conversio.ts` with `genConversionExpression()`
2. Add dispatch case in `generator.ts`

**Rust:**
- [x] `fons/faber/codegen/rs/expressions/conversio.ts`
- [x] `fons/faber/codegen/rs/generator.ts` — add dispatch (~line 260)
- Pattern: `.parse::<T>().unwrap()` / `.unwrap_or(f)` / `from_str_radix()`

**Python:**
- [x] `fons/faber/codegen/py/expressions/conversio.ts`
- [x] `fons/faber/codegen/py/generator.ts` — add dispatch (~line 285)
- Pattern: `int()` / `float()` / `str()` / `bool()` with optional base

**C++:**
- [x] `fons/faber/codegen/cpp/expressions/conversio.ts`
- [x] `fons/faber/codegen/cpp/generator.ts` — add dispatch (~line 317)
- Pattern: `std::stoll()` / `std::stod()` / `std::to_string()` / IIFE for fallback

**Zig:**
- [x] `fons/faber/codegen/zig/expressions/conversio.ts`
- [x] `fons/faber/codegen/zig/generator.ts` — add dispatch (~line 423)
- Pattern: `std.fmt.parseInt()` / `parseFloat()` with `catch` for fallback

**Faber (identity):**
- [x] `fons/faber/codegen/fab/expressions/conversio.ts`
- [x] `fons/faber/codegen/fab/generator.ts` — add dispatch (~line 240)
- Pattern: Re-emit `x numeratum`, `x numeratum vel f`, etc.

### Phase 7: Tests

- [x] `fons/proba/codegen/expressions/conversio.yaml` — Create test file (57 tests pass)

**Test file structure** (follows `qua.yaml` and `innatum.yaml` patterns):

```yaml
# Conversio expressions - type conversion operators
# Maps Faber numeratum/fractatum/textatum/bivalentum to target conversions

# === numeratum (to integer) ===

- name: basic numeratum
  source: fixum n = "42" numeratum
  expect:
      ts: 'const n = parseInt("42", 10);'
      py: 'n = int("42")'
      rs:
          - '"42".parse::<i64>().unwrap()'
      cpp:
          - 'std::stoll("42")'
      zig:
          - 'std.fmt.parseInt(i64, "42", 10)'
      fab:
          - 'fixum n = "42" numeratum'

- name: numeratum with fallback
  source: fixum n = input numeratum vel 0
  expect:
      ts: 'const n = parseInt(input, 10) || 0;'
      py: 'n = int(input) if input.isdigit() else 0'
      rs:
          - '.parse::<i64>().unwrap_or(0)'
      fab:
          - 'fixum n = input numeratum vel 0'

- name: numeratum with type param
  source: fixum n = "42" numeratum<i32>
  expect:
      ts: 'const n = parseInt("42", 10);'
      rs:
          - '"42".parse::<i32>().unwrap()'
      fab:
          - 'fixum n = "42" numeratum<i32>'

- name: numeratum with hex radix
  source: fixum n = "ff" numeratum<i32, Hex>
  expect:
      ts: 'const n = parseInt("ff", 16);'
      py: 'n = int("ff", 16)'
      rs:
          - 'i32::from_str_radix("ff", 16).unwrap()'
      fab:
          - 'fixum n = "ff" numeratum<i32, Hex>'

- name: numeratum with binary radix
  source: fixum n = "101" numeratum<u8, Bin>
  expect:
      ts: 'const n = parseInt("101", 2);'
      py: 'n = int("101", 2)'
      rs:
          - 'u8::from_str_radix("101", 2).unwrap()'
      fab:
          - 'fixum n = "101" numeratum<u8, Bin>'

# === fractatum (to float) ===

- name: basic fractatum
  source: fixum f = "3.14" fractatum
  expect:
      ts: 'const f = parseFloat("3.14");'
      py: 'f = float("3.14")'
      rs:
          - '"3.14".parse::<f64>().unwrap()'
      cpp:
          - 'std::stod("3.14")'
      fab:
          - 'fixum f = "3.14" fractatum'

- name: fractatum with fallback
  source: fixum f = input fractatum vel 0.0
  expect:
      ts: 'const f = parseFloat(input) || 0;'
      rs:
          - '.parse::<f64>().unwrap_or(0.0)'
      fab:
          - 'fixum f = input fractatum vel 0.0'

# === textatum (to string) ===

- name: number to string
  source: fixum s = 42 textatum
  expect:
      ts: 'const s = String(42);'
      py: 's = str(42)'
      rs:
          - '42.to_string()'
      cpp:
          - 'std::to_string(42)'
      fab:
          - 'fixum s = 42 textatum'

- name: boolean to string
  source: fixum s = verum textatum
  expect:
      ts: 'const s = String(true);'
      py: 's = str(True)'
      rs:
          - 'true.to_string()'
      fab:
          - 'fixum s = verum textatum'

# === bivalentum (to boolean) ===

- name: number to boolean
  source: fixum b = count bivalentum
  expect:
      ts: 'const b = Boolean(count);'
      py: 'b = bool(count)'
      rs:
          - 'count != 0'
      fab:
          - 'fixum b = count bivalentum'

- name: string to boolean
  source: fixum b = name bivalentum
  expect:
      ts: 'const b = Boolean(name);'
      py: 'b = bool(name)'
      rs:
          - '!name.is_empty()'
      fab:
          - 'fixum b = name bivalentum'

# === chaining ===

- name: conversion chain numeratum then textatum
  source: fixum s = "42" numeratum textatum
  expect:
      ts: 'const s = String(parseInt("42", 10));'
      py: 's = str(int("42"))'
      rs:
          - '.parse::<i64>().unwrap().to_string()'
      fab:
          - 'fixum s = "42" numeratum textatum'

- name: chained with fallback
  source: fixum n = x numeratum vel 0 vel 1
  expect:
      ts: 'const n = (parseInt(x, 10) || 0) ?? 1;'
      fab:
          - 'fixum n = x numeratum vel 0 vel 1'
```

### Key Reference Files
| Component | File | Template to Follow |
|-----------|------|-------------------|
| Keywords | `fons/faber/lexicon/keywords.ts` | `qua`, `innatum` entries |
| AST | `fons/faber/parser/ast.ts:2257-2303` | `QuaExpression`, `InnatumExpression` |
| Parser | `fons/faber/parser/index.ts:4846-4873` | `parseQuaExpression()` |
| Semantic | `fons/faber/semantic/index.ts:797-832` | `QuaExpression`, `InnatumExpression` cases |
| TS Codegen | `fons/faber/codegen/ts/expressions/qua.ts` | 34-line simple pattern |
| TS Codegen | `fons/faber/codegen/ts/expressions/innatum.ts` | 70-line with type checking |

## Error Cases

```fab
# Error: cannot convert genus to numerus
fixum n = meumObjectum numeratum

# Error: fallback type mismatch
fixum n = "42" numeratum vel "zero"  # expected numerus, got textus

# Error: textatum doesn't need vel (infallible)
fixum s = 42 textatum vel "default"  # warning: vel clause has no effect
```

## Open Questions

1. **Custom conversions**: Should user-defined types be able to implement conversion? Perhaps via a `pactum Convertibilis` pattern?

2. **Radix output for textatum**: Should `textatum` support formatting numbers in other bases? Perhaps `42 textatum<16>` → `"2a"`?

3. **Precision for fractatum**: Should `fractatum<f32>` vs `fractatum<f64>` be supported for explicit precision?

## Design Decisions

1. **`bivalentum` follows `nonnulla` semantics**: The existing `nonnulla` unary operator checks for non-null AND non-empty. `bivalentum` applies the same logic for type conversion consistency.

2. **Radix as predefined types**: Rather than literal numbers (`16`, `2`, `8`) or auto-detection from prefixes (`0xff`), radix uses predefined type names: `Hex`, `Oct`, `Bin`, `Dec`. This keeps angle brackets pure types (no mixed literals) and provides readable, self-documenting syntax.

3. **Type parameter for numeric subtypes**: Instead of separate keywords (`magnatum`, `i32atum`), we use `numeratum<magnus>` and `numeratum<i32>`. This scales to any numeric subtype without keyword proliferation.

4. **`vel` as separate token**: The fallback clause uses the existing `vel` keyword as a separate token (`numeratum vel 0`) rather than a suffix (`numeratumVel`), maintaining consistency with `vel`'s use as nullish coalescing.

5. **`bivalentum` on objects**: User-defined objects (`genus`) converted via `bivalentum` return `verum` if non-null, matching `nonnulla` semantics. Objects cannot be converted via `numeratum` or `fractatum` (compile error).

## Rivus Implementation

**Status: Complete**

The Rivus compiler (bootstrap compiler written in Faber) requires parallel changes to support conversio. Unlike Faber (TypeScript), Rivus uses `.fab` files with `discretio` (tagged unions) instead of interfaces.

### Files Modified

| Component | File | Changes |
|-----------|------|---------|
| **Lexicon enum** | `fons/rivus/ast/lexema.fab` | Added `Numeratum`, `Fractatum`, `Textatum`, `Bivalentum` to `VerbumId` enum |
| **Keywords** | `fons/rivus/lexicon/verba.fab` | Added keyword→VerbumId mappings in `verbumFaberId()` |
| **AST** | `fons/rivus/ast/expressia.fab` | Added `ConversioExpressia` variant to `Expressia` discretio |
| **Parser** | `fons/rivus/parser/expressia/unaria.fab` | Extended `parsePostfix()` after `qua` case |
| **Semantic** | `fons/rivus/semantic/expressia/index.fab` | Added case in expression dispatch |
| **Codegen** | `fons/rivus/codegen/ts/expressia/index.fab` | Added TS generation logic |

**Note:** Radix types (`Hex`, `Oct`, `Bin`, `Dec`) are parsed as identifiers rather than keywords to avoid breaking existing code that uses those names as enum members or variables.

### AST Variant

```fab
ConversionExpressia {
    Locus locus
    Expressia expressia
    textus signum              # "numeratum" | "fractatum" | "textatum" | "bivalentum"
    TypusAnnotatio? targetType # optional type param (e.g., i32, u64)
    textus? radix              # optional: "Dec" | "Hex" | "Oct" | "Bin"
    Expressia? fallback        # optional vel fallback
}
```

### Parser Logic (pseudocode)

```fab
# In parsePostfix(), after qua case:
si p.congruetVerbum("numeratum") aut p.congruetVerbum("fractatum") aut
   p.congruetVerbum("textatum") aut p.congruetVerbum("bivalentum") {
    fixum signum = p.procede().verbum
    varia targetType: TypusAnnotatio? = nihil
    varia radix: textus? = nihil
    varia fallback: Expressia? = nihil

    # Type params for numeratum/fractatum only
    si (signum == "numeratum" aut signum == "fractatum") et p.proba(SymbolumGenus.Minor) {
        p.procede()
        targetType = r.adnotatio()
        si p.congruet(SymbolumGenus.Coma) {
            fixum radixToken = p.specta(0)
            si radixToken.verbum inter ["Dec", "Hex", "Oct", "Bin"] {
                radix = p.procede().verbum
            }
        }
        p.expecta(SymbolumGenus.Maior, ParserErrorCodice.ExpectaturMaior)
    }

    # Fallback clause
    si p.probaVerbum("vel") {
        p.procede()
        fallback = parseUnaria(r)
    }

    expressia = finge ConversionExpressia { ... } qua Expressia
}
```

### Codegen Logic (pseudocode)

```fab
casu ConversionExpressia ut c {
    fixum expr = genExpressia(c.expressia, g)
    elige c.signum {
        casu "numeratum" {
            fixum radix = c.radix == "Hex" sic 16 secus c.radix == "Oct" sic 8 secus c.radix == "Bin" sic 2 secus 10
            si nonnihil c.fallback {
                redde scriptum("(parseInt(§, §) || §)", expr, radix, genExpressia(c.fallback, g))
            }
            redde scriptum("parseInt(§, §)", expr, radix)
        }
        casu "fractatum" { ... }
        casu "textatum" reddit scriptum("String(§)", expr)
        casu "bivalentum" reddit scriptum("Boolean(§)", expr)
    }
}
```

### Build & Test

```bash
# After modifying .fab files:
bun run build:rivus              # Rebuild rivus to opus/rivus/fons/ts/

# Test against rivus:
bun run test:rivus -t "conversio"

# Or run all rivus tests:
bun run test:rivus
```

### Key Differences from Faber

| Aspect | Faber (TypeScript) | Rivus (Faber) |
|--------|-------------------|---------------|
| AST | `interface` with `type` discriminator | `discretio` variant |
| Keywords | Array of objects | `verbumFaberId()` function |
| Pattern match | `switch`/`case` | `discerne`/`casu` |
| String templates | Template literals | `scriptum()` |

## Related

- `qua` - type assertion (compile-time only)
- `innatum` - native collection construction
- `vel` - nullish coalescing operator
- `nulla` / `nonnulla` - null/empty checking operators
