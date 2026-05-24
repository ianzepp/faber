---
title: Structurae
section: docs
order: 13
---

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
| `=`    | "assign value"              | Variable binding, reassignment, method bodies |

The colon represents a _declarative specification_: this field has this value by nature of its definition. The equals sign represents an _imperative action_: assign this value to that location.

This distinction creates consistency across the language. Object literals use colons (`{ nomen: "Marcus" }`), construction overrides use colons, and field defaults use colons. All three are specifying property values, not performing assignment.

### Methods

A `genus` can include methods alongside its fields. Methods are declared with the `functio` keyword:

```fab
genus Rectangle {
    numerus width: 1
    numerus height: 1

    functio area() -> numerus {
        redde ego.width * ego.height
    }

    functio perimeter() -> numerus {
        redde 2 * (ego.width + ego.height)
    }

    functio isSquare() -> bivalens {
        redde ego.width == ego.height
    }
}
```

Methods can return values, modify state, or both:

```fab
genus Counter {
    numerus count: 0

    functio increment() {
        ego.count = ego.count + 1
    }

    functio getValue() -> numerus {
        redde ego.count
    }
}
```

Methods are public by default, matching the struct semantics of `genus`. Use the `@ privata` annotation for internal helper methods.

### Self-Reference with ego

Within methods, `ego` refers to the current instance. The word is Latin for "I" or "self", making the self-reference explicit in every usage.

```fab
functio celebraNatalem() {
    ego.aetas = ego.aetas + 1
}
```

Unlike languages where `this` is implicit or optional, Faber requires explicit `ego` for all instance member access. This eliminates ambiguity between local variables and instance fields, and makes the flow of data through an object visible.

### Static Members with generis

Members that belong to the type itself rather than instances use the `generis` keyword. The word is the genitive form of `genus`, literally meaning "of the type":

```fab
genus Colores {
    generis fixum ruber = "#FF0000"
    generis fixum viridis = "#00FF00"
    generis fixum caeruleus = "#0000FF"
}

genus Math {
    generis fixum PI = 3.14159
    generis fixum E = 2.71828

    generis functio maximus(numerus a, numerus b) -> numerus {
        si a > b { redde a }
        redde b
    }
}
```

Access static members through the type name:

```fab
scribe Colores.ruber      # "#FF0000"
scribe Math.PI            # 3.14159
fixum m = Math.maximus(5, 3)  # 5
```

Static members are useful for constants, utility functions, and factory methods that don't require instance state.

### Field Visibility

Fields in a `genus` are public by default, following struct semantics where data is meant to be accessed directly. For fields that should be encapsulated, use visibility annotations:

```fab
genus Persona {
    textus nomen              # public (default)

    @ privatum
    numerus internaAetas      # private - only accessible within this genus
}
```

The `@ privatum` annotation marks a field as accessible only within the `genus` that declares it. The `@ protectum` annotation allows access from subtypes as well.

### Method Visibility

Methods follow the same pattern:

```fab
genus Processor {
    functio process() -> textus {       # public (default)
        redde ego.auxilium()
    }

    @ privata
    functio auxilium() -> textus {      # private helper
        redde "internal work"
    }
}
```

Note the grammatical agreement: `privatum` for fields (neuter), `privata` for methods (feminine, agreeing with `functio`).

### Abstract Types

The `abstractus` modifier creates a type that cannot be instantiated directly. Abstract types define structure and behavior that subtypes must complete:

```fab
abstractus genus Figura {
    @ abstracta
    functio area() -> numerus
}
```

Methods marked with `@ abstracta` have no body; subtypes must provide the implementation.

### Generics

A `genus` can be parameterized with type variables:

```fab
genus Capsa<T> {
    T valor

    functio accipe() -> T {
        redde ego.valor
    }
}

fixum c = novum Capsa<numerus> { valor: 42 }
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
    functio draw() -> vacuum
}

pactum Iterabilis<T> {
    functio sequens() -> T?
    functio habet() -> bivalens
}
```

Unlike `genus`, a `pactum` cannot have fields or property requirements. It defines only what a type can _do_, not what it _has_. This constraint keeps interfaces focused on behavior.

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

The `novum` keyword creates a new instance of a `genus`. The word is Latin for "new":

```fab
fixum p = novum Punctum { x: 10, y: 20 }
```

Field values are provided in an object literal following the type name. Required fields (those without defaults) must be specified:

```fab
genus Persona {
    textus nomen           # required - no default
    numerus aetas: 0       # optional - has default
}

# nomen is required, aetas is optional
fixum marcus = novum Persona { nomen: "Marcus" }
scribe marcus.aetas  # 0 (default)

# Override defaults by providing values
fixum julia = novum Persona { nomen: "Julia", aetas: 25 }
```

When all fields have defaults, the literal can be omitted entirely:

```fab
genus Counter {
    numerus count: 0
}

varia counter = novum Counter  # no braces needed
```

### Construction from Variables

When constructing from an existing variable, use the `de` (from) preposition:

```fab
fixum props = getPersonaProps()
fixum p = novum Persona de props
```

This merges the source object's fields into the new instance, following the same rules as literal construction.

### The creo() Hook

The optional `creo()` method runs after construction is complete. Use it for validation, clamping values, or computing derived state:

```fab
genus BoundedValue {
    numerus value: 0

    functio creo() {
        si ego.value < 0 {
            ego.value = 0
        }
        si ego.value > 100 {
            ego.value = 100
        }
    }
}
```

The initialization sequence is:

1. Field defaults are applied
2. Literal overrides (or `de` source) are merged
3. `creo()` runs if defined

By the time `creo()` executes, `ego` already has all its field values. The method takes no parameters because everything it needs is already on the instance.

A practical use is computing derived fields:

```fab
genus Circle {
    numerus radius: 1
    numerus diameter: 0
    numerus area: 0

    functio creo() {
        ego.diameter = ego.radius * 2
        ego.area = 3.14159 * ego.radius * ego.radius
    }
}

fixum c = novum Circle { radius: 5 }
scribe c.diameter  # 10
scribe c.area      # 78.54
```

Most types will not need `creo()`. The declarative field defaults handle the common case. Reserve `creo()` for invariants, validation, or derived initialization that cannot be expressed as simple defaults.

---

## Design Philosophy

Faber's type system reflects several deliberate choices:

**Composition over inheritance.** There is no `extends` keyword. Types compose behavior through `implet` and embed other types as fields. This avoids the fragility of deep inheritance hierarchies.

**Methods over getters.** Faber omits computed properties (getters). Derived values use explicit methods: `r.area()` rather than `r.area`. The parentheses honestly communicate that computation is happening. Getters that start simple often grow complex, but their API is locked to property syntax.

**Struct semantics by default.** Fields are public unless explicitly marked private. This transparency suits data-oriented design where types are containers of state rather than encapsulated black boxes.

**No classes, no constructors.** The `genus` keyword names a type of thing, not a blueprint for objects. Construction happens through `novum` with declarative field specification, not imperative constructor logic.

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
@ radix filtr, imperativus, perfectum
functio filtra<T>(ego lista<T>, praedicatum: functio(T) fit bivalens) fit vacuum
functio filtrata<T>(ego lista<T>, praedicatum: functio(T) fit bivalens) fit lista<T>
```

The first identifier is the verb stem (`filtr-`), followed by valid conjugation forms:

| Form                  | Ending                | Semantics                       |
| --------------------- | --------------------- | ------------------------------- |
| `imperativus`         | `-a`, `-e`, `-i`      | Mutates in place, synchronous   |
| `perfectum`           | `-ata`, `-ita`, `-ta` | Returns new value, synchronous  |
| `futurum_indicativum` | `-abit`, `-ebit`      | Mutates in place, asynchronous  |
| `futurum_activum`     | `-atura`, `-itura`    | Returns new value, asynchronous |
| `generator`           | `-ans`, `-ens`        | Yields values (streaming)       |

The compiler validates that called method names match declared forms. Calling `items.filtratura(pred)` would error if only `imperativus, perfectum` are declared.

### @ verte (Codegen Transform)

The `@ verte` annotation (Latin "turn, transform") defines how a method call transforms to target code. Two forms are supported:

**Simple method rename:**

```fab
@ verte ts "push"
@ verte py "append"
functio appende<T>(ego lista<T>, elem: T) fit vacuum
```

This compiles `items.appende(x)` to `items.push(x)` in TypeScript and `items.append(x)` in Python.

**Template with placeholders:**

```fab
@ verte ts (ego, elem) -> "[...§, §]"
@ verte py (ego, elem) -> "[*§, §]"
@ verte zig (ego, elem, alloc) -> "§.addita(§, §)"
functio addita<T>(ego lista<T>, elem: T) fit lista<T>
```

The `§` placeholders are filled positionally with the parameter values. For Zig, the allocator parameter comes last when needed.

Each `@ verte` specifies exactly one target. Use multiple annotations for multiple targets:

```fab
@ radix append, imperativus, perfectum
@ verte ts "push"
@ verte py "append"
@ verte rs "push"
@ verte cpp "push_back"
@ verte zig (ego, elem, alloc) -> "§.appende(§, §)"
functio appende<T>(ego lista<T>, elem: T) fit vacuum
```

### Design Philosophy

These annotations serve a specific purpose: defining the standard library in Faber source rather than scattered target-specific registries. User code typically does not need these annotations; they exist for stdlib authors and advanced library developers who want to provide optimized implementations across multiple targets.

The morphology system (`@ radix`) reflects Faber's Latin-first thesis: verb conjugations encode semantic information that modern languages express through ad-hoc naming conventions. Rather than memorizing `sort`/`sorted`, `reverse`/`reversed`, users learn that imperative forms mutate and participle forms return new values.

