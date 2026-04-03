# Typi

Faber Romanus uses a type-first syntax that places type annotations before identifiers, reflecting the Latin pattern where adjectives describing nature precede the noun they modify. Where TypeScript writes `const name: string`, Faber writes `fixum textus nomen`. This ordering reads naturally in Latin: "a fixed text, name"---the type describes what kind of thing the identifier represents.

The type system prioritizes explicitness and clarity. Every type name derives from Latin, chosen not for historical purity but because Latin's morphological richness allows type names to carry meaning. When you see `fractus` instead of `float`, you encounter the Latin root of "fraction"---a broken number, a number with parts. The etymology teaches.

---

## Primitive Types

### textus (String)

From the Latin *texere*, "to weave." Text is woven words, threads of meaning combined into fabric. The metaphor is ancient: we still speak of "spinning a yarn" and the "thread" of an argument.

```fab
fixum textus greeting = "Salve, Munde"
fixum textus empty = ""
```

String literals use double or single quotes. Template literals use backticks with `${...}` interpolation, just as in JavaScript:

```fab
fixum name = "Marcus"
fixum message = `Hello, ${name}`
```

### numerus (Integer)

From the Latin *numerus*, "number, count." This is the discrete counting number---whole, indivisible. When you need integers, you need `numerus`.

```fab
fixum numerus count = 42
fixum numerus negative = -100
fixum numerus hex = 0xFF
```

Numeric literals support decimal, hexadecimal (with `0x` prefix), and arbitrary-precision integers (with `n` suffix for big integers).

### fractus (Floating-Point)

From the Latin *fractus*, "broken." The past participle of *frangere*, "to break." A fractional number is a broken number---one that has been divided into parts. This is the etymological root of the English word "fraction."

Use `fractus` when you need decimal precision:

```fab
fixum fractus pi = 3.14159
fixum fractus rate = 0.05
```

The distinction between `numerus` and `fractus` mirrors the distinction between integers and floating-point numbers in systems languages. When precision matters---financial calculations, scientific computing---you choose deliberately.

### bivalens (Boolean)

From *bi-*, "two," and *valens*, "being strong, having value." A two-valued type. The boolean literals are `verum` (true) and `falsum` (false):

```fab
fixum bivalens active = verum
fixum bivalens disabled = falsum
```

These literals read as Latin adjectives: "it is true," "it is false."

### nihil (Null)

From the Latin *nihil*, "nothing." The absence of value. Where other languages use `null` or `nil` or `None`, Faber uses `nihil`:

```fab
fixum nothing = nihil
```

A variable holding `nihil` holds nothing---not zero, not an empty string, but the explicit absence of any value.

### vacuum (Void)

From the Latin *vacuum*, "empty, void." This is the return type of functions that complete but return no value:

```fab
functio log(textus message) -> vacuum {
    scribe message
}
```

The distinction from `nihil` is semantic: `nihil` is a value (the null value), while `vacuum` is the absence of a return. A function returning `vacuum` completes normally; it simply has nothing to hand back.

---

## Special Types

### ignotum (Unknown)

From *in-*, "not," and *gnoscere*, "to know." The unknown type. Unlike permissive "any" types in other languages, `ignotum` requires you to narrow before use:

```fab
fixum ignotum data = getExternalData()

# Error: cannot use ignotum directly
# scribe data.length

# Must narrow first
si data est textus {
    scribe data.longitudo()
}
```

Faber deliberately omits an "any" type. When you receive data of unknown type, you must either narrow it with type guards (`est`) or cast it explicitly (`qua`). This design makes uncertainty visible and intentional.

### numquam (Never)

From the Latin *numquam*, "never." This is the return type of functions that never return---those that throw exceptions, loop infinitely, or exit the process:

```fab
functio moritur() -> numquam {
    iace novum Error { message: "fatal" }
}

functio infinitus() -> numquam {
    dum verum { }
}
```

A function marked `numquam` is a one-way door. Control enters but does not exit. This distinguishes it from `vacuum`: a void function returns (with no value); a never function does not return at all.

### objectum (Object)

From the Latin *objectum*, "something thrown before." The root of the English word "object." This type represents any non-primitive value---anything that is not a number, string, boolean, or null:

```fab
functio getUser() -> objectum {
    redde { name: "Marcus", age: 30 }
}
```

Use `objectum` when a function returns an anonymous object structure. For known shapes, prefer defining a `genus` (struct) instead.

---

## Type Annotations

Faber uses type-first syntax. The type precedes the identifier it annotates:

```fab
fixum textus nomen = "Marcus"
varia numerus count = 0
```

This ordering---type before name---reflects Latin's pattern where descriptive modifiers precede their nouns. It also mirrors how we think: "I need a number for counting," not "I need count, which is a number."

### In Variable Declarations

Both immutable (`fixum`) and mutable (`varia`) declarations support type annotations:

```fab
fixum numerus age = 30
varia textus status = "pending"
```

Type annotations are optional when the type can be inferred:

```fab
fixum name = "Marcus"    # inferred as textus
fixum count = 42         # inferred as numerus
```

### In Function Signatures

Function parameters use the same type-first pattern. Return types follow the arrow:

```fab
functio adde(numerus a, numerus b) -> numerus {
    redde a + b
}

functio describe(textus nomen, numerus aetas) -> textus {
    redde scriptum("§ habet § annos", nomen, aetas)
}
```

A function that returns nothing uses `vacuum` or omits the return type entirely:

```fab
functio log(textus message) -> vacuum {
    scribe message
}
```

### Function Types

Higher-order functions accept or return other functions. Faber uses parenthesized syntax for function type annotations:

```fab
(paramTypes) -> returnType
```

A predicate function that takes a value and returns a boolean:

```fab
functio filtra(lista<T> items, (T) -> bivalens pred) -> lista<T> {
    # ...
}
```

A transformer that takes two functions and returns their composition:

```fab
functio compone((A) -> B f, (B) -> C g) -> (A) -> C {
    redde pro a: f(a) |> g
}
```

Function types can appear anywhere a type annotation is valid---parameters, return types, variable declarations, and generics:

```fab
fixum (numerus) -> numerus doubler = pro x: x * 2
typus Predicate<T> = (T) -> bivalens
typus Mapper<A, B> = (A) -> B
```

Multiple parameters are comma-separated; no parameters use empty parentheses:

```fab
() -> vacuum                     # no params, returns nothing
(numerus) -> textus              # one param
(textus, numerus) -> bivalens    # two params
```

---

## Nullable Types

The `?` suffix marks a type as nullable---able to hold either a value of that type or `nihil`:

```fab
fixum textus? maybeName = nihil
fixum numerus? maybeCount = 42
```

A nullable type requires handling before use. You cannot call methods on a `textus?` without first checking that it is not `nihil`:

```fab
fixum textus? name = getOptionalName()

# Using type guard
si name est nihil {
    scribe "No name provided"
}
secus {
    scribe name.longitudo()
}
```

The `est` operator performs type checking:

```fab
fixum isNull = maybeValue est nihil
```

Function return types can be nullable to indicate that a function might not find what it is looking for:

```fab
functio inveni(textus id) -> persona? {
    # might return nihil if not found
}
```

---

## Collections

Faber provides three built-in collection types, each named for Latin words describing containment and abundance.

### lista (Array/List)

From the Latin *lista*, "border, strip, list." An ordered, indexable sequence of elements:

```fab
fixum lista<textus> names = ["Marcus", "Julia", "Gaius"]
fixum lista<numerus> scores = [100, 95, 87]
```

Array shorthand uses brackets after the type:

```fab
fixum textus[] names = ["Marcus", "Julia", "Gaius"]
```

The shorthand `textus[]` is equivalent to `lista<textus>`.

For empty collections, use `innatum` to construct the native type:

```fab
varia items = [] innatum lista<textus>
varia cache = {} innatum tabula<textus, numerus>
```

Without `innatum`, empty literals lack methods. The `innatum` keyword (Latin "inborn") creates proper native instances---`new Map()` in TypeScript, `HashMap` in Rust, etc.

Access elements by index:

```fab
fixum first = names[0]
fixum last = names[names.longitudo() - 1]
```

### tabula (Map/Dictionary)

From the Latin *tabula*, "tablet, table, board." The writing tablet, a surface for recording associations. A `tabula` maps keys to values:

```fab
fixum tabula<textus, numerus> ages = {
    "Marcus": 30,
    "Julia": 25,
    "Gaius": 40
}
```

The type takes two parameters: key type and value type. Access values by key:

```fab
fixum marcusAge = ages["Marcus"]
```

### copia (Set)

From the Latin *copia*, "abundance, supply, plenty." A collection of unique values with no duplicates:

```fab
fixum copia<textus> uniqueNames = copia("Marcus", "Julia", "Marcus")
# Contains only "Marcus" and "Julia"
```

Sets are useful when you care about membership and uniqueness rather than order or key-value association.

---

## Type Aliases

The `typus` keyword creates named aliases for types, improving readability and enabling reuse:

```fab
typus UserId = numerus
typus Username = textus
typus IsActive = bivalens
```

Once defined, type aliases can be used anywhere a type is expected:

```fab
fixum UserId id = 42
fixum Username name = "Marcus"
```

Aliases are especially useful for complex types:

```fab
typus Names = lista<textus>
typus UserCache = tabula<textus, numerus>
typus OptionalName = textus?
```

This makes function signatures more readable:

```fab
functio findUser(UserId id) -> Username? {
    # ...
}
```

### typeof Extraction

Use `typus` on the right-hand side to extract a type from a value:

```fab
fixum config = { port: 3000, host: "localhost" }
typus Config = typus config
```

This creates a type alias matching the inferred type of the value.

---

## Generics

Type parameters allow writing code that works with multiple types. Enclose parameters in angle brackets:

```fab
genus capsa<T> {
    T valor

    functio accipe() -> T {
        redde ego.valor
    }
}

fixum c = novum capsa<numerus> { valor: 42 }
scribe c.accipe()
```

Generic constraints can limit what types are acceptable:

```fab
pactum Comparable<T> {
    functio compare(T other) -> numerus
}

functio maximum<T implet Comparable<T>>(T a, T b) -> T {
    si a.compare(b) > 0 {
        redde a
    }
    redde b
}
```

The type parameter `T` is constrained to types that implement the `Comparable` interface.

---

## Union Types

The `unio<A, B>` generic expresses that a value may be one of several types:

```fab
typus StringOrNumber = unio<textus, numerus>

functio process(StringOrNumber value) -> textus {
    si value est textus {
        redde value
    }
    redde value.toString()
}
```

Union types require narrowing before use. The compiler does not know which variant you have until you check:

```fab
fixum unio<textus, numerus> value = getValue()

# Must narrow
si value est numerus {
    scribe value * 2
}
sin value est textus {
    scribe value.longitudo()
}
```

---

## Type Casting

The `qua` keyword performs explicit type conversion:

```fab
fixum data = 42
fixum asText = data qua textus
```

Casts are explicit acknowledgments of risk. When you write `qua`, you are telling the compiler: "I know what I am doing." The compiler trusts you---but if you are wrong, runtime errors follow.

```fab
# Cast to nullable type
fixum num = 10
fixum maybe = num qua numerus?

# Cast with member access
fixum response = getResponse()
fixum body = response.body qua textus

# Cast for chaining
fixum len = (data qua textus).length
```

Use casts sparingly. Prefer type guards (`est`) when possible, as they provide compile-time safety.

---

## Type Checking

The `est` keyword checks whether a value is of a given type:

```fab
fixum maybeValue = getValue()

si maybeValue est textus {
    # Within this block, maybeValue is known to be textus
    scribe maybeValue.longitudo()
}
```

Type guards work with nullable types:

```fab
fixum textus? name = getOptionalName()

si name est nihil {
    scribe "No name"
}
secus {
    scribe name
}
```

The compiler narrows the type within the guarded block, eliminating the need for casts.
