# Faber Compared To Familiar Languages

This note is an orientation guide for readers and LLMs that already know
languages like Python, Rust, TypeScript, and Go. It is not the grammar
reference, and it is not a catalog of every example in `examples/exempla`.

Use these files as the current authority when precision matters:

- [`../EBNF.md`](../EBNF.md) for grammar and spec commentary.
- [`../README.md`](../README.md) for repo shape, command roles, and current
  tool boundaries.
- [`../explain`](../explain) and `faber explain <term>` for user-facing keyword
  and glyph reference.
- [`../stdlib/norma`](../stdlib/norma) for standard library declarations and
  target translation metadata.
- [`../crates/radix`](../crates/radix) for compiler implementation work.
- [`../crates/faber`](../crates/faber) for project and package tooling.

The short version:

- Python: say the thing directly.
- Rust: prove the thing mechanically.
- TypeScript: describe the thing structurally and let the ecosystem carry the
  runtime.
- Go: keep the thing ordinary and explicit.
- Faber: state the thing in a regular, reviewable form before it lowers to a
  target language.

Faber's center is not "Latin Python" or "Rust with different keywords." It is a
small source language that uses type-first declarations, Latin behavioral words,
and structural glyphs to make program intent easy to scan and hard to confuse.

## Current Tool Model

There are two active command surfaces:

- `faber` is the user-facing project and package tool.
- `radix` is the compiler-developer inspection tool.

The current compiler authority is Radix. The active pipeline is:

```text
Lex -> Parse -> Collect -> Resolve -> Lower -> Typecheck -> Analysis -> Codegen
```

Rust is the primary executable backend and the only package build target today.
TypeScript and Go support file emission. The `faber` target is a canonical
pretty-printer and round-trip surface, not a separate compiler.

That matters when reading or generating Faber: do not assume every target has
the same maturity. Prefer source that is explicit enough for the compiler to
carry facts forward rather than asking codegen to guess.

## What Faber Feels Like

Faber feels declarative, narrated, and review-oriented.

The syntax tries to keep semantic categories visible. This target-language
shape is not Faber:

```text
functio divide(numerus a, numerus b) -> numerus
```

Faber writes the type first and uses the canonical return glyph:

```fab
functio divide(numerus a, numerus b) → numerus
```

Local binding is not `let name: Type = value`; it is a declaration role, a type,
a name, and runtime binding:

```fab
fixum numerus count ← 0
varia textus label ← "ready"
```

If the initializer determines the type, use `_`:

```fab
fixum _ names ← ["Marcus", "Julia"]
```

This gives Faber a different rhythm from C-family and Python-family languages.
Declarations read from semantic shape toward binding name. Runtime value flow is
marked with `←`; compile-time definition uses `=`.

## Python Comparison

Python optimizes for immediate expression:

```python
def greet(name):
    return f"Salve, {name}"
```

The closest Faber shape is more explicit:

```fab
functio greet(textus name) → textus {
    redde "Salve, §"(name)
}
```

The difference is not just vocabulary.

Python often relies on surrounding context and library convention. Faber tries
to put more of the shape in the source line itself:

- `textus name`, not an untyped parameter.
- `→ textus`, not inferred return intent.
- `redde`, not a general-purpose English keyword reused from many languages.
- string-template application, not interpolation syntax borrowed from one target.

For quick scripts, Python is more fluid. For source that should be inspected by
another tool or another person before lowering to Rust, TypeScript, or Go, Faber
prefers steadier structure.

## Rust Comparison

Rust makes ownership, mutability, lifetimes, traits, and failure extremely
visible. Faber borrows some of that seriousness but not Rust's full semantic
pressure.

Rust:

```rust
fn divide(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        return None;
    }
    Some(a / b)
}
```

Faber:

```fab
functio divide(numerus a, numerus b) → numerus ∪ nihil {
    si b ≡ 0 ∴ redde nihil
    redde a / b
}
```

The nullable type is explicit, but it is written as a value-domain union:

```fab
numerus ∪ nihil
```

Do not invent `numerus?`, `Option<numerus>`, or `nihil numerus`.

Faber has ownership-flavored markers such as `de`, `in`, and `ex`, and Radix
enables Rust-specific borrow analysis for the Rust target. Still, Faber is not
Rust. Its source is more about preserving high-level intent and giving the
compiler enough structure to lower honestly. When Rust-specific representation
or borrow facts matter, assume they must be checked against Radix and the Rust
backend, not inferred from the prettiness of the Faber source.

## TypeScript Comparison

TypeScript and Faber both have structural instincts, but they put structure in
different places.

TypeScript:

```ts
type UserId = number;

interface Drawable {
  draw(): void;
}
```

Faber:

```fab
typus UserId = numerus

pactum Drawable {
    functio draw() → vacuum
}
```

Faber declarations are more uniform and less punctuation-heavy. `genus` covers
class or struct-shaped declarations; `pactum` covers interface-shaped contracts;
`discretio` covers tagged unions.

Object construction is also declarative:

```fab
genus Point {
    numerus x
    numerus y
}

fixum _ p ← Point {
    x = 10,
    y = 20
}
```

Notice the split:

- `=` defines fields inside a structural literal or declaration metadata.
- `←` binds the runtime value to `p`.

That distinction is one of the fastest ways to avoid writing target-language
syntax by mistake.

## Go Comparison

Go and Faber both value plain control flow, but Faber is more grammatical.

Go:

```go
for _, item := range items {
    if item == target {
        return item
    }
}
```

Faber:

```fab
itera ex items fixum item {
    si item ≡ target ∴ redde item
}
```

Iteration says whether it is walking values or keys:

```fab
itera ex items fixum item { ... }  # values
itera de table fixum key { ... }   # keys
itera pro 0‥10 fixum i { ... }     # range
```

Faber control flow is compact, but it is not free-form. Branches use a block or
a one-statement consequent introduced by `∴` or its accepted Latin alias `ergo`:

```fab
si score ≥ 90 ∴ redde "A"
sin score ≥ 80 ∴ redde "B"
secus ∴ redde "C"
```

Prefer `∴` in canonical examples. `ergo` is accepted, but glyph-first source is
the current style for structural joints.

## Canonical Surface

When generating or reviewing Faber, default to these forms:

```fab
fixum textus name ← "Marcus"
varia numerus count ← 0
functio greet(textus name) → textus { ... }
si ready ∴ redde value
fixum _ n ← raw ⇒ numerus vel 0
fixum _ x ← value ⇢ textus
functio maybe() → textus ∪ nihil { ... }
```

Avoid these stale or non-canonical forms:

```fab
name: textus
functio greet(name: textus): textus
textus?
Option<textus>
tempta { ... } cape err { ... }
demum { ... }
value qua textus
value innatum textus
novum Point { x = 1 }
```

Current error handling uses `fac` with `cape` for a local recoverable boundary:

```fab
fac {
    iace "bad input"
} cape err {
    nota err
}
```

`tempta` is legacy and Radix rejects it with a migration diagnostic. `demum`
cleanup semantics are deferred.

## Glyphs And Words

Faber uses glyphs where the symbol is structural:

- `←` for runtime binding and assignment.
- `→` for function return type.
- `⇥` for alternate recoverable exit type.
- `∴` for therefore or compact body.
- `≡` and `≠` for equality.
- `∪` for inline union types.
- `⇢` for type-directed inhabitation.
- `⇒` for runtime conversion.

Faber uses Latin words where the construct has behavioral or grammatical shape:

- `functio`, `genus`, `pactum`, `typus`, `ordo`, `discretio`.
- `fixum`, `varia`, `redde`, `iace`, `mori`, `tacet`.
- `si`, `sin`, `secus`, `dum`, `itera`, `elige`, `discerne`.
- `fac`, `cape`, `cura`.

That division is part of the feel of the language. A good Faber source file
should look like a stable grammar, not like a target language wearing Latin
labels.

## Nullability And Optionality

Faber distinguishes absence in a value from optional provision at a declaration
site.

Use `T ∪ nihil` when the value can be absent:

```fab
functio find(textus key) → numerus ∪ nihil
```

Use `sponte` after the name when a parameter or field may be omitted by the
caller or constructor:

```fab
functio connect(textus host, numerus port sponte) → vacuum

genus User {
    textus email sponte
}
```

Use `fixus` after the name for late-initialized slots that become fixed after
assignment. The canonical combined order is `sponte fixus`.

Do not use `ignotum` as a nullability escape. `ignotum` is the top-level unknown
type for escape hatches and incomplete knowledge, not "maybe null."

## Conversion And Construction

There are two important arrows:

```fab
fixum _ parsed ← "42" ⇒ numerus
fixum _ safe ← input ⇒ numerus vel 0
fixum _ text ← value ⇢ textus
```

Use `⇒` for runtime conversion, especially parsing or coercion that may need a
fallback.

Use `⇢` for type-directed inhabitation. It is postfix and target-type driven.
Prefer ordinary typed construction and `vacua` where they say the same thing
more directly:

```fab
fixum lista<numerus> xs ← vacua
fixum _ p ← Point { x = 10, y = 20 }
```

## Standard Library Feel

The standard library lives as Faber declarations in `stdlib/norma`, with target
translation metadata such as `@ verte`. A method like a list append can have one
Faber name and several target spellings:

```fab
@ verte ts "push"
@ verte rs "push"
@ externa
functio appende(T elem) → vacuum
```

This is a major part of Faber's design: source code should talk in Faber's
semantic vocabulary, while backend-specific names remain metadata.

That does not mean all target behavior is identical. Runtime-backed Rust support
lives in `crates/norma`, while other runtime support may live under `runtimes/`.
Treat target support as a current compiler capability, not as a promise implied
by a stdlib declaration alone.

## Common LLM Failure Modes

These are the mistakes to avoid during warm-up or code generation:

- Writing `name: Type` instead of `Type name`.
- Using ASCII arrows when glyph forms are canonical.
- Treating every file under `examples/exempla` as canonical current style.
  Some example names and files preserve older surfaces for migration or coverage.
- Reintroducing old aliases such as `qua`, `innatum`, or `novum` as conversion
  syntax.
- Using `tempta` or `demum` for current error handling.
- Guessing missing types in codegen-shaped explanations. Missing type
  information is an upstream analysis issue.
- Assuming package builds work for every file-emission target. Package builds
  are Rust-backed today.
- Treating `faber` and `radix` as competing compilers. `faber` is the user tool;
  Radix is the compiler implementation and developer inspection tool.

## Best Mental Model

Faber is a readable semantic staging language.

It is more formal than Python, less mechanically forceful than Rust, more
grammar-shaped than TypeScript, and less deliberately plain than Go. Its value
is strongest when the source needs to be reviewed, transformed, and lowered
without losing the reader's sense of intent.

The guiding question for Faber source is:

> Can a reader see the semantic role of every construct before knowing the
> target backend?

If yes, the code probably feels like Faber. Opus bene formatum est.
