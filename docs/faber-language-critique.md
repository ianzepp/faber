# Faber Language Critique After Current Grammar And Active Compiler

This note captures a broad language-level critique formed after reading:

- `README.md`
- `EBNF.md`
- `explain/`
- `examples/exempla/`
- the active root Rust workspace under `crates/`, especially `crates/radix`

It deliberately excludes archived bootstrap and self-hosting material. Those
trees are not part of this repository's current compiler authority, command
surface, or CI contract.

This is not a compiler bug report and not a verdict against the project. The
goal is to name what Faber feels like, where it is strong, where it is weak, and
what the current language surface implies.

## Short Version

Faber is no longer best described as a loose syntax experiment or target-language
staging notation. The current repo presents a more disciplined language:

- type-first declarations,
- glyph-forward structural operators,
- Latin behavioral vocabulary,
- an active Radix compiler,
- a Faber package/project CLI,
- a validated MIR inspection branch,
- a canonical Faber pretty-printer,
- explicit target-support boundaries.

The central problem is no longer:

- can Faber express enough?

It is now more like:

- can Faber keep a small, canonical semantic center as the grammar, examples,
  stdlib metadata, MIR, package tooling, and target backends evolve?

That is the heart of the critique.

## How Faber Feels Versus Python And Rust

### One-line comparison

- Python: just say it.
- Rust: prove it.
- Faber: state it clearly.

### Slightly longer comparison

- Python feels permissive, social, and idiomatic.
- Rust feels contractual, mechanical, and constraint-forward.
- Faber feels declarative, narrated, and review-oriented.

### Python

Python assumes shared human context. It is happy to lean on convention, library
familiarity, and the reader's ability to infer intent.

Strengths of the Python feel:

- quick to improvise in,
- easy to use for glue code,
- culturally dense and expressive,
- low ceremony for common tasks.

Weaknesses of the Python feel:

- meaning often depends on convention and context,
- hidden complexity can accumulate quietly,
- readability and correctness are often social achievements rather than
  syntactic ones.

### Rust

Rust insists that semantic relationships matter. Ownership, borrowing,
mutability, exhaustiveness, trait boundaries, and failure modes are pushed into
the source.

Strengths of the Rust feel:

- hard to ignore important invariants,
- strong semantic pressure,
- explicit relationship to runtime behavior,
- high trust when code is correct.

Weaknesses of the Rust feel:

- symbolic density,
- high authoring friction,
- substantial review burden even when code is good.

### Faber

Faber feels like an explicit review layer. The syntax tries to keep semantic
categories visible while reducing target-language-specific ceremony.

Examples of that feel:

- `si cond ∴ redde value`
- `itera ex items fixum item { ... }`
- `tempta ... cape ... demum`
- `Point { x = 10, y = 20 }`
- `fixum lista<numerus> xs ← vacua`
- `value ∷ textus`
- `input ⇒ numerus vel 0`

Strengths of the Faber feel:

- regular structure,
- intent-forward syntax,
- relatively low punctuation noise,
- good skimmability in small and medium examples,
- good reviewer-facing rhythm,
- clear separation between runtime conversion (`⇒`) and static type ascription
  (`∷`).

Weaknesses of the Faber feel:

- less fluid than Python,
- less semantically forceful than Rust,
- can sit in an awkward middle layer,
- can look cleaner than the downstream target/runtime reality actually is.

### The clearest summary

- Python: I can probably guess what this does.
- Rust: I can prove what this does.
- Faber: I can see what this means.

That remains the best concise description of the difference in feel.

## What Has Improved In The Current Surface

### 1. The construction/ascription split is much cleaner

Older critiques of Faber had to contend with `⇢` doing too many jobs. The
current language no longer has that problem in the same form.

The current split is clearer:

- typed construction uses the type before braces: `Point { x = 10 }`,
- empty collections use typed declaration plus `vacua`,
- static type ascription uses `∷`,
- runtime conversion uses `⇒`.

That removes a major source of visual and semantic confusion. In particular,
static type ascription no longer fights the leftward binding arrow with a
rightward cast arrow, and ordinary construction no longer overloads the same
surface.

### 2. Canonical forms are sharper

The current docs are more direct about canon:

- `∴` is canonical, while `ergo` remains accepted.
- `∷` is the only postfix static type-ascription operator.
- Old aliases such as `qua`, `innatum`, and `novum` are retired as conversion or
  construction syntax.
- `sponte` is a post-name declaration marker, not a nullable type marker.
- Nullable value types use `T ∪ nihil`.
- `ignotum` is an unknown/escape type, not nullability.
- Package builds are Rust-backed today; other targets are file-emission
  surfaces unless documented otherwise.

This is a healthier language posture. It is not just adding features; it is also
cutting old ambiguity.

### 3. The active authority surface is clearer

The active compiler authority is Radix in the root Rust workspace. The user tool
is Faber. The stdlib source is `stdlib/norma`. Runtime support lives in
`crates/norma`. The grammar and explain corpus have clearer roles.

That matters because small languages are vulnerable to authority drift. Faber is
in better shape when a reader can answer:

- what is the grammar?
- what is user-facing documentation?
- what is compiler implementation?
- what is package tooling?
- what is historical material?

The current repo answers those questions more cleanly than older project shapes.

### 4. MIR gives the language a better future execution story

The validated MIR inspection branch does not make MIR the production backend,
and the Rust MIR probe is intentionally temporary. Still, MIR is important
language infrastructure.

It gives Faber a place to make control flow, storage, runtime calls, aggregate
construction, nullable operations, and target-neutral execution shape explicit
below typed HIR. That reduces pressure on target backends to independently
rediscover language semantics.

This strengthens Faber's claim to be more than a pretty source-to-Rust surface.

## Core Weaknesses In Faber

### 1. It can be clear-but-blunt instead of clear-and-sharp

Faber is very good at making semantic roles visible.

But visibility can come at the cost of sharpness. A Faber program can look
settled while still hiding important downstream facts:

- runtime cost,
- ownership consequences,
- representation details,
- target-specific semantics,
- performance-sensitive structure,
- runtime/HAL availability.

This is especially important when the value proposition points toward executable
programs, CLI tools, and eventually WASM or lower-level targets.

### 2. It is weaker at expressing concrete downstream truth than Rust

Rust forces many realities into the source:

- ownership,
- borrowing,
- exact failure forms,
- representation and trait boundaries,
- exhaustiveness and mutability distinctions.

Faber can describe intent very well, but it does not force all of those truths
to stay equally visible.

That makes it a better review layer in one sense, but a weaker semantic pressure
system in another.

### 3. It lives in an awkward middle layer

Faber is:

- less fluid than Python,
- less semantically forceful than Rust.

That gives it a strange tradeoff profile. It can be too formal for quick
scripting and too gentle for low-level truth.

That does not make it useless. It means its best role is narrower than a normal
general-purpose language: clear CLI tools, typed transformation logic, package
entrypoints, compiler-shaped code, and backend-aware programs where reviewable
source is valuable.

### 4. The Latin vocabulary is both asset and tax

The Latin does useful work as a semantic namespace, but it still costs real
human adaptation effort.

Some terms buy real clarity:

- `functio`,
- `genus`,
- `pactum`,
- `redde`,
- `cape`,
- `discerne`,
- `custodi`.

Others are learnable mainly by project convention. The fairest version is:

- the Latin is not merely ornamental,
- but some of its cognitive cost does not directly purchase semantic precision.

### 5. The language still needs strong anti-sprawl discipline

Faber has made progress by retiring old aliases and narrowing canonical syntax.
That discipline needs to continue.

Surfaces to watch:

- symbolic and word aliases (`∴`/`ergo`),
- CLI annotations versus ordinary source syntax,
- stdlib morphology metadata versus user syntax,
- target-specific `@ verte` translation metadata,
- package/build behavior versus single-file compiler behavior,
- target support claims.

A language optimized for learnability and regularity should be suspicious of
parallel forms. Every tolerated alternative weakens the canonical center unless
it has a durable reason to exist.

### 6. Target transparency is incomplete in practice

The docs are honest that not all targets support all features. That honesty is
good.

But it still means Faber is not one simple portable thing. It is a source
language with several backend realities:

- Rust is the primary executable/package backend.
- Faber is a canonical pretty-printer and round-trip target.
- TypeScript and Go currently support file emission, not full package builds.
- MIR is validated and inspectable, but not the default production backend.
- Future WASM work remains a plan, not current support.

That is normal in language projects, but it weakens any claim that the language
is one coherent cross-target truth in practice.

### 7. Canon drift is especially dangerous for Faber

Because Faber's value depends heavily on readability, explicitness, and
regularity, stale examples and mixed authority surfaces are especially damaging.

If examples, grammar, explain docs, stdlib metadata, and compiler behavior drift
apart, the project loses exactly the thing it is trying to build:

- machine-legible canon,
- human-reviewable semantic clarity,
- stable cross-surface understanding.

The project has made real cleanup progress, but that also raises the bar:
current docs must stay current, and historical docs must stay clearly
historical.

### 8. It may under-serve messy reality programming

Python thrives in glue code and irregular practical seams. Rust thrives in
high-constraint, high-trust engineering.

Faber may be least comfortable when the work is:

- messy,
- target-specific,
- stateful in ugly ways,
- full of host interop detail,
- full of practical shortcuts that are hard to normalize cleanly.

That does not mean Faber cannot express such code. It means the language may not
be in its natural habitat there.

## What The Current Project Reveals About Faber's Design Philosophy

### 1. Faber is strongest as reviewed semantic source

The language feels best when used to describe:

- control flow,
- transformation logic,
- moderate abstraction,
- compiler and analysis structure,
- package entrypoints,
- intent before target-specific lowering.

### 2. Faber is a governed language platform, not just a grammar

The real language includes:

- grammar,
- examples,
- explain docs,
- active compiler behavior,
- stdlib metadata,
- package tooling,
- target support policy,
- MIR and future execution planning.

That makes it less like a pure notation and more like a governed platform.

### 3. The current active/historical split is a strength

The README is right to separate current workspace authority from archived
historical material. That split should remain strict.

Historical code can inform design judgment, but it should not define current
language truth.

## Revised Position

### Earlier position

Faber seemed elegant, interesting, and perhaps too gentle, too alias-prone, or
too middle-layer to carry large-scale reality comfortably.

### Current position

Faber is more substantial and more disciplined than that framing gave it credit
for.

The current language has improved by cutting overloaded construction/ascription
syntax, sharpening canonical forms, clarifying compiler authority, and adding a
validated MIR inspection layer.

But that stronger legitimacy shifts the main danger.

The real danger is no longer that Faber cannot express enough. The real danger
is that scaling it requires unusually strict control over:

- semantic scope,
- canonical forms,
- active authority surfaces,
- example freshness,
- target honesty,
- implementation truth.

In short:

Faber is more real than a toy language critique would suggest, but exactly
because it is more real, its main problem becomes governance rather than syntax.

## Condensed Critique

### Criticisms to soften

- It is only good for toy examples.
- The Latin is mostly decorative.
- The type/construction surface is inherently confused.

### Criticisms to strengthen

- The language must aggressively avoid overlapping surfaces.
- It can make complexity look tamer than it is.
- Portability is less unified in practice than the surface language implies.
- Canon drift is uniquely damaging because Faber's value depends on clarity.

### Main criticism

The central challenge is semantic governance, not syntax beauty.

## Best One-Sentence Version

Faber is a serious review-oriented language surface with a clearer current
canon than before, but its long-term strength depends on keeping one disciplined
semantic truth as syntax, tooling, stdlib metadata, MIR, and target backends
evolve.
