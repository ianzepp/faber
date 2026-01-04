# Faber Romanus: Research-Grade Rigor Without a PhD

**Date**: 2026-01-03
**Status**: Engineering strategy document (not pursuing PhD)

## Purpose

This document is not a PhD proposal.

It is a blueprint for applying **PhD-grade rigor** (clear claims, controlled comparisons, reproducible evaluation) to:

- Improve the language and compiler via measurable feedback loops
- Strengthen the project’s positioning beyond novelty
- Produce artifacts that read like serious research engineering, even outside academia

The goal is to move the narrative from:

> “TypeScript in a toga”

to:

> “A human-legible, machine-checkable intermediate language optimized for LLM→human→compiler workflows, backed by reproducible evaluation and multi-target compilation.”

## Core Reframing

### What This Project Is Not

- Not “a Latin programming language” as the core contribution
- Not a purely aesthetic or meme-driven syntax experiment
- Not a claim about AI intelligence

### What This Project Is

**Faber Romanus is an intermediate representation (IR) that is:**

- **LLM-friendly to write** (regular, low-entropy structure)
- **Human-friendly to skim** (word-based, explicit intent)
- **Compiler-friendly to validate** (deterministic grammar + strong diagnostics)
- **Multi-target by construction** (single AST, multiple backends)

The central workflow claim is:

> LLM drafts in Faber → human approves/skims → compiler emits production targets.

Even if the keyword vocabulary were changed tomorrow, the project remains valuable if the IR/harness/compiler methodology produces measurable improvements.

## Why Latin (Positioning, Not Mysticism)

Latin is not chosen for aesthetics. It is chosen as a plausible **high-frequency linguistic substrate** for Western web-trained models:

- Latin roots permeate English technical vocabulary
- Romance languages share morphology and word roots with Latin
- Legal, medical, scientific writing contains Latin fragments

This is a _hypothesis about priors_, not a belief.

If Latin provides a measurable advantage, it should be demonstrable with ablations and baselines. If it doesn’t, the project can still succeed on the “low-entropy IR” thesis.

## The “Rigor Loop” (How This Improves the Language)

PhD-grade rigor is a development method:

1. **State a claim** precisely (what improves, relative to what).
2. **Define metrics** that are externally checkable.
3. **Build a reproducible harness** to measure those metrics.
4. **Run baselines and ablations** to isolate causes.
5. **Use the results to drive implementation** (compiler features, diagnostics, grammar constraints).
6. **Repeat**, keeping historical results to detect regressions.

This makes language evolution measurable rather than aesthetic.

## Claims (Keep It Small and Falsifiable)

A strong strategy is to pick **2–3 primary claims** and treat everything else as supportive evidence.

### Claim A: LLM generation reliability improves

**Claim**: For a fixed task suite and prompting protocol, generating Faber yields lower failure rates than generating target languages directly.

**Primary metrics** (externally scored):

- Parse success rate
- Typecheck success rate
- End-to-end success rate (tests pass)
- Repair cost (number of edit iterations to pass)
- Token cost per passing solution

**Baselines** (minimum):

- Direct TypeScript (or Rust) generation
- Faber → TypeScript (or Rust) compilation

**Ablations** (to isolate mechanism):

- Faber-English (same grammar, English keywords)
- Faber-Symbols (same structure, symbolic keywords)

If Faber-English ≈ Faber-Latin, then the mechanism is primarily structure/regularity. That is still valuable; the narrative shifts accordingly.

### Claim B: Reviewability improves (human or AI)

**Claim**: Under fixed budget constraints (time for humans, token budget for AI), reviewing Faber yields higher bug recall / lower false positives than reviewing target language equivalents.

**Primary metrics**:

- Bug recall on seeded defects
- False positive rate
- Time-to-decision (humans)
- Token-to-decision (AI)

**Important constraint**: review must be graded against a labeled bug corpus. No self-judged “looks correct” scoring.

### Claim C: Multi-target compilation preserves semantics for a defined subset

**Claim**: For an explicitly defined subset of the language, code generated for N targets preserves semantics as validated by a shared test suite.

**Primary metrics**:

- Cross-target test pass rate
- Feature coverage (what subset is included)
- Determinism (same input → same output)

This claim builds compiler credibility, which is a major part of reputation.

## What Makes This “Research-Grade”

### 1) Reproducibility First

The project becomes reputable when third parties can rerun the same evaluation and see similar outcomes.

- Fixed harness
- Logged prompts
- Versioned task suite
- Deterministic formatting
- Stable diagnostics
- Clear failure taxonomy

### 2) External Scoring (Not Vibes)

Where possible, evaluate via:

- Compiler success/failure
- Typechecking
- Tests
- Runtime behavior

Prefer “the program passed” over “the model said it passed.”

### 3) Baselines You Control

To avoid the “TypeScript in a toga” critique, comparisons must separate:

- surface vocabulary vs structure
- structure vs toolchain feedback
- IR merits vs backend quality

Faber-English is strategically important because it removes Latin as a confound while keeping the grammar identical.

### 4) Confounds to Track Explicitly

If you want the results to survive skeptical readers, track these as covariates:

- Tokenization (tokens per program)
- Prompt length and format
- Allowed tools/feedback loops (compilers, error messages)
- Model versions (silent updates happen)

Treat “temperature=0” as “reduced randomness,” not absolute determinism.

## The Evaluation Harness (Practical Design)

The most valuable artifact (for both engineering and reputation) is a harness that can:

- Generate or accept Faber source
- Compile with `bun run faber compile`
- Compile to multiple targets
- Execute target test suites
- Score outcomes with a consistent rubric

### Suggested outcome taxonomy

Every run should end in exactly one bucket:

- `tokenize_error`
- `parse_error`
- `semantic_error` (type errors, name resolution, etc.)
- `codegen_error` (backend failure)
- `runtime_error`
- `test_failure`
- `success`

This taxonomy directly drives compiler work: improvements are measured by moving failures “left” (runtime → compile-time) and decreasing total failures.

## Evidence That Improves Implementation

Rigor is not just for positioning; it directly improves the language:

- If parse errors dominate, improve grammar constraints and diagnostics.
- If semantic errors dominate, improve type system, name resolution, error messages.
- If codegen errors dominate, tighten AST invariants and backend conformance.
- If runtime/test failures dominate, improve standard library + semantic checks.

This also produces a clean roadmap driven by data rather than taste.

## Who Is the Audience?

The document implicitly assumes humans need to be convinced—hence the focus on positioning, narrative, and "how to talk about it." But if the workflow is genuinely "LLM drafts → human skims → compiler emits," the human's role shrinks to approval gate, not authorship.

The real question: **who chooses the IR?**

### Short Term (Now – 2-3 Years): Human Developers

Today, humans choose the language, framework, and toolchain. LLMs comply with whatever is requested. Marketing must convince human decision-makers that Faber is worth adopting.

**Go-to-market**: Developer advocacy, benchmarks, technical reports, conference talks.

### Medium Term: Tool and Agent Builders

Agentic coding tools (Claude Code, Cursor, Devin, Codex successors) increasingly make tactical decisions—which files to edit, which patterns to use. The jump to "which language to draft in" is not far.

If Faber demonstrably reduces LLM error rates, agentic systems will adopt it as an internal representation—possibly without end users ever seeing `.fab` files. The IR becomes infrastructure, not interface.

**Go-to-market**: Integration with agent frameworks, SDK/API access, partnerships with tool vendors.

### Long Term (Speculative): LLMs as Autonomous Selectors

Do LLMs, given freedom to choose, gravitate toward lower-entropy representations? If an LLM could choose to draft in a language where it makes fewer errors and needs fewer repair cycles, why wouldn't it?

The "comfortable" hypothesis becomes testable: measure whether models, when given a choice, prefer Faber over alternatives. If yes, the market _is_ the models.

**Go-to-market**: Publish preference studies; position Faber as the "native" format for AI-generated code.

### Implications

- **Claims A, B, C remain valid across all audiences.** The metrics don't change; the narrative emphasis does.
- **The Latin question gets more interesting.** If LLMs genuinely find Latin roots more "comfortable" (lower perplexity, fewer tokenization artifacts), they might prefer Faber even without being told to. This is empirically testable once the harness exists.
- **The approval gate is the bottleneck.** If humans can skim Faber faster than TypeScript (Claim B), and the compiler guarantees correctness (Claim C), the human has less reason to care what the IR looks like. "I don't understand it, but it compiles and passes tests" is already how most people treat generated code.

## Positioning (How to Talk About It)

### Don’t lead with

- “Latin programming language”
- “over-represented substrate” (as a claim without measurements)
- “unfair advantage” (fun, but reads like marketing)

### Lead with

- “LLM-oriented intermediate representation with deterministic compilation”
- “multi-target compiler”
- “measured improvements in compilation success / test pass rate”
- “reproducible benchmark harness”

### Optional secondary story

Once measured:

- “Vocabulary alignment with high-frequency linguistic priors may help reliability”

Only elevate this if the ablations support it.

## Deliverables (Reputation Comes From Artifacts)

If you want non-academic credibility, produce things that look like serious engineering research:

- A “claims + metrics” one-pager
- A benchmark suite (small, curated at first)
- A harness to run it reproducibly
- A short technical report (8–20 pages) with results and limitations
- A public dashboard-like summary (even a markdown table) tracking regressions

## A Practical Roadmap

### Phase 1: Define the claims (days)

- Choose 2–3 claims.
- Write the scoring rubric.
- Decide baselines and ablations.

### Phase 2: Build the harness (1–3 weeks)

- Automate compilation + testing for at least one target.
- Log every input/output.
- Implement the failure taxonomy.

### Phase 3: Run small pilots (days)

- 10–30 tasks.
- 2–3 model families if possible.
- Identify dominant failure modes.

### Phase 4: Improve implementation using data (ongoing)

- Fix the most common failure class.
- Re-run.
- Track regressions.

### Phase 5: Scale tasks + publish results (ongoing)

- Grow to 100+ tasks.
- Add more targets.
- Publish the technical report.

## Notes on “AI Participants”

Using AI participants is aligned with the actual workflow being optimized: LLMs produce the drafts.

However:

- AI-based evaluation must be externally scored (compiler/tests).
- Human studies can be minimal and still useful: validate that the IR is skimmable enough for approval gates.

The hybrid approach is pragmatic:

- AI runs provide scale and reproducibility.
- Small human validation ensures the “approval gate” assumption is real.

## Bottom Line

This is a path to build a reputable language project without academia:

- Make a small number of falsifiable claims.
- Build a reproducible harness.
- Measure baselines and ablations.
- Use the data to drive language/compiler changes.

If you do this, the project stops being “a cute syntax” and becomes a serious, repeatable system: an LLM-oriented IR + compiler with evidence.

## Gemini 3 Pro: Notes

### 1. The Strategic Pivot is Correct

Moving the definition of Faber Romanus from a "Language" to an "**LLM-Native Intermediate Representation (IR)**" is the strongest possible move. It immediately neutralizes the "why would I learn this?" objection.

- **Verdict:** Keep this central. The claim "LLM drafts → Human audits → Compiler builds" defines a distinct market slot that existing languages (designed for humans first) cannot easily occupy.

### 2. The "Faber-English" Baseline is Critical

The section on **Ablations** (Claim A) is the intellectual anchor of this entire plan.

- **Why:** Without `Faber-English` (same grammar, English keywords), critics will always conflate the _structure_ of the language with the _vocabulary_.
- **Recommendation:** Prioritize the implementation of the `Faber-English` keyword mapping in the compiler. It allows you to prove whether the "Latin advantage" is real or if the gains come solely from the rigid, type-first grammar.

### 3. Gap Analysis: The "Code Distribution" Problem

In **"Why Latin"**, the hypothesis rests on Latin being a "high-frequency linguistic substrate."

- **The Risk:** While base models (GPT-4, Claude) know Latin, their _coding_ capabilities are fine-tuned on GitHub (Python, TS, Rust). Latin syntax might be **out-of-distribution** for the specific "coding heads."
- **Refinement:** Control for "In-Context Learning" (ICL). Distinguish between **Zero-shot** (testing priors) and **Few-shot** (testing learnability/regularity). Latin might lose on Zero-shot but win on Few-shot due to lower token ambiguity.

### 4. Claim B (Reviewability) is the Hardest to Prove

**Claim B** (human review) is the weakest link because "time-to-decision" varies wildly by human skill.

- **Suggestion:** Narrow this claim to **"Ambiguity Reduction"**. Can a human correctly predict runtime behavior? (e.g., "What is the type of variable X?"). This is objectively measurable without complex user studies.

### 5. The Confound of Compiler Diagnostics

In **Claim A**, there is a hidden variable: **Error Message Quality**.

- **The Problem:** If the compiler gives poor error messages, the "Repair cost" metric will tank, regardless of language design.
- **Recommendation:** Treat "Diagnostic Quality" as a prerequisite for benchmarks.

### Summary

**Top Action Items:**

1.  **Build the Harness First:** The "product" is the benchmark harness.
2.  **Implement Faber-English:** Essential for the control group.
3.  **Define the Prompting Strategy:** Test language structure, not doc memorization.

## Claude Opus 4.5: Notes

### 1. The IR Framing is Defensible

The shift from "Latin programming language" to "LLM-oriented IR" is the correct strategic move. "LLM drafts -> human audits -> compiler emits" is a concrete, measurable workflow claim that existing languages cannot easily counter.

### 2. Falsifiability is Built In

The ablation design (Faber-Latin vs Faber-English vs Faber-Symbols) is intellectually honest. You're explicitly setting up conditions where the Latin hypothesis could fail, and acknowledging the project survives either way. This is good research hygiene.

### 3. The Failure Taxonomy is Practical

The `tokenize_error` -> `parse_error` -> `semantic_error` -> ... -> `success` progression gives a clear improvement axis that directly feeds compiler prioritization. This is one of the most useful artifacts in the document.

### 4. Gaps to Address

- **Task suite definition is absent.** The document discusses metrics and harnesses but never specifies what tasks you'll actually test. FizzBuzz-level? Algorithm challenges? Real-world patterns like "implement a REST endpoint"? Task distribution will heavily influence results.

- **Which models?** "2-3 model families" is mentioned but not named. Claude, GPT-4, Gemini? Open-weight models like Llama? Tokenization and training data differ significantly.

- **Claim B may not be worth the effort.** Human reviewability is the hardest to measure. Unless you have a specific use case (compliance audits, security review), deprioritize this and focus on Claims A and C.

- **The Faber-English ablation is non-trivial.** It requires maintaining a parallel keyword set through the entire compiler. Is this already implemented? If not, the Phase 2 timeline (1-3 weeks) is optimistic.

- **No discussion of statistical power.** "10-30 tasks" in Phase 3 may be too small to detect meaningful differences across 3+ conditions. You might get noisy results that don't replicate at scale.

### 5. Key Sentence to Promote

Line 157: "Faber-English is strategically important because it removes Latin as a confound while keeping the grammar identical." This is the intellectual core of the entire ablation strategy. Consider promoting it to the executive summary.

### 6. Recommended Sequencing

Get Claim C (multi-target semantic preservation) working first. That's pure engineering with immediate credibility payoff-no LLM experiments needed, just deterministic compiler correctness. Use that stable foundation to run the LLM experiments for Claim A.

### Summary

The document is honest about what Faber Romanus is and isn't, and sets up a framework where you can win even if Latin turns out to be irrelevant. The main risk is overcommitting to the full research protocol before the tooling is ready. Stabilize the compiler, then run the experiments.

## GLM-4.7: Notes

### 1. Strategic Assessment: IR Framing is Correct

The IR framing is the right move. "TypeScript in a toga" was always a weak argument; "LLM-friendly IR" is a defensible market position. The workflow claim (LLM drafts → human audits → compiler emits) separates you from every mainstream language.

- **Verdict:** Keep this central. This is the strongest differentiator.

### 2. Critical Gaps

**Task suite definition is missing.** Claims A and B are impossible without specifying what you're measuring. Algorithm challenges? Real-world patterns? API integrations? The distribution heavily influences your results - a suite of simple algorithms will favor regular structure; real-world CRUD might favor library knowledge.

**No statistical power analysis.** 10-30 tasks in Phase 3 is noise. With 3+ conditions (Faber-Latin, Faber-English, Faber-Symbols, direct TS), you'll need more to detect meaningful differences. Power analysis should drive sample size, not convenience.

**Model drift is unaddressed.** Silent updates happen. "temperature=0" is not determinism. You need version locking or at least documented model snapshots for reproducibility.

### 3. Claim Prioritization

Deprioritize Claim B (reviewability). It's the hardest to measure, the noisiest (human skill varies wildly), and the least critical to the core value prop. Focus on A (LLM reliability) and C (multi-target semantics) first.

Claim C should be completed before any LLM experiments. It's pure engineering - no models, no cost, just compiler correctness. Build credibility on the thing you control.

### 4. Implementation Reality Check

Faber-English is non-trivial. The ablation seems simple ("same grammar, different keywords") but requires maintaining parallel keyword sets through tokenizer → parser → diagnostics → codegen. 1-3 weeks is optimistic unless you've already architected for keyword abstraction.

The harness is the real deliverable, but its scope is undefined. Does it handle:

- Prompt logging and versioning?
- Automatic recompilation when compiler changes?
- Regression tracking across runs?
- A/B testing of different prompt strategies?

**Recommendation:** Start with a minimal harness that just compiles/tests, measure baseline success rates, then iterate. Don't overbuild before you know what works.

### 5. The "Who Is the Audience?" Section is Insightful

The long-term question - whether LLMs would voluntarily choose a low-entropy IR - is the most interesting speculative claim. That's testable once you have the harness: give models a choice and measure preference. That paper would get attention.

### Summary

The framework is sound. The failure taxonomy is genuinely useful. The main risk is overcommitting to the full research protocol before the tooling is stable. Stabilize the compiler (Claim C), build a minimal harness, measure baseline, then iterate. Don't try to prove everything at once.

## GPT-5.2: Notes

### 1. The Document’s Center of Gravity is Right

Treating Faber as an **LLM-oriented intermediate representation** (not “a Latin language”) is the correct keystone. It sets a win condition you can actually deliver: deterministic compilation + measurable deltas in failure/repair rates.

- **Verdict:** Keep the IR framing as the first sentence of any public pitch.

### 2. Narrow to Two Primary Claims (A + C)

Claim B (human reviewability) is high-cost, high-noise, and easy to argue with. Unless you have a concrete driver (security audits, compliance review, regulated codebases), it should be optional.

- **Recommendation:** Make Claims A and C primary, and treat B as a “later study” or replace it with a smaller, mechanically-gradeable proxy (e.g., type inference questions, control-flow questions, seeded bug localization with fixed time budget).

### 3. Write Down the “Benchmark Contract” Explicitly

Right now the document implies a harness, but the key missing piece is the contract that prevents critics (and future-you) from accidentally moving the goalposts.

The contract should specify:

- **Task suite distribution:** what categories exist, what is excluded, and why.
- **Tool/feedback policy:** whether compiler errors are provided, whether tests are provided, and how many repair iterations are allowed.
- **Prompt protocol:** template, constraints, and any few-shot examples.
- **Stopping rules:** success/failure, max tokens/time, and retry budget.

If these aren’t frozen, “repair cost” becomes a measurement of prompt engineering and diagnostic verbosity instead of IR quality.

### 4. Split the Hypothesis into “Structure” vs “Vocabulary”

You already have the right ablation idea; make the causal story even more explicit:

- **H1 (structure/regularity):** same grammar/AST discipline reduces errors.
- **H2 (Latin priors):** Latin-ish vocabulary improves reliability beyond structure.

`Faber-English` is not just “nice to have”; it’s the only clean test of H2.

### 5. Make Reproducibility Concrete (What Gets Logged)

“Rerunnable” needs to mean: someone can run the same harness and get comparable aggregates, even if individual generations differ.

At minimum, log per-run:

- Compiler git SHA, harness git SHA
- Model identifier + timestamp (and region/provider if relevant)
- Prompt, raw model output, normalized source
- All tool outputs (compiler diagnostics, test output)
- Outcome bucket + iteration count

A simple `jsonl` log format becomes a serious artifact quickly.

### 6. Faber-English Implementation: Avoid a Parallel Compiler

The risk is accidentally forking the language into two first-class dialects.

- **Recommendation:** implement a single canonical token/AST pipeline, with a thin “dialect vocabulary table” at the tokenizer/parser boundary.
- **EDGE:** diagnostics should remain canonical (Latin) internally, but the harness can optionally post-process messages for the English condition so error-message familiarity doesn’t become an untracked confound.

### Summary

The plan is solid and defensible. The highest-value improvements are (1) freezing the benchmark contract, (2) sequencing Claim C before any model runs, and (3) treating `Faber-English` as a first-class experimental control without letting it become a second language.
