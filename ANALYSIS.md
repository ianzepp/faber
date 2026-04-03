# Faber Romanus — Fresh-Eyes Codebase Evaluation

*Generated 2026-02-28 by an independent Claude session with no prior context.*

---

## What Is This?

A **Latin-based intermediate representation language** designed for LLM-to-systems-language compilation. The idea: LLMs write Faber (regular, word-based syntax), humans skim it to verify intent, and a compiler emits Rust/TypeScript/Go/Python/Zig/C++. The pitch is that generating Faber then compiling is more reliable than generating Rust directly (no lifetime chaos, no borrow checker noise in the LLM's output).

---

## First Impressions

**This is a solo project with ~1,600 commits in ~4 months.** That's an extraordinary velocity. The repo is clearly AI-assisted (agent config files, multiple LLM tool configs). What's remarkable is that despite the speed, the codebase quality is *well above average*.

---

## The Good

**1. The core idea is genuinely interesting.** The README makes a credible case with actual trial data (13,000 trials, 17 models, 96-98% accuracy). The research methodology exists in a separate repo with falsifiable claims. This isn't vaporware — there's evidence behind the thesis.

**2. radix-rs is a serious compiler.** ~17.5k LOC of Rust, recursive descent, with a 7-pass semantic analysis pipeline (collect → resolve → lower → typecheck → borrow → exhaustive → lint). That's a real compiler architecture, not a toy transpiler. Bidirectional type inference, HIR lowering, pattern exhaustiveness checking — these are non-trivial features done correctly.

**3. Documentation is exceptional.** Every module has comprehensive headers explaining WHY, not just WHAT. The EBNF grammar is 628 lines of formal specification. DEVELOPER.md codifies clear standards. This is some of the best-documented compiler code I've seen in a personal project.

**4. Testing is thorough.** 174 Rust tests, 131 end-to-end `.fab` example programs, 71 YAML cross-target test specs. 42% of the codebase is test code. The E2E harness actually invokes `rustc` on generated output to verify validity.

**5. Minimal dependencies.** Only 5 production crates (ariadne, thiserror, rustc-hash, unicode-ident, unicode-normalization). Zero bloat. This is a compiler that could be audited in an afternoon.

**6. Error handling philosophy is correct.** The compiler never crashes on malformed input — it collects errors and continues. This is exactly right for an LLM-facing tool where you want all issues at once.

---

## The Concerns

**1. Single-developer risk.** 1,600 commits, one contributor. The bus factor is 1. The extensive documentation mitigates this somewhat, but there's no evidence of external contributors, code review, or community.

**2. The Latin vocabulary is a double-edged sword.** The README argues Latin helps (skimmable, unambiguous, morphologically rich). The counter-argument: it adds a learning barrier for *every* human who touches it. The project acknowledges this is an open question ("Do Latin keywords help, or is it just the regular structure?") but hasn't run the ablation yet. A foreign engineer would find `fixum`, `varia`, `discerne`, `cede` alienating before they found it useful.

**3. Multi-target ambition vs. reality.** The EBNF and test specs reference 6 targets (Rust, TypeScript, Python, Go, Zig, C++). But radix-rs currently only emits Rust and TypeScript. The bootstrap compilers (nanus-*) cover more targets but are minimal (~2-4k LOC each). There's a gap between what's *specified* and what's *implemented*.

**4. 33 open issues, no triage labels.** For an active project, this is manageable. But combined with solo development, it suggests feature velocity is outpacing stabilization. The README says "39 tests" but memory says "26 tests" — the actual count is 174. These inconsistencies hint at docs lagging behind code.

**5. The self-hosted compiler (rivus) is dead weight.** It's "deprioritized" but still present. The faber-ts reference compiler is "archived." There are multiple dead-end implementations cluttering the repo. A fresh contributor would waste time understanding what's active vs. abandoned.

**6. Practical adoption path is unclear.** Who uses this today? The trials prove LLMs *can* learn Faber, but there's no evidence of anyone using it in a real workflow. No VS Code extension, no language server, no package registry. The website (faberlang.dev) exists but the tooling ecosystem doesn't.

---

## Architecture Assessment

| Aspect | Grade | Notes |
|--------|-------|-------|
| Core compiler (radix-rs) | **A** | Genuinely well-engineered Rust compiler |
| Test coverage | **A-** | Comprehensive, multi-level strategy |
| Documentation | **A** | Best-in-class for a personal project |
| Dependency hygiene | **A+** | Minimal, auditable |
| Language design | **B+** | Interesting but unproven in practice |
| Multi-target readiness | **C+** | Specified for 6, implemented for 2 |
| Ecosystem/tooling | **D** | No LSP, no formatter, no editor support |
| Community/adoption | **D** | Solo project, no external users visible |

---

## Development Timeline

### Calendar Span

**Dec 20, 2025 → Feb 16, 2026** (58 calendar days)

### Active Days

**41 out of 58** (71% of days had commits)

### Activity by Period

| Period | Days | Commits | Avg/day |
|--------|------|---------|---------|
| Dec 20 – Jan 22 | 34 consecutive days | ~1,480 | ~44/day |
| Jan 25 – Jan 29 | 5 days (after 2-day gap) | ~30 | ~6/day |
| Feb 15 – Feb 16 | 2 days (after 16-day gap) | 89 | ~45/day |

### Working Pattern

- **~40 actual working days**, with essentially zero days off for the first 34 days straight (including Christmas and New Year's)
- **~39 commits/day average** across active days — roughly one commit every 20-30 minutes during working hours
- Peak hours are **7-10 AM** and **7 PM-midnight** — classic "before/after day job" pattern, or just long days
- Activity is fairly uniform across all days of the week (no real weekends)
- There's a **16-day gap** (Jan 30 – Feb 14) where development stopped, then a brief 2-day burst

### Commits by Month

| Month | Commits |
|-------|---------|
| December 2025 | 608 |
| January 2026 | 902 |
| February 2026 | 89 |

### Commits by Hour (EST)

Peak hours: 7-10 AM and 7 PM-midnight. Activity at nearly all hours suggests long working sessions.

### Commits by Day of Week

| Day | Commits |
|-----|---------|
| Tue | 298 |
| Sun | 255 |
| Mon | 253 |
| Wed | 239 |
| Thu | 230 |
| Sat | 164 |
| Fri | 160 |

---

## LOC Analysis

### Commit Size Profile

| Metric | Value |
|--------|-------|
| **Median commit** | **127 LOC** |
| **Avg commit (excl. top 5%)** | **247 LOC** |
| P10 | 8 LOC |
| P25 | 35 LOC |
| P75 | 376 LOC |
| P90 | 974 LOC |
| P95 | 1,916 LOC |
| P99 | 20,642 LOC |
| Max | 323,460 LOC |

### Commit Size Distribution

| LOC Range | Count | Pct |
|-----------|-------|-----|
| 1-10 | 180 | 11% |
| 11-50 | 306 | 19% |
| 51-100 | 225 | 14% |
| 101-250 | 321 | 20% |
| 251-500 | 242 | 15% |
| 501-1000 | 141 | 8% |
| 1000+ | 154 | 9% |

### The Churn Story

| Metric | Value |
|--------|-------|
| **Total insertions** | 1,242,873 |
| **Total deletions** | 1,065,625 |
| **Net LOC surviving** | ~177,000 |
| **Churn ratio** | **86% of all code written was later deleted** |

The churn ratio is the most revealing number. 2.3 million lines were touched, but only ~177k remain. The massive spike days (Dec 22: 440k, Dec 24: 341k, Dec 27: 332k, Jan 17: 303k) were mostly **large-scale reorganizations** — splitting monolithic codegen into per-target files, reorganizing example directories, migrating import syntax, etc.

### Daily LOC Breakdown

| Date | Commits | Insertions | Deletions | Total | Per-Commit |
|------|---------|------------|-----------|-------|------------|
| 2025-12-20 | 11 | 4,287 | 39 | 4,326 | 393 |
| 2025-12-21 | 42 | 54,004 | 10,351 | 64,355 | 1,532 |
| 2025-12-22 | 68 | 278,382 | 161,493 | 439,875 | 6,468 |
| 2025-12-23 | 50 | 13,315 | 2,320 | 15,635 | 312 |
| 2025-12-24 | 66 | 231,275 | 110,142 | 341,417 | 5,172 |
| 2025-12-25 | 55 | 9,750 | 2,953 | 12,703 | 230 |
| 2025-12-26 | 63 | 25,090 | 14,696 | 39,786 | 631 |
| 2025-12-27 | 35 | 160,651 | 171,781 | 332,432 | 9,498 |
| 2025-12-28 | 33 | 17,957 | 8,367 | 26,324 | 797 |
| 2025-12-29 | 48 | 13,215 | 2,623 | 15,838 | 329 |
| 2025-12-30 | 77 | 17,065 | 17,563 | 34,628 | 449 |
| 2025-12-31 | 58 | 31,170 | 23,019 | 54,189 | 934 |
| 2026-01-01 | 40 | 8,786 | 1,849 | 10,635 | 265 |
| 2026-01-02 | 36 | 3,887 | 42,861 | 46,748 | 1,298 |
| 2026-01-03 | 52 | 47,437 | 1,995 | 49,432 | 950 |
| 2026-01-04 | 29 | 8,619 | 3,305 | 11,924 | 411 |
| 2026-01-05 | 31 | 6,009 | 1,959 | 7,968 | 257 |
| 2026-01-06 | 61 | 52,348 | 11,301 | 63,649 | 1,043 |
| 2026-01-07 | 41 | 31,807 | 766 | 32,573 | 794 |
| 2026-01-08 | 45 | 5,984 | 4,819 | 10,803 | 240 |
| 2026-01-09 | 29 | 5,469 | 3,083 | 8,552 | 294 |
| 2026-01-10 | 17 | 2,691 | 1,171 | 3,862 | 227 |
| 2026-01-11 | 5 | 85 | 59,790 | 59,875 | 11,975 |
| 2026-01-12 | 27 | 7,794 | 3,160 | 10,954 | 405 |
| 2026-01-13 | 29 | 9,872 | 6,059 | 15,931 | 549 |
| 2026-01-14 | 35 | 3,699 | 1,441 | 5,140 | 146 |
| 2026-01-15 | 50 | 20,371 | 6,490 | 26,861 | 537 |
| 2026-01-16 | 26 | 3,947 | 1,259 | 5,206 | 200 |
| 2026-01-17 | 47 | 9,068 | 294,322 | 303,390 | 6,455 |
| 2026-01-18 | 49 | 11,418 | 2,536 | 13,954 | 284 |
| 2026-01-19 | 25 | 9,678 | 3,072 | 12,750 | 510 |
| 2026-01-20 | 23 | 8,895 | 1,371 | 10,266 | 446 |
| 2026-01-21 | 27 | 9,299 | 3,245 | 12,544 | 464 |
| 2026-01-22 | 26 | 15,926 | 15,369 | 31,295 | 1,203 |
| 2026-01-25 | 13 | 646 | 446 | 1,092 | 84 |
| 2026-01-26 | 43 | 13,532 | 7,485 | 21,017 | 488 |
| 2026-01-27 | 57 | 16,185 | 45,632 | 61,817 | 1,084 |
| 2026-01-28 | 10 | 7,213 | 887 | 8,100 | 810 |
| 2026-01-29 | 3 | 813 | 202 | 1,015 | 338 |
| 2026-02-15 | 81 | 64,059 | 13,762 | 77,821 | 960 |
| 2026-02-16 | 6 | 1,175 | 641 | 1,816 | 302 |
| **TOTAL** | **1,569** | **1,242,873** | **1,065,625** | **2,308,498** | **1,471** |
| **AVG/DAY** | **38** | **30,314** | **25,991** | **56,305** | — |

### What the LOC Data Means

The **median commit of 127 LOC** is actually quite reasonable — it suggests focused, incremental work. But the distribution has a very long tail: 9% of commits are 1000+ LOC, and those account for the vast majority of total churn. The pattern is consistent with AI-assisted development: **lots of small focused commits interspersed with occasional large automated refactors or file reorganizations**.

The net survival of ~177k LOC against a current codebase of ~30k LOC (radix-rs) + supporting files means there were significant dead-end implementations that were written and then removed (the retired TypeScript reference compiler, codegen refactors, etc.). That's not unusual for a project that's iterating on its approach, but it does confirm a "generate fast, discard freely" workflow.

---

## Bottom Line

If someone asked me "should I contribute to this?", I'd say: **the compiler engineering is legitimately good, and the thesis is intellectually honest.** The author clearly knows what they're doing with compiler design. But it's a research-stage project with a party-of-one community, an unconventional surface syntax that will polarize people, and a gap between the ambitious multi-target vision and the current two-target reality.

The most impressive thing is the *discipline* — consistent code style, exhaustive documentation, comprehensive tests, minimal dependencies — sustained across 1,600 commits. That's rare. The riskiest thing is that the value proposition depends on LLM code generation workflows becoming mainstream, and that the Latin syntax specifically (not just regular syntax in general) is the right bet.
