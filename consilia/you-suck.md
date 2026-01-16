# You Suck: A Comprehensive Critique of Faber

**Date**: 2026-01-15
**Author**: Claude Opus 4.5 (wearing "senior PL compiler engineer with a flair of GPT's autism" hat)
**Status**: Brutal honesty, as requested

---

## Executive Summary

Faber is a competent prototype that works for the happy path. It is not a production compiler. The Latin aesthetic is fine, but the claim that "Latin is pre-trained in LLMs" is unsupported by your own empirical data. The compiler implementation has architectural debt that will compound as features are added.

This document catalogs everything wrong, organized from foundational issues upward.

---

## Part I: Compiler Architecture

### Level 0: Fundamental Design Smells

#### 1. The Tokenizer Leaks Concerns

The tokenizer (`tokenizer/index.ts:598`) consults the lexicon to distinguish keywords from identifiers:

```typescript
if (isKeyword(value)) {
    const kw = getKeyword(value)!;
    addToken('KEYWORD', value, pos, kw.latin);
} else {
    addToken('IDENTIFIER', value, pos);
}
```

This is backwards. The tokenizer's job is to produce a stream of lexemes with minimal semantic knowledge. Keyword classification should happen in the parser or a pre-processing pass.

**Why this matters:**
- You've coupled lexical and syntactic phases
- Adding a new contextual keyword requires modifying the lexer
- You can't have identifiers that shadow keywords in different contexts

#### 2. The Lexicon is Dead Code

The `lexicon/` module implements Latin morphological parsing—noun declensions, verb conjugations, case analysis. Beautiful infrastructure.

The parser never uses any of it.

The tokenizer treats everything as either KEYWORD or IDENTIFIER. The morphological analysis is computed and then... discarded. Either:
- Delete it (if unused)
- Use it (if Latin morphology matters to semantics)

The current state is worst: maintained complexity with no payoff.

#### 3. No Formal Grammar

The EBNF in `EBNF.md` is documentation, not specification. There's no:
- Grammar file that generates parser tables
- Formal validation that parser matches grammar
- Mechanical extraction of first/follow sets

Every production is hand-coded. Grammar drift is inevitable.

---

### Level 1: Parser Architecture

#### 4. The Parser is a 1,400-Line Monolith

`parser/index.ts` defines a closure-based parser where everything is nested inside the `parse()` function. This creates:

- No testability of individual productions
- All state is captured in closure—can't inspect or reset
- Forward declarations via `let parseExpressionFn: () => Expression` then later assignment

The "resolver" pattern is a workaround for circular dependencies that shouldn't exist if module boundaries were correct.

#### 5. Precedence Climbing Isn't Visible

The EBNF shows a precedence hierarchy but the parser encodes it in function call structure, not data. Compare to a proper Pratt parser:

```typescript
const PRECEDENCE = {
    '=': 1, '||': 2, '&&': 3, '==': 4, // ...
};
```

Your precedence is implicit, scattered across files, and error-prone to modify.

#### 6. Error Recovery is Keyword-Based

`synchronize()` advances until hitting statement-starting keywords. This fails for:
- Expressions that span multiple lines
- Block contents that don't start with keywords
- Any construct where the error isn't near a keyword boundary

You'll skip valid code trying to find synchronization points.

---

### Level 2: Type System

#### 7. SemanticType is a Bag of Special Cases

`semantic/types.ts` defines 11 variants. But:

- `UserType` is "placeholder for future class/struct support"—it's used now
- `GenusType` stores fields/methods as parallel Maps—no structural sharing
- `DiscretioType` stores variants but no exhaustiveness info

The type equality function has nominal comparison by string name:

```typescript
if (a.kind === 'user') {
    if ((b.kind === 'enum' || b.kind === 'genus' || ...) && a.name === b.name) {
        return true;
    }
}
```

This breaks if two modules define `genus Foo` with different structures.

#### 8. No Proper Type Inference Algorithm

`isAssignableTo()` is ad-hoc:

```typescript
// Numeric type promotion: numerus -> fractus, numerus -> decimus
if (source.kind === 'primitive' && target.kind === 'primitive') {
    const numericTypes = ['numerus', 'fractus', 'decimus'];
    if (numericTypes.includes(source.name) && numericTypes.includes(target.name)) {
        return true;
    }
}
```

This says `fractus` is assignable to `numerus`, which loses precision silently. There's no coercion hierarchy, no unification, just a series of `if` statements.

#### 9. The Semantic Analyzer is 3,350 Lines

One file. No separation between:
- Type inference algorithm
- Type checking (compatibility validation)
- Module resolution
- Symbol table management
- Error recovery

The "three-phase analysis" (Predeclaration → Signature Resolution → Body Analysis) is a workaround for not having constraint-based type inference. You're manually ordering the work because the type system can't handle unordered constraints.

---

### Level 3: Code Generation

#### 10. Direct String Concatenation

The codegen returns strings via template interpolation. This is the most brittle approach:

- No intermediate representation means no optimization passes
- No pretty-printing abstraction—indentation is manual
- Escaping bugs are inevitable

A proper pipeline would build a target-language AST and serialize at the end.

#### 11. No Target Language Type Checking

You generate TypeScript and hope it compiles. There's no validation that:
- Generated identifiers are valid TS identifiers
- Generated types exist in the TS type system
- Generated code won't throw at runtime

---

### Level 4: Infrastructure

#### 12. 80+ Keywords is a Code Smell

Compare:
- Go: 25 keywords
- Rust: 39 keywords
- C: 32 keywords
- Faber: 80+

Four keywords just for variable binding (`varia`, `fixum`, `figendum`, `variandum`). Either many are contextual and shouldn't be reserved, or the language is feature-bloated.

#### 13. No Visitor Pattern

AST traversal is ad-hoc. Every phase that walks the tree implements its own traversal. There's no:
- `ASTVisitor<T>` interface
- `traverse(node, visitor)` function
- Transform infrastructure for rewrites

#### 14. YAML Tests are Stringly-Typed

```yaml
expect:
  ts: 'const x = (1 + 2);'
```

This tests exact string output. Changes to formatting break tests even if semantics are preserved. You need semantic equivalence testing.

#### 15. No Incremental Compilation

Every compilation reparses everything. The `resolvedModules: Map<string, Program>` in `SemanticResult` is batch-everything design.

---

## Part II: The Latin Hypothesis

### The Claim

> "Latin as a morphology (with children French/Spanish/etc) has vastly more training data points than TypeScript. I don't need to explicitly train an LLM on how to understand Latin keywords or conjugations. It just exists."

### Why This is Plausible

LLMs have ingested:
- Classical Latin (Caesar, Cicero, Virgil)
- Church Latin (Vulgate, liturgical texts)
- Academic Latin (scientific nomenclature, legal maxims)
- Romance languages (billions of documents)

So `redde` → "give back" is indeed pre-trained.

### Why This Doesn't Help

#### 1. Knowing Latin ≠ Knowing Faber Syntax

The LLM knows `redde` means "give back." It doesn't know:
- `redde` is a statement that takes an optional expression
- `redde` isn't a function call
- `redde` can't appear inside an expression

The *semantics* of Latin words are pre-trained. The *syntax* of combining them is not.

#### 2. Your Latin is Grammatically Incoherent

Your keywords mix:
- `fixum` — neuter singular perfect passive participle
- `varia` — feminine nominative plural
- `functio` — nominative singular noun
- `redde` — 2nd person singular present active imperative
- `si` — conjunction

This isn't Latin grammar. It's Latin *vocabulary* used as keywords. An LLM that knows Latin might be *more* confused—expecting agreement, conjugation, declension that doesn't exist.

#### 3. The Morphology Isn't Used

The lexicon parses declensions and conjugations. The parser requires exact keyword matches:

```typescript
if (checkKeyword('redde')) {
    return parseReddeStatement(resolver);
}
```

If Latin morphology mattered, you'd accept `reddit` (indicative) and `reddunt` (plural). You don't.

#### 4. The Training Distribution is Wrong

LLMs have seen Latin in contexts like:
> "Redde mihi librum" (Give back to me the book)

They have NOT seen:
```faber
functio summa(lista<numerus> numeri) -> numerus {
    redde totus
}
```

The statistical associations from Latin prose don't transfer to Latin-as-code.

---

## Part III: What Your Empirical Data Actually Shows

### The Read/Write Asymmetry

From `framework-1.1-results.md`:

| Direction | Accuracy |
|-----------|----------|
| faber_to_ts (reading) | 90-95% |
| ts_to_faber (writing) | 54-65% |

**Writing is 35-40 points worse than reading.**

If Latin were pre-trained, writing should be *easier*—you're emitting tokens the model has seen millions of times. But it's harder.

### The Real Explanation

1. **Models are fighting TypeScript priors.** The dominant failure (69%) is type errors—producing `x: number` instead of `numerus x`.

2. **Reading is forgiving.** When reading Faber, the model sees novel syntax and can infer meaning from context. The output is TypeScript, which it knows cold.

3. **Writing requires precision.** The model must produce exact tokens in exact order. TypeScript habits keep firing.

4. **Latin vocabulary is irrelevant.** The failure mode is *word order*, not vocabulary. Whether the type word is `numerus` or `number`, putting it in the wrong position fails.

### The Missing Ablation

Your thesis explicitly calls out:

> "Faber-English is strategically important because it removes Latin as a confound while keeping the grammar identical."

**You haven't run it.** All data is Faber-Latin only.

Until you have Faber-English results, you cannot claim Latin helps. Your current data supports:

- **H1 (structure/regularity):** The type-first, explicit-keyword syntax is what matters
- **H2 (Latin priors):** Unknown—untested

### What EBNF Success Really Means

> "Grammar-only context outperforms prose documentation — Formal EBNF grammar yields 92-98% accuracy"

This doesn't support Latin. EBNF works because *EBNF is code* and *models are good at code*. GitHub is full of grammar files. This has nothing to do with Latin.

---

## Part IV: What Would Actually Help

### 1. Run Faber-English Immediately

Same grammar, English keywords. If it matches Faber-Latin, your story is about *structure*. If it outperforms, Latin is *hurting* you.

### 2. Compare to Direct TypeScript Generation

The core claim is that generating Faber reduces errors vs generating TypeScript directly. Run the head-to-head comparison.

### 3. Analyze Keyword-Level Errors

Which Latin keywords are hardest to produce correctly? Prediction:
- High error: `secus`, `redde`, `discerne` (rare in any context)
- Low error: `si`, `et`, `non` (common Latin, common in code)

If confirmed, this supports "familiarity matters" not "Latin is magic."

### 4. Fix the Compiler Architecture

If you want Faber to scale:
- Extract the lexicon or delete it
- Add an IR between AST and target
- Implement proper bidirectional type checking
- Build target-language ASTs instead of string concatenation

### 5. Reframe the Marketing

The data supports:

> "Faber's explicit, type-first syntax reduces LLM errors compared to TypeScript's implicit, type-last syntax."

The data does not support:

> "Latin is in the training data so LLMs find it comfortable."

---

## Part V: The Verdict

### What Faber Is

A well-intentioned prototype with:
- Working tokenizer, parser, semantic analyzer, codegen
- Multi-target compilation (TS, and aspirationally Python/Rust/Zig/C++)
- A serious evaluation harness (rare for language projects)
- Reproducible experiments and honest documentation

### What Faber Isn't

A production-ready compiler. The architecture is "prototype that shipped"—adding generics, proper type inference, or incremental compilation would require significant rework.

### The Latin Question

The aesthetic is fine. Keep it if you like it. But the empirical case for "Latin helps LLMs" is:
- Theoretically plausible
- Empirically untested (no Faber-English ablation)
- Contradicted by the read/write asymmetry

The honest story is: "We built a regular, explicit syntax that might help LLMs. We're testing whether the Latin vocabulary specifically adds value."

### What's Actually Impressive

You built the infrastructure to falsify your own hypotheses. Most language projects never get past vibes. You have:
- 15,000+ trials
- 17 models
- Reproducible harness
- Honest result documentation

That's more rigor than most PhD theses. Use it.

---

## Appendix: Quick Reference of Issues

| Category | Issue | Severity |
|----------|-------|----------|
| Tokenizer | Keyword detection in lexer | Medium |
| Lexicon | Morphology code is dead | Low (delete or use) |
| Parser | 1,400-line monolith | Medium |
| Parser | Implicit precedence | Medium |
| Parser | Keyword-based sync | Low |
| Types | Ad-hoc assignability | High |
| Types | No unification | High |
| Semantic | 3,350-line monolith | Medium |
| Codegen | String concatenation | Medium |
| Codegen | No target validation | Low |
| Testing | String-based assertions | Low |
| Infra | No incremental compilation | Low (for now) |
| Latin | Hypothesis untested | **Critical** |
| Latin | Morphology unused | Medium |
| Latin | 80+ keywords | Low |

---

*"The code works. That's the problem—it works just well enough that you'll never fix it."*
