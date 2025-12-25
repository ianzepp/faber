# Types Design

## Implementation Status

### Implemented

- `genus` declaration with fields and methods
- Field defaults using `:` syntax
- Field visibility (`privatus` for private, public by default)
- Static members (`generis`)
- Type parameters (`genus capsa<T>`)
- `pactum` declaration
- `implet` for interface implementation
- Methods in genus with `ego` self-reference
- `novum Type` instantiation without parentheses
- `novum Type { ... }` inline field overrides (literal object only)
- `creo()` post-initialization hook (no args, `ego` pre-merged)
- Auto-merge: constructor merges field defaults + `{ ... }`
- `novum Type de dato` construction from existing variable (ablative — "from this source")

### Not Yet Implemented

- Value semantics (copy on assign)

---

## Core Concepts

| Faber | Closest analog | Description |
|-------|----------------|-------------|
| `genus` | struct | Data type with fields and methods |
| `pactum` | interface/protocol | Contract of methods only (no properties) |

## What Doesn't Exist

- No classes
- No `extends` / inheritance
- No property requirements in interfaces (methods only)

---

## genus

**Etymology:** "birth, origin, kind, type" — a category of thing.

### Syntax

```
genus persona {
    textus nomen
    numerus aetas

    functio saluta() -> textus {
        redde "Salve, " + ego.nomen
    }
}
```

### Default Field Values

Fields can have defaults in their declaration using `:` (colon):

```
genus persona {
    textus nomen: "Incognitus"
    numerus aetas: 0
    bivalens activus: verum
}

fixum p = novum persona  // uses all defaults
fixum q = novum persona { nomen: "Marcus" }  // override nomen only
```

**Why `:` not `=`?**

The colon means "has the value of" — a declarative specification. The equals sign means "assign this value" — an imperative action.

| Syntax | Meaning | Context |
|--------|---------|---------|
| `:` | "has value" / "defaults to" | Field defaults, object literals, construction |
| `=` | "assign value" | Variable binding, reassignment, method bodies |

This aligns field defaults with object literal syntax (`{ nomen: "Marcus" }`) and construction overrides (`de overrides`), creating a consistent "property specification" form throughout the language.

### Naming

- Type names are **lowercase**: `genus persona`, `genus lista<T>`
- Follows Latin convention (common nouns, not proper nouns)

### Field Visibility

- Fields are **public by default** (struct semantics)
- Use `privatus` for private fields:

```
genus persona {
    textus nomen              // public (default)
    privatus numerus aetas    // private
}
```

### Method Visibility

- Methods are **public by default** (struct semantics)
- Use `privatus` for private methods:

```
genus persona {
    functio saluta() -> textus { ... }      // public (default)
    privatus functio auxilium() { ... }     // private helper
}
```

### Self Reference

- **`ego`** refers to the current instance
- Latin "I" / "self"

```
functio celebraNatalem() {
    ego.aetas = ego.aetas + 1
}
```

### Type-Level Members (Static)

Use **`generis`** ("of the genus") for members that belong to the type, not instances:

```
genus colores {
    generis fixum ruber = "#FF0000"
    generis fixum viridis = "#00FF00"
    generis fixum caeruleus = "#0000FF"
}

genus math {
    generis fixum PI = 3.14159
    generis fixum E = 2.71828

    generis functio maximus(numerus a, numerus b) -> numerus {
        si a > b { redde a }
        redde b
    }
}

// Access via type name
scribe colores.ruber      // "#FF0000"
scribe math.PI            // 3.14159
fixum m = math.maximus(5, 3)  // 5
```

**Naming rationale:** `generis` is the genitive of `genus` — literally "of the type."

### Derived Values (Use Methods)

Faber intentionally omits computed properties (getters). Use methods instead:

```
genus rectangulum {
    numerus latitudo: 1
    numerus altitudo: 1

    functio area() -> numerus {
        redde ego.latitudo * ego.altitudo
    }
}

fixum r = novum rectangulum { latitudo: 10, altitudo: 5 }
scribe r.area()  // 50
```

**Why no getters?**

1. **Getters become traps** — start simple, grow complex, but API is locked to property access
2. **Methods are honest** — `r.area()` clearly indicates computation
3. **Symmetry** — no setters either (use `nexum` for reactivity, methods for validation)
4. **Simplicity** — one way to do derived values, not two

**For reactive state**, use `nexum` which auto-generates getters/setters with invalidation hooks.

### Constructor

The compiler automatically merges field defaults with `{ ... }` overrides. The optional `creo` hook runs afterward for validation or derived fields — it takes no arguments because `ego` already has the merged values:

```
genus persona {
    textus nomen: "Incognitus"
    numerus aetas: 0

    functio creo() {
        // ego.nomen and ego.aetas are already set
        si ego.aetas < 0 {
            ego.aetas = 0
        }
    }
}
```

**Initialization order:**
1. Field defaults applied
2. `{ ... }` or `de props` overrides merged
3. `creo()` runs (if defined)

Most types won't need `creo` at all — it's only for invariants, clamping, or computing derived state.

### Instantiation

Use `novum` with optional `{ .. }` for field values:

```
// With defaults only
fixum p = novum persona

// Override specific fields
fixum q = novum persona { nomen: "Marcus", aetas: 30 }

// Override specific fields from another source
fixum props = getPersonaProps()
fixum q = novum persona de props
```

The compiler merges defaults with overrides, then calls `creo()` if defined.

### Generics

```
genus capsa<T> {
    T valor

    functio accipe() -> T {
        redde ego.valor
    }
}

fixum c = novum capsa<numerus> { valor: 42 }
```

---

## pactum

**Etymology:** "agreement, contract, pact" — a promise of behavior.

### Syntax

```
pactum iterabilis<T> {
    functio sequens() -> T?
    functio habet() -> bivalens
}
```

### Constraints

- **Methods only** — no property requirements
- Defines what a type can do, not what it has

### Implementation

Use `implet` to declare that a genus fulfills a pactum:

```
genus cursorem<T> implet iterabilis<T> {
    numerus index
    lista<T> data

    functio sequens() -> T? {
        si ego.index >= ego.data.longitudo() {
            redde nihil
        }
        fixum valor = ego.data[ego.index]
        ego.index = ego.index + 1
        redde valor
    }

    functio habet() -> bivalens {
        redde ego.index < ego.data.longitudo()
    }
}
```

Multiple pactum:

```
genus foo implet bar, baz {
    // must implement methods from both bar and baz
}
```

### Index Signatures (`aperit`)

Use **`aperit`** ("opens to") to declare that a genus or pactum accepts arbitrary string keys:

```
// Just aperit — any string key returns textus
genus config aperit textus { }

// With implet (order: genus → implet → aperit)
genus persona implet Nominabilis aperit textus {
    textus id
    textus nomen
}

// On pactum
pactum DynamicRecord aperit textus { }
```

**Etymology:** `aperire` — "to open, uncover" — the type opens to any key.

**Constraints:**
- Order is fixed: `implet` before `aperit`
- Multiple interfaces use comma: `implet A, B aperit T`
- Only one `aperit` per type (same as TypeScript)

**TypeScript output:**
```typescript
class config {
    [key: string]: string;
}

interface DynamicRecord {
    [key: string]: string;
}
```

**Note:** This is a TypeScript-specific feature. Other targets (Zig, Rust, C++) don't have equivalent semantics — use `tabula<textus, T>` for dynamic key/value storage on those targets.

### Tuple Types (`series`)

Use **`series<A, B, ...>`** for fixed-length, heterogeneous collections (tuples):

```
typus Point = series<numerus, numerus>
fixum p: Point = [10, 20]

typus Record = series<numerus, textus, bivalens>
fixum r: Record = [1, "foo", verum]

// Inline type annotation
fixum series<textus, numerus> pair = ["answer", 42]
```

**Etymology:** `series` — "row, sequence, chain" — a fixed sequence of elements.

**Target mappings:**

| Target | `series<A, B>` |
|--------|----------------|
| TypeScript | `[A, B]` |
| Python | `tuple[A, B]` |
| Rust | `(A, B)` |
| C++ | `std::tuple<A, B>` |
| Zig | anonymous struct |

**Values:** Use bracket literals `[a, b]`. The type annotation determines array vs tuple.

### typeof (`typus` on RHS)

Use **`typus`** on the right-hand side of a type alias to extract a type from a value:

```
fixum config = { port: 3000, host: "localhost" }
typus Config = typus config

// Config is now the type { port: numerus, host: textus }
```

**Etymology:** Same keyword as type alias declaration — context disambiguates (LHS = declaration, RHS = extraction).

**Target mappings:**

| Target | `typus x` |
|--------|-----------|
| TypeScript | `typeof x` |
| C++ | `decltype(x)` |
| Zig | `@TypeOf(x)` |

### Never Type (`numquam`)

Use **`numquam`** as the return type for functions that never return:

```
functio moritur() -> numquam {
    iace novum Error { message: "fatal" }
}

functio infinitus() -> numquam {
    dum verum { }
}
```

**Etymology:** `numquam` — "never" — the function never completes normally.

**Distinct from `vacuum`:**
- `vacuum` — function returns, but with no value (void)
- `numquam` — function never returns (throws, loops forever, exits)

**Target mappings:**

| Target | `numquam` |
|--------|-----------|
| TypeScript | `never` |
| Rust | `!` |
| Zig | `noreturn` |
| Python | `NoReturn` |
| C++ | `[[noreturn]]` attribute |

---

## Value Semantics (Future)

Goal: genus instances behave like value types (copy on assign, no accidental aliasing) so they map cleanly to Zig/Rust structs. **Current reality:**

- TypeScript backend emits `class` declarations (reference semantics)
- Assignments copy references (`b = a` points to the same object)
- Constructors already use `this`, so switching to structs requires coordinated runtime changes

```
// Today
fixum a = novum punctum { x: 1, y: 2 }
fixum b = a        // b references the same object
b.x = 10           // a.x is ALSO 10 (reference semantics)
```

**Open questions:**
- Should we introduce explicit reference wrappers (`arcus<persona>`, `&persona`) once value semantics arrive?
- How do we express borrowing/ownership hints in Latin syntax?

---

## Primitive Types

### Numeric Types

| Type | Meaning | TS | Rust | Zig |
|------|---------|----|----- |-----|
| `numerus` | Integer | `number` | `i64` | `i64` |
| `fractus` | Floating point | `number` | `f64` | `f64` |
| `decimus` | Arbitrary precision decimal | `Decimal` | `BigDecimal` | — |
| `magnus` | Arbitrary precision integer | `bigint` | `BigInt` | — |

```
fixum numerus count = 42
fixum fractus ratio = 3.14159
fixum decimus price = 19.99d
fixum magnus huge = magnus("99999999999999999999")
```

**Etymology:**
- `numerus` — "number, count"
- `fractus` — "broken, fractional" (root of "fraction")
- `decimus` — "tenth, decimal"
- `magnus` — "great, large" (arbitrary size)

### Other Primitives

| Type | Meaning | Notes |
|------|---------|-------|
| `textus` | String/text | UTF-8 encoded |
| `bivalens` | Boolean | `verum`/`falsum` |
| `nihil` | Null | Absence of value |
| `vacuum` | Void | Function returns nothing |
| `numquam` | Never | Function never returns |
| `octeti` | Bytes | Raw byte data (Uint8Array) |
| `objectum` | Object | Any non-primitive value |
| `ignotus` | Unknown | Must narrow before use |

**Etymology:**
- `textus` — "woven, texture" (text as woven words)
- `bivalens` — "two-valued" (true/false)
- `nihil` — "nothing"
- `vacuum` — "empty, void"
- `numquam` — "never" (function doesn't return)
- `octeti` — "groups of eight" (bytes)
- `objectum` — "something thrown before" (root of "object")
- `ignotus` — "unknown, unrecognized" (in- + gnoscere, "not known")

### Object Type

The `objectum` type represents any non-primitive value (maps to TypeScript's `object`).
Object literals automatically infer as `objectum`:

```
functio getUser() -> objectum {
    redde { name: "Marcus", age: 30 }
}

fixum user = getUser()
scribe user.name
```

Use `objectum` when a function returns an anonymous object structure.
For known shapes, prefer defining a `genus` instead.

### Unknown Type

The `ignotus` type represents a value whose type is not known at compile time. Unlike permissive "any" types in other languages, `ignotus` requires explicit narrowing before use.

```
fixum data: ignotus = getExternalData()

// Error: cannot use ignotus directly
// scribe data.length

// Must narrow first with type check
si data est textus {
    scribe data.longitudo()  // OK, data is textus here
}

// Or cast explicitly (unsafe)
fixum name = data ut textus
scribe name.longitudo()
```

**Why no `any` type?**

Faber deliberately omits a permissive "any" type. The `ignotus` type requires you to either:
1. Narrow with `est` (safe, compiler-verified)
2. Cast with `ut` (explicit acknowledgment of risk)

This design:
- Maps cleanly to strict targets (Zig, Rust) where "any" doesn't exist
- Makes type uncertainty visible and intentional
- Encourages proper type handling rather than escape hatches

**Target mappings:**

| Target | `ignotus` |
|--------|-----------|
| TypeScript | `unknown` |
| Python | `Any` (but with runtime checks encouraged) |
| Zig | `anytype` / `*anyopaque` |
| Rust | Generics or `Box<dyn Any>` |

---

## Type Annotations

### In Variable Declarations

```
fixum persona p = novum persona { ... }
varia lista<textus> items = []
```

### Function Parameters and Returns

```
functio processare(persona p) -> textus {
    redde p.nomen
}
```

### Optional Types

```
textus? nomen            // may be nihil
functio inveni() -> persona?
```

### Ownership Annotations (Rust/Zig Targets)

For systems targets, Latin prepositions annotate borrowing semantics:

| Preposition | Meaning | Rust | Zig |
|-------------|---------|------|-----|
| (none) | Owned | `T` | allocator-managed |
| `de` | Borrowed, read-only | `&T` | `*const T`, `[]const u8` |
| `in` | Mutable borrow | `&mut T` | `*T` |

```
// No preposition = owned, will be moved/consumed
functio consume(textus data) -> Result

// "de" (from/concerning) = borrowed, read-only
functio read(de textus source) -> numerus

// "in" (into) = mutable borrow, will be modified
functio append(in textus target, textus suffix)
```

These keywords are **target-specific** — valid only for Rust/Zig projects. TypeScript/Python targets reject them as syntax errors. See `codegen/rust.md` and `codegen/zig.md` for details.

---

## Implementation Notes

### TypeScript Target

```typescript
// genus persona {
//     textus nomen: "Incognitus"
//     numerus aetas: 0
//     functio creo() { if (this.aetas < 0) this.aetas = 0 }
// }
class persona {
    nomen: string = "Incognitus";
    aetas: number = 0;

    constructor(overrides: { nomen?: string, aetas?: number } = {}) {
        // Merge overrides into defaults
        if (overrides.nomen !== undefined) this.nomen = overrides.nomen;
        if (overrides.aetas !== undefined) this.aetas = overrides.aetas;
        // Then call creo() if defined
        this.creo();
    }

    private creo() {
        if (this.aetas < 0) this.aetas = 0;
    }
}

// novum persona { nomen: "Marcus" }
new persona({ nomen: "Marcus" })
```

### Zig Target

```zig
// genus persona {
//     textus nomen: "Incognitus"
//     numerus aetas: 0
//     functio creo() { ... }
// }
const Persona = struct {
    nomen: []const u8 = "Incognitus",
    aetas: i64 = 0,

    pub fn init(overrides: anytype) Persona {
        var self = Persona{};
        // Merge overrides
        if (@hasField(@TypeOf(overrides), "nomen")) self.nomen = overrides.nomen;
        if (@hasField(@TypeOf(overrides), "aetas")) self.aetas = overrides.aetas;
        // Then call creo() if defined
        self.creo();
        return self;
    }

    fn creo(self: *Persona) void {
        if (self.aetas < 0) self.aetas = 0;
    }
};
```

### Pactum → Interface/Trait

- TypeScript: `interface`
- Zig: Comptime duck typing or vtable pattern

---

## Open Questions

1. **`novum Type ex dato` syntax** — We want to pass previously declared objects (e.g., `novum civis ex claudia_datum`) without retyping the literal. Need to settle the Latin case/preposition story and how it lowers to the merge step.
2. **Value/reference interop** — Once value semantics arrive, how do we opt into shared references? (`arcus<persona>`? `&persona`?) What does that look like in JS/TS output?

---

## Destructuring

Object destructuring only, single level. No array destructuring.

### Basic

```
fixum { nomen, aetas } = p
```

### Renaming

Both Latin and symbolic syntax supported:

```
fixum { nomen ut n } = p      // Latin: "nomen as n"
fixum { nomen: n } = p        // symbolic

scribe n  // "Marcus"
```

### Default Values

Both Latin and symbolic syntax supported:

```
fixum { aetas vel 0 } = p     // Latin: "aetas or 0"
fixum { aetas ?? 0 } = p      // symbolic (nil-coalescing)
```

### Combined

```
fixum { nomen ut n, aetas vel 0 } = p
```

### Partial

Grabbing only some fields is valid:

```
fixum { nomen } = p  // ignore aetas
```

### Mutable Bindings

```
varia { nomen, aetas } = p
```
