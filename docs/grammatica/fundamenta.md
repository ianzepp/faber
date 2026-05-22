# Fundamenta

The fundamentals of Faber: program structure, variable bindings, literals, output, and patterns. These are the building blocks upon which all Faber programs rest. Understanding them means understanding how Faber uses Latin's clarity to make programming concepts visible.

For quick installed help on a glyph or keyword, use `faber explain <term>`, for example `faber explain ≡` or `faber explain proba`. The explain corpus is embedded into the Faber tool so it works outside a checkout.

## Program Structure

Every Faber program needs an entry point. This is where execution begins, and Faber provides two forms depending on whether your program is synchronous or asynchronous.

### Synchronous Entry: incipit

The keyword `incipit` marks the synchronous entry point. It is the third person singular present active indicative of _incipere_ (to begin): "it begins."

```fab
incipit {
    nota "Salve, Munde!"
}
```

This is the simplest possible Faber program. The `incipit` block contains the statements that execute when the program runs. Functions and types defined outside `incipit` become module-level declarations that the entry point can call.

```fab
functio greet(textus name) → textus {
    redde "Salve, §!"(name)
}

incipit {
    nota greet("Marcus")
}
```

### Asynchronous Entry: incipiet

When your program needs to perform asynchronous operations at the top level, use `incipiet` instead. This is the future tense: "it will begin." The naming reflects the same present-versus-future contrast that Faber uses elsewhere in its Latin vocabulary, but current function declarations still use arrow returns plus annotations such as `@ futura`.

```fab
incipiet {
    fixum _ data = cede fetchData()
    nota data
}
```

Inside an `incipiet` block, you can use `cede` (yield/await) with regular bindings to await async operations.

## Variables

Faber distinguishes between mutable and immutable bindings with distinct keywords. This explicitness is a core design principle: the reader should know at a glance whether a value can change.

### Immutable Bindings: fixum

The keyword `fixum` declares an immutable binding. It is the perfect passive participle of _figere_ (to fix, fasten): "that which has been fixed." Once bound, a `fixum` value cannot be reassigned.

```fab
fixum _ greeting ← "Salve, Mundus!"
fixum _ x ← 10
fixum _ y ← 20
fixum _ sum ← x + y
```

Immutable bindings are the default choice in Faber. They communicate intent clearly: this value will not change for the remainder of its scope. Prefer `fixum` unless you have a specific reason for mutability.

Type annotations are optional when the type can be inferred, but you can be explicit:

```fab
fixum numerus age ← 30
fixum textus name ← "Marcus"
fixum bivalens active ← verum
```

The pattern is always type-first: `fixum <type> <name> ← <value>`. This mirrors Latin's adjective-noun ordering and distinguishes Faber from languages that place types after names.

### Mutable Bindings: varia

The keyword `varia` declares a mutable binding. It comes from _variare_ (to vary): "let it vary." A `varia` binding can be reassigned throughout its scope.

```fab
varia _ counter ← 0
nota counter       # 0

counter = 1
nota counter       # 1

counter = counter + 10
nota counter       # 11
```

Use `varia` when a value genuinely needs to change, such as loop counters, accumulators, or state that evolves over time.

```fab
varia numerus count = 0
varia textus status = "pending"
varia bivalens running = falsum

count = 100
status = "complete"
running = verum
```

## Literals

Faber supports the standard literal types with Latin keywords for boolean and null values.

### Numbers

Integers and floating-point numbers use standard notation:

```fab
fixum _ integer = 42
fixum _ decimal = 3.14
fixum _ negative = -100
```

For typed declarations, `numerus` is the integer type and `fractus` (from _frangere_, to break) is the floating-point type:

```fab
fixum numerus count = 42
fixum fractus rate = 0.05
```

### Strings

Strings use double quotes:

```fab
fixum _ greeting = "hello"
```

Block strings use `❝` and `❞`, may span lines, and preserve their content:

```fab
fixum _ quote = ❝he said "salve"❞
```

Formatted strings use template application with `§` placeholders:

```fab
fixum _ name = "Mundus"
fixum _ message = "Hello §"(name)
```

### Booleans: verum and falsum

Rather than `true` and `false`, Faber uses Latin: `verum` (true, real) and `falsum` (false, deceptive). These are not arbitrary choices. Latin's _verum_ shares its root with English "verify" and "veracity"; _falsum_ gives us "falsify."

```fab
fixum _ yes = verum
fixum _ no = falsum
fixum bivalens active = verum
```

The type `bivalens` (two-valued) names what a boolean is: a value that can be one of exactly two states.

### Null: nihil

The absence of a value is expressed as `nihil` (nothing). This is clearer than symbols like `null` or `nil` that have become so familiar we no longer notice their meaning.

```fab
fixum _ nothing = nihil
```

## Diagnostics

Faber provides diagnostic statements corresponding to different severity levels. These are for developer-facing notes, inspection, and warnings; use HAL stdlib methods such as `consolum.scribe` or `solum.scribe` for program output.

### Neutral Note: nota

The keyword `nota` emits a neutral diagnostic note. It is the imperative of _notare_ (to note, mark).

```fab
nota "Hello, world!"
```

Multiple arguments are emitted space-separated:

```fab
fixum _ nomen = "Marcus"
fixum _ aetas = 30
nota "Name:", nomen
nota "Age:", aetas
nota "Coordinates:", x, y
```

`scribe` remains as a compatibility alias for neutral diagnostic output, but new code should prefer `nota`.

### Debug Output: vide

The keyword `vide` writes to debug output. It is the imperative of _videre_ (to see): "see!" Use it for diagnostic information that should be visible during development but filtered in production.

```fab
vide "Debug: entering main loop"
vide "Debug: count =", count
```

### Warning Output: mone

The keyword `mone` writes to warning output. It is the imperative of _monere_ (to warn, advise): "warn!" Use it for conditions that are not errors but deserve attention.

```fab
mone "Warning: deprecated feature used"
mone "Warning: count exceeds threshold:", count
```

## Comments

Comments begin with `#` and extend to the end of the line. There is no block comment syntax.

```fab
# This is a comment
fixum _ x = 10  # inline comment
```

Comments explain _why_, not _what_. The code itself shows what is happening; comments provide context that cannot be derived from the code alone.

## Destructuring

Faber provides patterns for extracting values from objects and arrays. The syntax uses `ex` (from, out of) to indicate the source and the binding keyword to indicate mutability.

### Object Destructuring

To extract properties from an object, use `ex <source> fixum <properties>`:

```fab
fixum _ person = { name: "Marcus", age: 30, city: "Roma" }
ex person fixum name, age

nota name   # "Marcus"
nota age    # 30
```

Use `ut` (as) to rename properties during extraction:

```fab
fixum _ user = { name: "Julia", email: "julia@roma.com" }
ex user fixum name ut userName, email ut userEmail

nota userName    # "Julia"
nota userEmail   # "julia@roma.com"
```

Use `varia` instead of `fixum` for mutable bindings:

```fab
fixum _ data = { count: 100, active: verum }
ex data varia count, active

count = 200
active = falsum
```

The rest pattern `ceteri` (the rest, the others) collects remaining properties:

```fab
fixum _ fullUser = { id: 1, name: "Gaius", email: "g@roma.com", role: "admin" }
ex fullUser fixum id, ceteri details

nota id       # 1
nota details  # { name: "Gaius", email: "g@roma.com", role: "admin" }
```

### Array Destructuring

Arrays use bracket notation in the pattern:

```fab
fixum _ numbers = [1, 2, 3]
fixum [a, b, c] = numbers

nota a  # 1
nota b  # 2
nota c  # 3
```

Partial destructuring extracts only what you need:

```fab
fixum _ values = [1, 2, 3, 4, 5]
fixum [one, two] = values

nota one  # 1
nota two  # 2
```

The underscore `_` skips elements:

```fab
fixum _ triple = [10, 20, 30]
fixum [_, middle, _] = triple

nota middle  # 20
```

The rest pattern works with arrays too:

```fab
fixum _ items = [1, 2, 3, 4, 5]
fixum [head, ceteri tail] = items

nota head  # 1
nota tail  # [2, 3, 4, 5]
```

Mutable array destructuring uses `varia`:

```fab
fixum _ coords = [100, 200]
varia [x, y] = coords

x = x + 50
y = y + 50

nota x  # 150
nota y  # 250
```

---

These fundamentals are the vocabulary and grammar of Faber. With them, you can write clear, expressive programs. The more advanced features documented elsewhere build upon this foundation, but these basics are sufficient for substantial work.
