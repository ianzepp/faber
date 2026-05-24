---
title: Faber Romanus
description: An LLM-oriented intermediate representation that compiles to Zig, Rust, C++, TypeScript, or Python
order: 0
---

<div class="hero">

# Faber Romanus

<p class="tagline">The Roman Craftsman — An LLM-oriented intermediate representation that compiles to Zig, Rust, C++, TypeScript, or Python</p>

</div>

<div class="install-block">
Installation coming soon
</div>

<div class="links-block">

[GitHub](https://github.com/ianzepp/faber-romanus)

</div>

## The Problem

LLMs write code. Humans review it. But systems languages — Zig, Rust, C++ — are hard for both:

- **LLMs struggle with symbol-dense syntax.** Lifetimes, borrow checkers, template metaprogramming — these create semantic chaos that increases error rates.
- **Humans can't skim generated code.** Reviewing 50 lines of Rust requires understanding Rust. You can't verify "yes, that logic looks right" without parsing the syntax mentally.

You don't need an IR to generate TypeScript. You need one to generate Rust without lifetime annotation chaos.

## The Solution

Faber Romanus is an **intermediate representation** optimized for LLM generation and human review.

```faber
ex items pro item {
    si item.price > 100 {
        scribe item.name
    }
}
```

- **LLMs write Faber.** Word-based, regular syntax. No lifetime annotations, no pointer semantics, no template noise.
- **Humans skim Faber.** You see `si` (if), `pro` (for), `scribe` (print). You don't need to know Zig to verify the loop logic.
- **Compiler emits target code.** Zig, Rust, C++, TypeScript, or Python. The generated code is what actually runs.

The workflow: **LLM drafts Faber → Human approves → Compiler emits production code.**

## Why It Works

**No ecosystem problem.** Faber compiles to the target language, so you use its libraries directly. `ex hono importa Hono` becomes `import { Hono } from 'hono'`. No need to rewrite npm/PyPI/crates.io.

**Grammar designed for LLMs.** The EBNF specification is built for LLM consumption. Trials show models achieve 96-98% accuracy after reading the grammar specification alone — no prose documentation required.

**Regular structure.** Type-first declarations, consistent block patterns, no operator overloading. The regularity may matter more than the vocabulary.

**Semantic vocabulary.** Latin keywords encode intent: `fixum` (fixed/immutable) vs `varia` (variable/mutable), `cede` (yield/await), `redde` (give back/return).

## Example

```faber
functio salve(nomen) -> textus {
    redde "Salve, " + nomen + "!"
}

fixum nomen = "Mundus"
scribe salve(nomen)
```

Compiles to TypeScript:

```typescript
function salve(nomen): string {
    return 'Salve, ' + nomen + '!';
}

const nomen = 'Mundus';
console.log(salve(nomen));
```

## Links

- [Grammar Reference](/docs/grammar.html) - Complete language specification (EBNF)
- [Examples](/docs/examples.html) - Code examples by feature
- [Thesis and Trials](/research/) - LLM learnability evaluation
- [GitHub](https://github.com/ianzepp/faber-romanus) - Source code
