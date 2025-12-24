# Types Design

## Implementation Status

### Implemented

- `genus` declaration with fields and methods
- Field defaults using `:` syntax
- Field visibility (`publicus`)
- Static members (`generis`)
- Type parameters (`genus capsa<T>`)
- `pactum` declaration
- `implet` for interface implementation
- Methods in genus with `ego` self-reference
- `novum Type` instantiation without parentheses
- `novum Type cum { ... }` inline field overrides (literal object only)
- `creo()` post-initialization hook (no args, `ego` pre-merged)
- Computed properties (`numerus area => ...`)
- Auto-merge: constructor merges field defaults + `cum { ... }` overrides

### Not Yet Implemented

- `novum Type ex dato` construction from existing variable (ablative — "from this source")
- Passing previously declared objects directly to `cum` (inline literal required today)
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
fixum q = novum persona cum { nomen: "Marcus" }  // override nomen only
```

**Why `:` not `=`?**

The colon means "has the value of" — a declarative specification. The equals sign means "assign this value" — an imperative action.

| Syntax | Meaning | Context |
|--------|---------|---------|
| `:` | "has value" / "defaults to" | Field defaults, object literals, construction |
| `=` | "assign value" | Variable binding, reassignment, method bodies |

This aligns field defaults with object literal syntax (`{ nomen: "Marcus" }`) and construction overrides (`cum { nomen: "Marcus" }`), creating a consistent "property specification" form throughout the language.

### Naming

- Type names are **lowercase**: `genus persona`, `genus lista<T>`
- Follows Latin convention (common nouns, not proper nouns)

### Field Visibility

- Fields are **private by default**
- Use `publicus` for public access:

```
genus persona {
    publicus textus nomen     // accessible from outside
    numerus aetas             // private
}
```

### Method Visibility

- Methods are **private by default**
- Use `publicus` for public methods:

```
genus persona {
    publicus functio saluta() -> textus { ... }
    functio auxilium() { ... }  // private helper
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

### Computed Properties

- Declare with `type name => expression`
- Expression runs in the instance context (`ego` is available)
- Honors `publicus`/`generis` modifiers (public getters vs. type-level constants)
- Currently read-only (no setter syntax yet)

```
genus rectangulum {
    numerus latitudo: 1
    numerus altitudo: 1
    publicus numerus area => ego.latitudo * ego.altitudo
}
```

### Constructor

The compiler automatically merges field defaults with `cum { ... }` overrides. The optional `creo` hook runs afterward for validation or derived fields — it takes no arguments because `ego` already has the merged values:

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
2. `cum { ... }` overrides merged
3. `creo()` runs (if defined)

Most types won't need `creo` at all — it's only for invariants, clamping, or computing derived state.

### Instantiation

Use `novum` with optional `cum` for field values:

```
// With defaults only
fixum p = novum persona

// Override specific fields
fixum q = novum persona cum { nomen: "Marcus", aetas: 30 }
```

The compiler merges defaults with overrides, then calls `creo()` if defined.

### Generics

```
genus capsa<T> {
    T valor

    publicus functio accipe() -> T {
        redde ego.valor
    }
}

fixum c = novum capsa<numerus> cum { valor: 42 }
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

    publicus functio sequens() -> T? {
        si ego.index >= ego.data.longitudo() {
            redde nihil
        }
        fixum valor = ego.data[ego.index]
        ego.index = ego.index + 1
        redde valor
    }

    publicus functio habet() -> bivalens {
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

---

## Value Semantics (Future)

Goal: genus instances behave like value types (copy on assign, no accidental aliasing) so they map cleanly to Zig/Rust structs. **Current reality:**

- TypeScript backend emits `class` declarations (reference semantics)
- Assignments copy references (`b = a` points to the same object)
- Computed properties/constructors already use `this`, so switching to structs requires coordinated runtime changes

```
// Today
fixum a = novum punctum cum { x: 1, y: 2 }
fixum b = a        // b references the same object
b.x = 10           // a.x is ALSO 10 (reference semantics)
```

**Open questions:**
- Should we introduce explicit reference wrappers (`arcus<persona>`, `&persona`) once value semantics arrive?
- How do we express borrowing/ownership hints in Latin syntax?

---

## Type Annotations

### In Variable Declarations

```
fixum persona p = novum persona cum { ... }
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

// novum persona cum { nomen: "Marcus" }
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
