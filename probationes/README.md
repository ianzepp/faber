# Probationes

LLM learnability research harness for the Faber programming language.

## Overview

Tests whether models can learn Faber syntax and semantics from limited context. Faber is a Latin-keyword programming language designed to be "LLM-comfortable" — using linguistic priors and regular structure to accelerate learning.

**Key question:** How much context does a model need to correctly read and write Faber code?

## Quick Start

```bash
# Prerequisites (from faber-romanus root)
export OPENROUTER_API_KEY="your-key"

# Run a single model test (1,900 trials: 5 contexts × 4 n-shots × 95 tasks)
bun run trial --model gpt-3.5-turbo

# Run with specific settings
bun run trial --model llama-3.1-8b --n-shot 3 --context minimal

# Verbose output
bun run trial --model gpt-3.5-turbo --verbose
```

## CLI Options

| Flag | Description | Example |
|------|-------------|---------|
| `--model` | Model ID or glob pattern | `gpt-3.5-turbo`, `llama-*` |
| `--task` | Task ID or glob pattern | `ts_to_faber_*` |
| `--category` | Task category filter | `declarations`, `functions` |
| `--n-shot` | Comma-separated n-shot values | `3` or `0,1,3,10` |
| `--context` | Context level | `examples-only`, `minimal`, `basic`, `complete` |
| `--dialect` | Language dialect | `latin` (default) |
| `--verbose`, `-v` | Show detailed progress | |

## Task Types

All tasks produce verifiable output for automated grading:

| Type | Description | Example |
|------|-------------|---------|
| `translate_ts_to_faber` | Convert TypeScript to Faber | `const x = 5` → `fixum x = 5` |
| `translate_faber_to_ts` | Convert Faber to TypeScript | `fixum x = 5` → `const x = 5` |
| `predict_output` | Predict what Faber code outputs | `scribe(42)` → `42` |
| `complete_code` | Fill in missing keyword | `___ x = 5` → `fixum` |

**95 tasks total** across categories: declarations, conditionals, functions, loops, arithmetic, boolean, error_handling, control_flow, data_structures, types, operators, strings, modules, entry_point.

## Three-Level Grading

Each response is graded on three levels:

| Level | Check | Faber | TypeScript |
|-------|-------|-------|------------|
| **A** | Typechecks | `faber check` passes | `bun run` starts |
| **B** | Runs | `faber run` exits 0 | `bun run` exits 0 |
| **C** | Correct output | stdout matches expected | stdout matches expected |

A task passes only if all three levels pass. Error types: `syntax_error`, `type_error`, `runtime_error`, `wrong_output`, `no_response`.

## Context Levels

| Context | Description |
|---------|-------------|
| `examples-only` | Just few-shot examples, no vocabulary reference |
| `minimal` | Key vocabulary (fixum=const, etc.) + syntax rules |
| `basic` | Quick reference with types, keywords, operators |
| `complete` | Full grammar reference with all patterns |

## Configuration

### Models (`config/models.yml`)

```yaml
framework_version: "1.1"  # Bump when tasks/grading/prompts change

models:
  - id: gpt-3.5-turbo
    provider: openrouter
    name: openai/gpt-3.5-turbo
    cost_per_1m_input: 0.50
    cost_per_1m_output: 1.50
```

### Trial Settings

```yaml
trials:
  n_shots: [0, 1, 3, 10]
  dialects: [latin]
  contexts: [examples-only, minimal, basic, complete]
  temperature: 0.0
  max_tokens: 1000
  seed: 42
```

## Results

Results are saved to `results/{run-id}/` and **committed to git as research artifacts**:

| File | Contents |
|------|----------|
| `raw_responses.jsonl` | Every prompt/response with tokens and latency |
| `graded_results.jsonl` | Three-level grading for each response |
| `analysis.md` | AI-generated analysis of patterns |

Each result includes `framework_version` and `git_sha` for reproducibility.

### Sample Output

```
.....................x...........x..x.....

39/42 passed (93%) in 19.6s — $0.01
Analysis: results/2026-01-05T20-27-27/analysis.md
```

Quiet by default: `.` = pass, `x` = fail, `E` = error. Use `--verbose` for details.

## Project Structure

```
faber-trials/
├── harness/
│   ├── runner.ts      # Main CLI entry point
│   ├── api.ts         # OpenRouter client with retry logic
│   ├── grader.ts      # Three-level compile-based grading
│   ├── prompt.ts      # Context and prompt construction
│   ├── loader.ts      # YAML task/config loading
│   ├── logger.ts      # JSONL result logging
│   ├── analyzer.ts    # AI-powered result analysis
│   └── types.ts       # TypeScript interfaces
├── tasks/
│   ├── translate.yml  # TS↔Faber translation tasks
│   ├── predict.yml    # Output prediction tasks
│   └── complete.yml   # Code completion tasks
├── examples/
│   └── base.yml       # Few-shot example pool
├── config/
│   └── models.yml     # Model definitions and trial config
└── results/           # Output directory (gitignored)
```

## Key Faber Syntax

Type-first parameter syntax (like C/Go, not TypeScript):

```faber
// Correct Faber syntax
functio double(numerus x) -> numerus {
  redde x * 2
}

// NOT TypeScript style
functio double(x: numerus) -> numerus { ... }  // WRONG
```

For-of loops: collection first, then binding:

```faber
ex items pro item {
  scribe item
}
```

## API Retry Logic

The runner automatically retries on:
- **429** (rate limit): exponential backoff 1s, 2s, 4s, 8s, 16s
- **5xx** (server errors): same backoff

Max 5 retries before failing the task.

## Interpreting Results

**By model size:**
- 1B models: ~20% (below capability threshold)
- 8B models: ~85% (competitive with larger models)
- GPT-3.5/Haiku: ~90-95%

**By context:**
- More context helps capable models (+10-15%)
- Tiny models can't leverage additional context

**By n-shot:**
- 0-shot: baseline (vocabulary only)
- 3-shot: good balance
- 10-shot: diminishing returns, may hurt tiny models

## Framework Versioning

Bump `framework_version` in `config/models.yml` when changing:
- Task definitions (new tasks, modified expected output)
- Grading logic (grader.ts)
- Prompt structure (prompt.ts)

This allows comparing results across framework versions.
