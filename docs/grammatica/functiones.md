# Functiones

Functions in Faber are declared using the `functio` keyword, derived from the Latin _functio_ meaning "performance, execution." This chapter covers function declarations, parameters, return types, async patterns, generators, and clausura expressions.

## Declaring Functions

### Basic Syntax

A function declaration begins with `functio` followed by the function name, parameter list in parentheses, optional return type, and the function body in braces:

```fab
functio saluta() {
    nota "Salve, Mundus!"
}
```

Functions that return values specify the return type after an arrow (`→`) and use `redde` (Latin "give back, return") to yield the result:

```fab
functio nomen() → textus {
    redde "Marcus Aurelius"
}
```

Functions may also declare a recoverable alternate-exit type after `⇥`. `redde`
still exits through the normal `→` path; `iace` exits through the `⇥` path and
must match that type.

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b ≡ 0 ergo iace "division by zero"
    redde a / b
}
```

`mori` remains fatal and is not part of the typed recoverable path. Calling a
function with a `⇥` contract requires explicit handling or propagation syntax.
Today, `fac { ... } cape err { ... }` is the canonical local handler form; broader
propagation syntax remains deferred.

See [`examples/exempla/functio/exitus.fab`](../../examples/exempla/functio/exitus.fab)
for a small recoverable-exit example.

### Parameters

Faber uses type-first syntax for parameters, placing the type before the parameter name. This mirrors natural language order ("a string called name") and aligns with languages like Go, Rust, and Zig:

```fab
functio quadratum(numerus n) → numerus {
    redde n * n
}

functio adde(numerus a, numerus b) → numerus {
    redde a + b
}

functio describe(textus nomen, numerus aetas) → textus {
    redde "§ habet § annos"(nomen, aetas)
}
```

When a parameter has no explicit type annotation, the compiler infers it from usage:

```fab
functio duplica(n) → numerus {
    redde n * 2
}
```

### Dual Parameter Naming

Following Swift's pattern, parameters can have separate external (callsite) and internal (body) names using `ut` (Latin "as"):

```fab
functio greet(textus location ut loc) {
    nota loc  # internal name
}

greet(location: "Roma")  # external name at callsite
```

The `ut` keyword provides a unified aliasing syntax across the language:

- Imports: `importa ex "norma:hal/consolum" privata consolum`
- Destructuring: `ex persona fixum nomen ut n`
- Parameters: `textus location ut loc`

All three express the same concept: "X, known locally as Y."

### Voluntary Parameters with `sponte`

The `sponte` marker (Latin "of one's own accord") follows the parameter name and marks the slot as voluntary: the caller may omit it. It is a declaration marker, not a type prefix.

```fab
functio greet(textus nomen, textus titulus sponte) → textus {
    si titulus est nihil {
        redde "Salve, §!"(nomen)
    }
    redde "Salve, § §!"(titulus, nomen)
}

greet("Marcus")              # titulus receives nihil
greet("Marcus", "Dominus")   # titulus receives "Dominus"
```

### Default Values

Default values use `vel` (Latin "or"), consistent with the nullish coalescing operator in expressions. The `vel` follows any `sponte` marker:

```fab
functio paginate(numerus pagina sponte vel 1, numerus per_pagina sponte vel 10) → textus {
    redde "Page § with § items"(pagina, per_pagina)
}

paginate()        # "Page 1 with 10 items"
paginate(2)       # "Page 2 with 10 items"
paginate(2, 25)   # "Page 2 with 25 items"
```

The choice of `vel` provides consistency: `vel` already means "or if nil" in expressions like `value vel "default"`, making parameter defaults read naturally as "numerus pagina or 1."

Default values only make sense for owned parameters. Borrowed (`de`) and mutable (`in`) parameters require the caller to provide a value since there is nothing to borrow by default.

`sponte` governs whether a value must be *provided* at the call site. When the value domain itself may be `nihil`, use the union form `T ∪ nihil` in return types, aliases, and type annotations (see `∪` and the typi chapter). These are distinct concepts.

### Rest Parameters

The `ceteri` modifier (Latin "the rest, the others") collects remaining arguments into an array:

```fab
functio sum(ceteri numerus[] nums) → numerus {
    varia _ total ← 0
    itera ex nums fixum n {
        total ⊕ n
    }
    redde total
}

sum(1, 2, 3, 4, 5)  # 15
```

Rest parameters must come last in the parameter list.

## Return Types

### Arrow Syntax

The arrow `→` specifies a function's return type directly. This is the simplest form and compiles with minimal overhead:

```fab
functio compute() → numerus {
    redde 42
}
```

When the function returns nothing, omit the return type entirely or specify `vacuum`:

```fab
functio doNothing() {
    # no return value
}

functio doNothingExplicit() → vacuum {
    redde
}
```

### Historical Note On Older Docs

Some older Faber docs and examples used verb-form return syntax such as `fit`, `fiet`, `fiunt`, and `fient`. The current grammar contract in [`EBNF.md`](../../EBNF.md) no longer uses those forms for function declarations.

Today, function return shape is expressed with:

- `→` for the return type
- `@ futura` for async functions
- `@ cursor` for generator functions
- both `@ futura` and `@ cursor` for async generators

```fab
functio getId() → textus {
    redde "abc"
}

@ futura
functio fetchData(textus url) → textus {
    redde "data"
}

@ cursor
functio range(numerus n) → numerus {
    itera ex 0..n fixum i {
        cede i
    }
}
```

Treat any verb-form function examples you encounter elsewhere in the docs as historical drift unless the grammar contract is updated.

## Async Functions

### The `@ futura` Annotation

The `@ futura` annotation (Latin "future things," neuter plural of _futurus_) marks a function as asynchronous. Combined with arrow syntax, it returns a `promissum<T>` / Promise:

```fab
@ futura
functio fetchData(textus url) → textus {
    fixum _ response = cede fetch(url)
    redde response.text()
}
```

The choice of _futura_ leverages Latin's grammatical future tense to express temporal semantics: the result will be available in the future.

### The cede Keyword

Inside async functions, `cede` (Latin "yield, give way, surrender") awaits a promise:

```fab
@ futura
functio processAll(textus[] urls) → textus[] {
    varia _ results = []
    itera ex urls fixum url {
        fixum _ data = cede fetchData(url)
        results.appende(data)
    }
    redde results
}
```

The etymology captures the semantics precisely: the function cedes control until the async operation completes.

### Current Async Declaration

Use `@ futura` with arrow return syntax for async functions:

```fab
@ futura
functio fetchData() → textus {
    redde "data"
}
```

If you see `fiet` in older material, treat it as a stale example rather than current grammar.

## Generator Functions

### The `@ cursor` Annotation

The `@ cursor` annotation (Latin "runner," from _currere_ "to run") creates a generator function:

```fab
@ cursor
functio range(numerus n) → numerus {
    itera ex 0..n fixum i {
        cede i
    }
}
```

In generator context, `cede` yields values rather than awaiting them, reusing the same keyword for both semantics based on function context.

### Async Generators

Async generators combine both annotations:

```fab
@ futura
@ cursor
functio fetchAll(textus[] urls) → textus {
    itera ex urls fixum url {
        cede fetch(url)
    }
}
```

If you see `fiunt` or `fient` in older material, treat them as historical examples rather than current declaration syntax.

### Iterating Over Generators

Generator results can be consumed with `itera ex` loops:

```fab
itera ex rangeSync(5) fixum num {
    nota num
}
```

## Generic Functions

### Type Parameters with prae

The `prae` keyword (Latin "before") declares compile-time type parameters. Combined with `typus` ("type"), it introduces generic type variables:

```fab
functio max(prae typus T, T a, T b) → T {
    si a > b { redde a }
    redde b
}

fixum _ larger = max(10, 20)           # T inferred as numerus
fixum _ longer = max("alpha", "beta")  # T inferred as textus
```

Type parameters must come first in the parameter list, followed by regular parameters. This matches conventions in TypeScript, Rust, and Zig.

Multiple type parameters are supported:

```fab
functio pair(prae typus T, prae typus U, T first, U second) → [T, U] {
    redde [first, second]
}
```

## Clausura Expressions (Closures)

### Basic Syntax

Clausura expressions use `clausura` (Latin for "closure") followed by parameters, a colon, and an expression:

```fab
fixum _ double = clausura x: x * 2
fixum _ add = clausura a, b: a + b
```

The colon separates parameters from the body. For single expressions, the result is implicitly returned.

### With Return Type Annotation

When type annotation is needed, use an arrow before the colon:

```fab
fixum _ add = clausura a, b → numerus: a + b
fixum _ isPositive = clausura n → bivalens: n > 0
```

### Block Bodies

For multi-statement clausuras, use braces and explicit `redde`:

```fab
fixum _ process = clausura x {
    varia _ result = x * 2
    result ⊕ 10
    redde result
}
```

### Zero-Parameter Clausuras

When a clausura takes no parameters, place the colon immediately after `clausura`:

```fab
fixum _ getFortyTwo = clausura: 42
```

### Async Clausuras

Async is inferred from the presence of `cede` in the body:

```fab
fixum _ fetchAndProcess = clausura url {
    fixum _ data = cede fetch(url)
    redde process(data)
}
```

This is useful for callbacks in async contexts:

```fab
app.post("/users", clausura context {
    fixum _ data = cede context.json()
    redde data
})
```

### Common Patterns

Clausuras shine in functional operations:

```fab
fixum _ numbers = [1, 2, 3, 4, 5]

# Filter
fixum _ evens = numbers.filter(clausura x: x % 2 ≡ 0)

# Map
fixum _ doubled = numbers.map(clausura x: x * 2)

# Reduce
fixum _ sum = numbers.reduce(0, clausura acc, x: acc + x)
```

## Allocator Binding with curata

The `curata` modifier (Latin "cared for," from _curare_ "to care for") declares that a function requires an allocator. This is essential for Zig targets where memory allocation is explicit:

```fab
functio greet(textus name) curata alloc ut a → textus {
    redde "Hello, §!"(name)
}
```

The allocator name following `curata` declares the required allocator key. Use `ut` to give it a local body name; without `ut`, the local name is the same as the required name.

At call sites, the allocator is automatically injected when calling from within a `cura` block:

```fab
incipit {
    cura "arena" fixum _ alloc {
        nota greet("World")  # alloc auto-injected for Zig targets
    }
}
```

The `curata` modifier keeps the function signature clean: the allocator is a Zig codegen concern, not a semantic parameter. Callers within a `cura` block don't pass it explicitly; Zig lowering can thread it through.

For non-Zig targets, `curata` and `cura` allocator scopes have no runtime effect.

## Ownership Prepositions in Parameters

Latin prepositions indicate how parameters are passed and what the function may do with them:

- `de` (from/concerning): borrowed, read-only access
- `in` (into): mutable borrow, the function may modify the value

```fab
functio processPoints(de Point[] points, in Point[] targets) {
    # points is borrowed (read-only)
    # targets is mutably borrowed
    itera ex points fixum point {
        targets.appende(point)
    }
}
```

These prepositions combine naturally with other parameter modifiers:

```fab
functio analyze(textus source, de numerus depth sponte) → numerus {
    si depth est nihil { redde 3 }
    redde depth
}
```

The prepositions express semantic intent about ownership and mutability. They serve as documentation for readers and enable stricter checking in future compiler versions.

## Summary

Faber's function system balances Latin linguistic authenticity with practical programming needs:

- `functio` for declaration, `redde` for return
- Type-first parameters with `ut` aliasing
- `si` for optional, `vel` for defaults, `ceteri` for rest
- Arrow `→` for return types
- `@ futura` and `@ cursor` annotations for async and generator behavior, plus post-function modifiers such as `curata`, `errata`, `immutata`, and `iacit`
- `cede` for await (async) or yield (generator)
- `prae typus` for generics
- `clausura` for closures (async inferred from `cede` usage)

The Latin vocabulary maps naturally to programming concepts: _futura_ captures async's temporal nature, _cursor_ captures generator behavior, and _cede_ captures yielding control.
