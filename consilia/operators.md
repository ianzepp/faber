# Operators Design

New operators for spread/rest, type casting, and type checking.

## Implementation Status

| Operator | Purpose | TypeScript | Zig | Python | Status |
|----------|---------|------------|-----|--------|--------|
| `sparge` | Spread | `...x` | TBD | `*x` | Not implemented |
| `ceteri` | Rest params | `...args` | TBD | `*args` | Not implemented |
| `ut` | Type cast | `x as T` | `@as(T, x)` | N/A | Not implemented |
| `est` (type) | Type check | `instanceof` | TBD | `isinstance` | Partial (equality only) |

---

## sparge (Spread)

**Etymology:** From Latin *spargere* — "to scatter, spread, sprinkle"

Spreads elements of an array or properties of an object into a new container.

### Array Spread

```faber
fixum a = [1, 2, 3]
fixum b = [4, 5, 6]

// Concatenation
fixum combined = [sparge a, sparge b]    // [1, 2, 3, 4, 5, 6]

// Shallow copy
fixum copy = [sparge a]                   // [1, 2, 3]

// Interleaved with literals
fixum wrapped = [0, sparge a, 99]         // [0, 1, 2, 3, 99]
```

### Object Spread

```faber
fixum defaults = { timeout: 5000, retries: 3 }
fixum custom = { sparge defaults, timeout: 10000 }
// { timeout: 10000, retries: 3 }

// Immutable update pattern
fixum user = { name: "Marcus", age: 30 }
fixum updated = { sparge user, age: 31 }
// { name: "Marcus", age: 31 }
```

### Function Call Spread

Pass array elements as individual arguments:

```faber
functio add(a, b, c) -> numerus {
    redde a + b + c
}

fixum nums = [1, 2, 3]
scribe add(sparge nums)  // 6
```

### Design Notes

- `sparge` is prefix syntax: `sparge x`, not `x sparge`
- Single-level only — `sparge sparge x` is a syntax error (use `.plana()` for flattening)
- Order matters for objects: later properties override earlier ones

### Target Mappings

| Target | Array Spread | Object Spread | Call Spread |
|--------|--------------|---------------|-------------|
| TypeScript | `[...a]` | `{...o}` | `fn(...a)` |
| Python | `[*a]` | `{**o}` | `fn(*a)` |
| Zig | Manual loop or `++` | N/A (structs) | N/A |

---

## ceteri (Rest)

**Etymology:** From Latin *ceteri* — "the rest, the others, the remaining"

Collects remaining elements into an array.

### Rest Parameters

Variadic function parameters:

```faber
functio sum(ceteri lista<numerus> nums) -> numerus {
    redde nums.reducta(0, (acc, n) => acc + n)
}

scribe sum(1, 2, 3, 4, 5)  // 15
```

With leading fixed parameters:

```faber
functio log(textus level, ceteri lista<textus> messages) {
    ex messages pro msg {
        scribe "[" + level + "]", msg
    }
}

log("INFO", "Starting", "Loading config", "Ready")
```

### Rest in Destructuring

Collect remaining object properties:

```faber
fixum user = { name: "Marcus", age: 30, city: "Roma", role: "admin" }

ex user fixum { name, ceteri profile }
// name = "Marcus"
// profile = { age: 30, city: "Roma", role: "admin" }
```

### Type Inference

When no type annotation is provided, `ceteri` infers `lista<ignotus>`:

```faber
functio f(ceteri args) { }  // args: lista<ignotus>
```

### Design Notes

- `ceteri` must be the last parameter in a function signature
- In destructuring, `ceteri` collects all unmentioned properties
- The collected value is always an array (for params) or object (for destructuring), never `nihil`
- Empty rest produces empty array `[]` or empty object `{}`

### Target Mappings

| Target | Rest Params | Rest Destructuring |
|--------|-------------|-------------------|
| TypeScript | `...args: T[]` | `{ a, ...rest }` |
| Python | `*args` | `**rest` (dict) |
| Zig | Slice parameter | N/A |

---

## ut (Type Cast)

**Etymology:** From Latin *ut* — "as, in the capacity of"

Asserts that a value has a specific type without runtime checking.

### Basic Usage

```faber
fixum data: ignotus = getData()
fixum name = data ut textus

// In expressions
scribe (response.body ut objectum).id
```

### With Type Checking

Typically used after narrowing with `est`:

```faber
functio process(data: ignotus) -> textus {
    si data est textus {
        redde data  // already narrowed, ut not needed
    }

    // Force cast when you know better than the compiler
    redde data ut textus
}
```

### Design Notes

- `ut` is a compile-time assertion — no runtime overhead or checking
- Use `est` first when possible to let the compiler narrow safely
- Incorrect casts produce undefined behavior (target-dependent)
- No `ut!` variant (runtime-checked cast) for now

### Target Mappings

| Target | Cast Syntax |
|--------|-------------|
| TypeScript | `x as T` |
| Python | N/A (dynamic typing) |
| Zig | `@as(T, x)` |
| Rust | `x as T` |
| C++ | `static_cast<T>(x)` |

---

## est (Type Check)

**Etymology:** From Latin *est* — "is, exists"

Already implemented for strict equality (`===`). Extends to type checking when the right-hand side is a type name.

### Current Behavior (Equality)

```faber
x est 5           // x === 5
x est "hello"     // x === "hello"
x est verum       // x === true
```

### Extended Behavior (Type Check)

When RHS is a type name:

```faber
// Primitive type check (typeof)
x est numerus     // typeof x === "number"
x est textus      // typeof x === "string"
x est bivalens    // typeof x === "boolean"

// Class/genus check (instanceof)
x est Persona     // x instanceof Persona
x est Error       // x instanceof Error
```

### In Conditionals

```faber
functio describe(value: ignotus) -> textus {
    elige {
        value est textus => redde "Text: " + value
        value est numerus => redde "Number: " + value
        value est bivalens => redde "Boolean: " + value
        aliter => redde "Unknown type"
    }
}
```

### Type Narrowing

After `est` check, the type is narrowed in that branch:

```faber
functio process(data: ignotus) {
    si data est textus {
        // data is textus here
        scribe data.longitudo()
    }
    aliter si data est lista<numerus> {
        // data is lista<numerus> here
        scribe data.summa()
    }
}
```

### Design Notes

- Context determines interpretation: RHS is a type name → type check; RHS is a value → equality
- The compiler distinguishes types from values at semantic analysis time
- Primitive types use `typeof`, genus/pactum use `instanceof`

### Target Mappings

| Target | Primitive Check | Class Check |
|--------|-----------------|-------------|
| TypeScript | `typeof x === "t"` | `x instanceof T` |
| Python | `isinstance(x, t)` | `isinstance(x, T)` |
| Zig | Comptime type check | N/A |
| Rust | Pattern matching | Pattern matching |

---

## sed (Regex Literals)

**Etymology:** Playful nod to Unix `sed` (stream editor). In Latin, `sed` means "but"—semantically unrelated, but the Unix association is so strong that using it for regex feels natural.

### Syntax

```faber
sed /pattern/flags
```

The `sed` keyword signals the tokenizer to switch to regex mode until the closing `/`.

### Basic Usage

```faber
fixum digits = sed /\d+/
fixum email = sed /[a-z]+@[a-z]+\.[a-z]+/i
fixum words = sed /\b\w+\b/g
```

### Flags

| Flag | Meaning |
|------|---------|
| `g` | Global (all matches) |
| `i` | Case insensitive |
| `m` | Multiline |
| `s` | Dotall (`.` matches newlines) |

### In Expressions

Regex literals work anywhere an expression is expected:

```faber
// Conditional
si text.congruet(sed /^\d{3}-\d{4}$/) {
    scribe "Valid phone"
}

// Method argument
fixum clean = text.muta(sed /\s+/g, " ")

// Variable
fixum pattern = sed /[A-Z][a-z]+/
fixum names = text.inveniOmnes(pattern)
```

### String Methods for Regex

| Faber | JS Equivalent | Description |
|-------|---------------|-------------|
| `congruet(sed)` | `test()` / `match()` | Test if pattern matches |
| `inveni(sed)` | `match()` | Find first match |
| `inveniOmnes(sed)` | `matchAll()` | Find all matches |
| `muta(sed, textus)` | `replace()` | Replace matches |
| `scinde(sed)` | `split()` | Split by pattern |

### Design Notes

- Keyword prefix (`sed /`) disambiguates from division operator
- Space between `sed` and `/` required (consistent with `novum`, `sparge`, etc.)
- Escaping follows target language rules (backslash escapes)
- Compile-time syntax validation where possible

### Target Mappings

| Target | Compilation |
|--------|-------------|
| TypeScript | `/pattern/flags` (native RegExp) |
| Python | `re.compile(r"pattern", flags)` |
| Zig | Library-dependent (not in stdlib) |
| Rust | `Regex::new(r"pattern")` |
| C++ | `std::regex("pattern", flags)` |

### Open Questions

1. **Zig support**: Require external library or mark as unsupported?
2. **Named capture groups**: `sed /(?<name>\w+)/` syntax?
3. **Interpolation**: Allow variables in pattern? `sed /${prefix}\d+/`

---

## vel (Nullish Coalescing)

**Etymology:** Latin *vel* — "or" (in the sense of alternatives).

Provides a fallback value when the left operand is `nihil`. Currently implemented in destructuring; extending to general expressions.

### Syntax

```faber
expression vel fallback
```

### Nullish, Not Falsy

`vel` triggers **only** on `nihil`, not on falsy values:

```faber
0 vel 5           // 0 (not nihil)
"" vel "default"  // "" (not nihil)
falsum vel verum  // falsum (not nihil)
nihil vel 5       // 5
```

This is more precise than logical OR and prevents surprising behavior with valid zero/empty/false values.

### Basic Usage

```faber
fixum name = user.name vel "Anonymous"
fixum count = getData() vel 0
fixum config = loadConfig() vel defaultConfig
```

### Chaining

Left-to-right evaluation, first non-nihil wins:

```faber
fixum value = primary vel secondary vel tertiary vel default
```

### In Destructuring (existing)

```faber
ex config fixum { timeout vel 5000, retries vel 3 }
```

### With Optional Chaining (future)

Pairs naturally with safe property access:

```faber
fixum city = user?.address?.city vel "Unknown"
```

### Comparison with Logical Or

| Expression | `vel` (nullish) | `aut` (logical) |
|------------|-----------------|-----------------|
| `0 vel 5` | `0` | `5` |
| `"" vel "x"` | `""` | `"x"` |
| `falsum vel verum` | `falsum` | `verum` |
| `nihil vel 5` | `5` | `5` |

Use `vel` for defaults. Use `aut` for boolean logic.

### Target Mappings

| Target | Compilation |
|--------|-------------|
| TypeScript | `??` |
| Python | `if x is None else` pattern |
| Zig | `orelse` |
| Rust | `.unwrap_or()` / `?` |
| C++ | `value_or()` / ternary |

---

## Future Operators

Candidates for future implementation:

| Operator | Purpose | Notes |
|----------|---------|-------|
| `?.` | Optional chaining | Safe property access |
