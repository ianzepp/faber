# Structurae

Faber provides two fundamental building blocks for defining data structures: `genus` for concrete data types with fields and methods, and `pactum` for behavioral contracts that define what a type can do. This document explains how to declare, instantiate, and work with both.

The Latin terminology reflects the conceptual distinction: a `genus` (meaning "birth, origin, kind") describes what something _is_, while a `pactum` (meaning "agreement, contract") describes what something _promises to do_.

---

## genus (Data Types)

A `genus` declaration creates a data type with fields and optional methods. Unlike class-based languages that emphasize inheritance hierarchies, Faber's `genus` follows struct semantics: fields are public by default, and composition is preferred over inheritance.

### Declaration

The basic form declares a type name followed by its fields inside braces:

```fab
genus Punctum {
    numerus x
    numerus y
}
```

Each field declaration specifies the type first, then the field name. This follows Faber's type-first convention, making the shape of data immediately visible.

A `genus` can contain any number of fields:

```fab
genus Persona {
    textus nomen
    numerus aetas
    bivalens activus
}
```

Type names in Faber are conventionally lowercase, following Latin's case conventions where common nouns are not capitalized. The parser is case-insensitive, but the canonical style uses lowercase: `genus persona`, not `genus Persona`.

### Field Defaults

Fields can specify default values using the colon (`:`) syntax:

```fab
genus Persona {
    textus nomen: "Incognitus"
    numerus aetas: 0
    bivalens activus: verum
}
```

When a default is provided, the field becomes optional during instantiation. Fields without defaults are required.

The colon syntax deserves explanation. Faber distinguishes between two operations:

| Syntax | Meaning                     | Context                                       |
| ------ | --------------------------- | --------------------------------------------- |
| `:`    | "has value" / "defaults to" | Field defaults, object literals, construction |
| `←`    | "assign value"              | Variable binding, reassignment, method bodies |

The colon represents a _declarative specification_: this field has this value by nature of its definition. The left arrow represents an _imperative action_: assign this value to that location.

This distinction creates consistency across the language. Object literals use colons (`{ nomen: "Marcus" }`), construction overrides use colons, and field defaults use colons. All three are specifying property values, not performing assignment.

### Methods

A `genus` can include methods alongside its fields. Methods are declared with the `functio` keyword:

```fab
genus Rectangle {
    numerus width: 1
    numerus height: 1

    functio area() → numerus {
        redde ego.width * ego.height
    }

    functio perimeter() → numerus {
        redde 2 * (ego.width + ego.height)
    }

    functio isSquare() → bivalens {
        redde ego.width == ego.height
    }
}
```

Methods can return values, modify state, or both:

```fab
genus Counter {
    numerus count: 0

    functio increment() {
        ego.count ← ego.count + 1
    }

    functio getValue() → numerus {
        redde ego.count
    }
}
```

Methods are public by default, matching the struct semantics of `genus`. The grammar reserves visibility-style annotations, but the active `radix-rs` implementation does not yet carry a stable member-visibility contract through parsing, lowering, and code generation, so this page sticks to the public-by-default rule.

### Self-Reference with ego

Within methods, `ego` refers to the current instance. The word is Latin for "I" or "self", making the self-reference explicit in every usage.

```fab
functio celebraNatalem() {
    ego.aetas ← ego.aetas + 1
}
```

Unlike languages where `this` is implicit or optional, Faber requires explicit `ego` for all instance member access. This eliminates ambiguity between local variables and instance fields, and makes the flow of data through an object visible.

### Static Members with generis

Members that belong to the type itself rather than instances use the `generis` keyword. The word is the genitive form of `genus`, literally meaning "of the type":

```fab
genus Colores {
    generis textus ruber: "#FF0000"
    generis textus viridis: "#00FF00"
    generis textus caeruleus: "#0000FF"
}
```

Access static members through the type name:

```fab
scribe Colores.ruber      # "#FF0000"
```

Static `generis` fields are useful for constants and shared configuration. The broader `generis functio` surface still lags between older fixtures and the active `radix-rs` checker, so this page keeps the example to the code-backed field form.

### Member Visibility

Fields in a `genus` are public by default, following struct semantics where data is meant to be accessed directly. That public-by-default model is the only member-visibility behavior this page treats as stable today.

The broader visibility story is still unresolved across the repo:

- `EBNF.md` reserves annotation spellings such as `@ publica`, `@ protecta`, and `@ privata`
- older bootstrap-era fixtures still mention legacy `@ publicum` / `@ protectum` / `@ privatum` forms
- the active `radix-rs` parser accepts method-position visibility annotations more readily than field-position ones
- `radix-rs` lowering and code generation do not currently carry a dedicated member-visibility model

Until that contract is reconciled, prefer plain public members in examples and treat visibility annotations as implementation work in progress rather than tutorial-ready surface area.

### Abstract Types

The `abstractus` modifier creates a type that cannot be instantiated directly. Abstract types define structure and behavior that subtypes must complete:

```fab
abstractus genus Figura {
    @ abstracta
    functio area() → numerus
}
```

Methods marked with `@ abstracta` have no body; subtypes must provide the implementation.

### Generics

A `genus` can be parameterized with type variables:

```fab
genus Capsa<T> {
    T valor

    functio accipe() → T {
        redde ego.valor
    }
}

fixum c = { valor: 42 } novum Capsa<numerus>
scribe c.accipe()  # 42
```

Multiple type parameters are comma-separated: `genus Pair<K, V> { ... }`.

---

## pactum (Interfaces)

A `pactum` defines a contract: a set of method signatures that a type promises to implement. The word means "agreement" or "pact", reflecting its role as a behavioral promise rather than a structural definition.

### Declaration

A `pactum` declares method signatures without implementations:

```fab
pactum Drawable {
    functio draw() → vacuum
}

pactum Iterabilis<T> {
    functio sequens() → si numerus
    functio habet() → bivalens
}
```

Unlike `genus`, a `pactum` cannot have fields or property requirements. It defines only what a type can _do_, not what it _has_. This constraint keeps interfaces focused on behavior. When a method returns an optional value, current grammar uses the `si` prefix form rather than a trailing `?`.

### Implementation with implet

A `genus` fulfills a `pactum` using the `implet` keyword (Latin "fulfills"):

```fab
genus Circle implet Drawable {
    numerus radius: 10

    functio draw() {
        scribe scriptum("Drawing circle with radius §", ego.radius)
    }
}

genus Square implet Drawable {
    numerus side: 5

    functio draw() {
        scribe scriptum("Drawing square with side §", ego.side)
    }
}
```

The implementing type must provide concrete implementations for all methods declared in the `pactum`. The compiler verifies this at compile time.

A type can implement multiple interfaces:

```fab
genus Document implet Readable, Writable, Printable {
    # must implement methods from all three pactum
}
```

---

## Instantiation

### Creating Instances with novum

The `novum` keyword participates in postfix construction. The word is Latin for "new", and the current `radix-rs` surface uses it after the source literal:

```fab
fixum p = { x: 10, y: 20 } novum Punctum
```

Field values are provided in an object literal before the type name. Required fields (those without defaults) must be specified:

```fab
genus Persona {
    textus nomen           # required - no default
    numerus aetas: 0       # optional - has default
}

# nomen is required, aetas is optional
fixum marcus = { nomen: "Marcus" } novum Persona
scribe marcus.aetas  # 0 (default)

# Override defaults by providing values
fixum julia = { nomen: "Julia", aetas: 25 } novum Persona
```

When all fields have defaults, the literal can be omitted entirely:

```fab
genus Counter {
    numerus count: 0
}

varia counter = {} novum Counter
```

### Construction from Existing Values

When you already have source values, build the object literal explicitly and then apply `novum`:

```fab
fixum props = getPersonaProps()
fixum p = {
    nomen: props.nomen,
    aetas: props.aetas
} novum Persona
```

This keeps construction explicit and uses the same field-checking rules as any other struct literal.

### The creo() Hook

The optional `creo()` method runs after construction is complete. Use it for validation, clamping values, or computing derived state:

```fab
genus BoundedValue {
    numerus value: 0

    functio creo() {
        si ego.value < 0 {
            ego.value ← 0
        }
        si ego.value > 100 {
            ego.value ← 100
        }
    }
}
```

The current `radix-rs` implementation treats `creo()` as a post-construction hook. The initialization sequence is:

1. Field defaults are applied
2. Literal overrides are merged
3. `creo()` runs if defined

By the time `creo()` executes, `ego` already has all its field values. The method takes no parameters because everything it needs is already on the instance.

A practical use is computing derived fields:

```fab
genus Circle {
    numerus radius: 1
    numerus diameter: 0
    numerus area: 0

    functio creo() {
        ego.diameter ← ego.radius * 2
        ego.area ← 3.14159 * ego.radius * ego.radius
    }
}

fixum c = { radius: 5 } novum Circle
scribe c.diameter  # 10
scribe c.area      # 78.54
```

Most types will not need `creo()`. The declarative field defaults handle the common case. Reserve `creo()` for invariants, validation, or derived initialization that cannot be expressed as simple defaults.

---

## Design Philosophy

Faber's type system reflects several deliberate choices:

**Composition over inheritance.** There is no `extends` keyword. Types compose behavior through `implet` and embed other types as fields. This avoids the fragility of deep inheritance hierarchies.

**Methods over getters.** Faber omits computed properties (getters). Derived values use explicit methods: `r.area()` rather than `r.area`. The parentheses honestly communicate that computation is happening. Getters that start simple often grow complex, but their API is locked to property syntax.

**Struct semantics by default.** Fields are public by default in the current docs and examples. This transparency suits data-oriented design where types are containers of state rather than encapsulated black boxes.

**No classes, no constructors.** The `genus` keyword names a type of thing, not a blueprint for objects. Construction happens through postfix `novum` with declarative field specification, not imperative constructor logic.

These choices produce code that is explicit about data flow and honest about computation. The Roman craftsman built things to last; Faber aims for code that remains comprehensible as it evolves.

---

## Stdlib Annotations

Faber's standard library uses specialized annotations to define how Latin-named methods map to target language implementations. These annotations enable the compiler to generate appropriate code for each target (TypeScript, Python, Zig, etc.) from a single source definition.

### @ innatum (Native Type Mapping)

The `@ innatum` annotation (Latin "inborn, innate") declares how a `genus` maps to native types in each target language:

```fab
@ innatum ts "Array", py "list", zig "Lista", rs "Vec", cpp "std::vector"
genus lista<T> { }
```

This tells the compiler that `lista<T>` should compile to `Array` in TypeScript, `list` in Python, and so on. The native type receives the methods defined on the genus.

### @ subsidia (External Implementation)

The `@ subsidia` annotation (Latin "support, aid") specifies external implementation files for targets where inline code generation is insufficient:

```fab
@ innatum ts "Array", py "list", zig "Lista"
@ subsidia zig "subsidia/zig/lista.zig"
genus lista<T> { }
```

When the compiler encounters a method without an inline `@ verte` for a target, it falls back to the subsidia file. This is useful for targets like Zig where allocator threading and memory management require substantial wrapper code.

### @ radix (Morphology Declaration)

The `@ radix` annotation (Latin "root, stem") declares the morphological stem and valid verb forms for a method:

```fab
@ radix append, imperativus, perfectum
@ externa
functio appende(T elem) → vacuum

@ externa
functio addita(T elem) → lista<T>
```

The first identifier is the verb stem (`append-`), followed by valid conjugation forms. In stdlib declarations the receiver is implicit: these signatures live inside the `lista` genus rather than spelling `ego` as a normal parameter.

| Form                  | Ending                | Semantics                       |
| --------------------- | --------------------- | ------------------------------- |
| `imperativus`         | `-a`, `-e`, `-i`      | Mutates in place, synchronous   |
| `perfectum`           | `-ata`, `-ita`, `-ta` | Returns new value, synchronous  |
| `futurum_indicativum` | `-abit`, `-ebit`      | Mutates in place, asynchronous  |
| `futurum_activum`     | `-atura`, `-itura`    | Returns new value, asynchronous |
| `generator`           | `-ans`, `-ens`        | Yields values (streaming)       |

The compiler validates that called method names match declared forms. Calling `items.appendatura(elem)` would error if only `imperativus, perfectum` are declared.

### @ verte (Codegen Transform)

The `@ verte` annotation (Latin "turn, transform") defines how a method call transforms to target code. Two forms are supported:

**Simple method rename:**

```fab
@ verte ts "push"
@ verte py "append"
@ verte rs "push"
@ externa
functio appende(T elem) → vacuum
```

This compiles `items.appende(x)` to `items.push(x)` in TypeScript and `items.append(x)` in Python.

**Template with placeholders:**

```fab
@ verte ts (ego, elem) → "[...§0, §1]"
@ verte py (ego, elem) → "[*§0, §1]"
@ verte zig (ego, elem, alloc) → "§0.addita(§2, §1)"
@ externa
functio addita(T elem) → lista<T>
```

The `§` placeholders are filled positionally with the parameter values. `§0` refers to the receiver, `§1` to the first explicit argument, and so on. For Zig, the allocator parameter comes last when needed.

Each `@ verte` specifies exactly one target. Use multiple annotations for multiple targets:

```fab
@ radix append, imperativus, perfectum
@ verte ts "push"
@ verte py "append"
@ verte rs "push"
@ verte cpp "push_back"
@ verte zig (ego, elem, alloc) → "§0.appende(§2, §1)"
@ externa
functio appende(T elem) → vacuum
```

### Design Philosophy

These annotations serve a specific purpose: defining the standard library in Faber source rather than scattered target-specific registries. User code typically does not need these annotations; they exist for stdlib authors and advanced library developers who want to provide optimized implementations across multiple targets.

The morphology system (`@ radix`) reflects Faber's Latin-first thesis: verb conjugations encode semantic information that modern languages express through ad-hoc naming conventions. Rather than memorizing `sort`/`sorted`, `reverse`/`reversed`, users learn that imperative forms mutate and participle forms return new values.
