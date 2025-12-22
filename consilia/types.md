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
- Methods in genus (without `ego`)

### Not Yet Implemented

- `ego` (self-reference in methods)
- `novum Type` without parentheses
- `novum Type cum { ... }` field override syntax
- `creo` constructor function
- Value semantics (copy on assign)
- Computed properties

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

### Constructor

The `creo` function receives a key/value object and initializes fields:

```
genus persona {
    textus nomen
    numerus aetas

    functio creo(valores) {
        ego.nomen = valores.nomen
        ego.aetas = valores.aetas ?? 0
    }
}
```

### Instantiation

Use `novum` with optional `cum` for field values:

```
// With defaults (calls creo with empty object)
fixum p = novum persona

// With values
fixum p = novum persona cum { nomen: "Marcus", aetas: 30 }

// The cum object is passed to creo()
```

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

## Value Semantics

Genus instances are **value types** by default:

- Assigned by copy, not reference
- Matches Rust `struct` and Zig `struct` semantics
- Enables future compilation to both targets

```
fixum a = novum punctum cum { x: 1, y: 2 }
fixum b = a        // b is a copy
b.x = 10           // a.x is still 1
```

**Open question:** Do we need explicit reference types? Options:
- `&persona` reference syntax
- `arcus<persona>` wrapper type (like Rust's `Rc`/`Arc`)
- Defer until needed

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

---

## Implementation Notes

### TypeScript Target

```typescript
// genus persona { textus nomen, numerus aetas }
class persona {
    nomen: string;
    aetas: number;

    constructor(valores: { nomen?: string, aetas?: number }) {
        this.nomen = valores.nomen ?? "";
        this.aetas = valores.aetas ?? 0;
    }
}

// novum persona cum { nomen: "Marcus" }
new persona({ nomen: "Marcus" })
```

### Zig Target

```zig
// genus persona { textus nomen, numerus aetas }
const Persona = struct {
    nomen: []const u8,
    aetas: i64,

    pub fn init(valores: anytype) Persona {
        return .{
            .nomen = valores.nomen orelse "",
            .aetas = valores.aetas orelse 0,
        };
    }
};
```

### Pactum → Interface/Trait

- TypeScript: `interface`
- Zig: Comptime duck typing or vtable pattern

---

## Open Questions

1. **Computed properties** — Getter/setter syntax? Deferred.
   ```
   genus rectangulum {
       numerus latus
       numerus altitudo

       // computed?
       numerus area => ego.latus * ego.altitudo
   }
   ```

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
