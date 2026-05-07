# Faber Language Critique After Grammar, Exempla, And Rivus

This note captures a broad language-level critique formed after reading:

- `EBNF.md`
- `examples/exempla/`
- `compilers/rivus/`

It is not a compiler bug report and not a verdict against the project. The goal is to name what Faber feels like, where it is strong, where it is weak, and what changed after looking beyond the clean examples into the self-hosted compiler tree.

## Short Version

Faber is more serious and more capable than the clean examples alone suggest. It can clearly support real compiler-shaped software, not just toy snippets.

But that stronger legitimacy changes the main criticism.

The central problem is no longer:

- can Faber express enough?

It is now more like:

- can Faber maintain one canonical semantic center as the language, examples, targets, and compiler implementations evolve?

That is the heart of the critique.

---

## How Faber Feels Versus Python And Rust

### One-line comparison

- Python: just say it
- Rust: prove it
- Faber: state it clearly

### Slightly longer comparison

- Python feels permissive, social, and idiomatic.
- Rust feels contractual, mechanical, and constraint-forward.
- Faber feels declarative, narrated, and review-oriented.

### Python

Python feels like a language that assumes shared human context. It is happy to lean on convention, library familiarity, and the reader's ability to interpolate intent.

Strengths of the Python feel:

- quick to improvise in
- easy to use for glue code
- culturally dense and expressive
- low ceremony for common tasks

Weaknesses of the Python feel:

- meaning often depends on convention and context
- hidden complexity can accumulate quietly
- readability and correctness are often social achievements rather than syntactic ones

### Rust

Rust feels like a language that insists that semantic relationships matter. Ownership, borrowing, mutability, exhaustiveness, trait boundaries, and failure modes are pushed into the surface.

Strengths of the Rust feel:

- hard to ignore important invariants
- strong semantic pressure
- explicit relationship to runtime behavior
- high trust when code is correct

Weaknesses of the Rust feel:

- symbolic density
- high authoring friction
- review burden is substantial even when code is good

### Faber

Faber feels like an explicit review layer. The syntax tries to keep semantic categories visible while reducing punctuation noise and target-language-specific ceremony.

Examples of that feel:

- `si ...`
- `itera ex ... fixum item { ... }`
- `custodi { ... }`
- `tempta ... cape ... demum`
- `{ ... } ⇢ Type`
- `input ⇒ numerus vel 0`

Strengths of the Faber feel:

- regular structure
- intent-forward syntax
- relatively low symbol noise
- good skimmability in small and medium examples
- good reviewer-facing rhythm

Weaknesses of the Faber feel:

- less fluid than Python
- less semantically forceful than Rust
- can sit in an awkward middle layer
- can look cleaner than the downstream reality actually is

### The clearest summary

- Python: I can probably guess what this does.
- Rust: I can prove what this does.
- Faber: I can see what this means.

That is the best concise description of the difference in feel.

---

## Early Critique Before Looking At Rivus

Before reading `compilers/rivus/`, the main concerns were:

1. Faber might work best only in curated examples.
2. The Latin vocabulary might be more ornamental than necessary.
3. The language might be too semantically gentle for systems-language targets.
4. There might be too much alias and syntax-surface pressure.
5. The language might be strongest as a staging layer, but weak as a large-scale working language.

Reading Rivus changed some of that and strengthened some of it.

---

## What Changed After Looking At Rivus

### Criticisms that got weaker

#### 1. "Maybe Faber only works in toy examples"

This got weaker.

`compilers/rivus/` is a real compiler-shaped codebase written in Faber. It has:

- AST definitions
- lexer and parser phases
- parser mutual-recursion control via a pactum seam
- semantic analysis
- module resolution
- multiple codegen targets
- validation surfaces

That means Faber can carry substantial modular software, not just polished demo code.

#### 2. "The Latin is mostly decorative"

This also got weaker.

The Latin still imposes a learning cost, but once the language is used across a real compiler tree, the vocabulary starts to feel less like branding and more like semantic namespace partitioning.

Examples from Rivus:

- `sententia`
- `expressia`
- `resolvitor`
- `morphologia`
- `discretio`
- `cura`
- `custodi`

At that point the vocabulary is doing real conceptual work.

The cost remains real, but the benefit is also more real than it first appears.

### Criticisms that got stronger

#### 1. Alias and surface sprawl

This got much stronger.

The grammar, examples, and compiler trees together show a language with many overlapping surfaces:

- word forms and symbolic forms
- old and new naming directions
- user-facing syntax plus CLI annotation language plus stdlib morphology language
- target-portable ideals plus target-specific capability realities
- multiple compilers with different degrees of authority

Any one of those is manageable. Taken together, they create a real governance problem.

The threat is not that the grammar is unreadable. The threat is that the language becomes too easy to spread semantically across:

- aliases
- historical forms
- examples that teach older subsets
- different compiler truth surfaces
- target-specific capability envelopes

This is the strongest language-level criticism.

#### 2. Faber can make hard programs look cleaner than they are

This got stronger too.

Originally this criticism mostly meant: Faber may hide the harshness of Rust, Zig, or other target-level semantics.

After Rivus, there is a second layer:

Faber can also make compiler architecture itself look calmer and more settled than it is.

Rivus has real complexity:

- giant files
- capability matrices
- manual host shims
- partial target stories
- evolving syntax eras
- target divergence machinery

The code is often readable, but readability can create a false impression of global simplicity.

That is not a criticism of readability. It is a warning that readability can mask real architectural burden.

#### 3. Target divergence is more serious than it first appears

This got much stronger.

The living language is not just one grammar. It is really something like:

- a formal grammar in `EBNF.md`
- a worked example corpus in `examples/exempla/`
- a primary active compiler in `compilers/radix-rs`
- a self-hosted experimental compiler in `compilers/rivus`
- multiple target backends with uneven support
- capability validation and target policy surfaces

That means portability is not merely a parsing or codegen problem. It becomes a governance problem.

A language can claim one syntax and still have multiple lived truths:

- what the grammar allows
n- what the examples normalize
- what the active compiler guarantees
- what the experimental compiler still supports
- what each target can honestly carry

Faber is close enough to that line that this must be treated as a core design risk.

---

## New Main Criticism: Semantic Governance

This is the biggest update after reading Rivus.

Faber's hardest problem is not syntax beauty and not expressiveness. It is semantic governance.

The key questions are:

- what syntax is canonical versus merely tolerated?
- which compiler defines the public truth?
- which examples are current and which are historical residue?
- which target promises are real and which are aspirational?
- which aliases stay and which should be cut?
- which surfaces are pedagogical and which are contractual?

A mainstream language gets some help here from ecosystem gravity. Faber does not.

So the danger is not generic "custom language risk." It is more specific:

Faber needs unusually strong canon discipline to avoid becoming a beautiful but drifting semantic federation.

That is the new center of the critique.

---

## Core Weaknesses In Faber

### 1. It can be clear-but-blunt instead of clear-and-sharp

Faber is very good at making semantic roles visible.

But that visibility can come at the cost of sharpness.

A Faber program can look semantically settled while still hiding important downstream facts:

- runtime cost
- ownership consequences
- representation details
- target-specific semantics
- performance-sensitive structure

This is especially important when the value proposition points toward systems-language targets.

### 2. It is weaker at expressing concrete downstream truth than Rust

Rust forces many realities into the source:

- ownership
- borrowing
- exact failure forms
- representation and trait boundaries
- exhaustiveness and mutability distinctions

Faber can describe intent very well, but it does not force those truths to stay equally visible.

That makes it a better review layer in one sense, but a weaker semantic pressure system in another.

### 3. It lives in an awkward middle layer

Faber is:

- less fluid than Python
- less semantically forceful than Rust

That gives it a strange tradeoff profile.

It can be too formal for quick scripting and too gentle for low-level truth.

That does not make it useless. It just means its best role is narrower than a normal general-purpose language.

### 4. The Latin vocabulary is both asset and tax

The Latin does useful work as a semantic namespace, but it still costs real human adaptation effort.

Not every keyword carries equal intuitive value on first encounter. Some are excellent. Some are merely consistent.

So the fairest version is:

- the Latin is not merely ornamental,
- but some of its cognitive cost does not directly purchase semantic precision.

### 5. The language wants regularity but still carries too much parallel surface

This includes things like:

- `⇢`, `qua`, `innatum`, `novum`
- `⇒` plus named conversion shorthands
- symbolic versus wordy variants
- alternative range notations
- compact one-line sugar versus block forms

A language optimized for learnability and regularity should be deeply suspicious of redundant parallel forms.

Every tolerated alternative weakens the canonical center.

### 6. Some constructs carry too much semantic load

`⇢` is the clearest example.

Across docs and examples it is doing several jobs:

- compile-time cast
- native collection construction
- genus or struct instantiation
- territory inherited from earlier terms like `innatum` and `novum`

That is elegant at one level, but it also risks becoming a semantic junk drawer.

If one surface operator does too many conceptually different things, local clarity suffers.

### 7. Target transparency is incomplete in practice

The docs are honest that not all targets support all features. That honesty is good.

But it still means the language is not one simple stable portable thing. It is a surface language with multiple backend realities.

That is common in language projects, but it weakens the strength of any claim that the language itself is one coherent cross-target truth.

### 8. Canon drift is more dangerous for Faber than for mainstream languages

Because the language's value depends heavily on readability, explicitness, and regularity, stale examples and mixed authority surfaces are especially damaging.

If examples, grammar, and compilers drift apart, the project loses exactly the thing it is trying to build:

- machine-legible canon
- human-reviewable semantic clarity
- stable cross-surface understanding

### 9. It may under-serve messy reality programming

Python thrives in glue code and irregular practical seams.
Rust thrives in high-constraint, high-trust engineering.

Faber may be least comfortable when the work is:

- messy
- target-specific
- stateful in ugly ways
- full of host interop detail
- full of practical shortcuts that are hard to cleanly normalize

That does not mean it cannot express such code. It means the language may not be in its natural habitat there.

---

## What Rivus Reveals About Faber's Design Philosophy

Rivus makes a few things clearer.

### 1. Faber is not merely a syntax experiment

Rivus proves that the language can carry architectural structure. That is important.

### 2. Faber is at its strongest as a semantic staging language

The language feels best when used to describe:

- control flow
- transformation logic
- moderate abstraction
- compiler and analysis structure
- reviewed intent before target-specific lowering

### 3. The language project is broader than the grammar

The real language includes:

- grammar
- examples
- compiler implementations
- codegen conventions
- capability validation
- target support policy

That makes it less like a pure notation and more like a governed language platform.

### 4. Self-hosting raises the stakes

Once the language hosts a real compiler, the burden of authority becomes much higher.

It is no longer enough for the syntax to be elegant. The project must decide what counts as living truth.

---

## Revised Position

### Earlier position

Faber seemed elegant, interesting, and perhaps too gentle, too alias-prone, or too middle-layer to carry large-scale reality comfortably.

### Revised position

Faber is more substantial and more legitimate as a real language than that earlier framing gave it credit for.

But that stronger legitimacy shifts the main danger.

The real danger is no longer that it cannot scale. The real danger is that scaling it requires unusually strict control over:

- semantic scope
- canonical forms
- active authority surfaces
- example freshness
- target honesty
- implementation truth

In short:

Faber is more real than a toy language critique would suggest, but exactly because it is more real, its main problem becomes governance rather than syntax.

---

## Condensed Critique

### Criticisms to soften

- It is only good for toy examples.
- The Latin is mostly decorative.

### Criticisms to strengthen

- The language carries too many overlapping surfaces.
- It can make complexity look tamer than it is.
- Portability is less unified in practice than the surface language implies.

### Main new criticism

The central challenge is semantic governance, not syntax design.

---

## Best One-Sentence Version

Faber is more legitimate as a substantial language than the clean examples alone suggest, but that legitimacy shifts the main danger from "can it express enough?" to "can it keep one canonical truth as the language and its compilers evolve?"
