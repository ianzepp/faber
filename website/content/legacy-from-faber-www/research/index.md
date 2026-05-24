---
title: Thesis and Trials
description: LLM learnability research for Faber
section: research
order: 0
---

# Thesis and Trials

Can LLMs learn Faber effectively? The [faber-romanus](https://github.com/ianzepp/faber-romanus) project includes an evaluation harness to test this systematically.

## Hypothesis

Faber's design choices - Latin vocabulary, regular morphology, consistent syntax - should make it easier for LLMs to learn from few examples compared to languages optimized for human ergonomics.

## Evaluation

The trials test:

- **Multiple models** - From GPT-3.5 to Llama 3.2 1B
- **N-shot learning** - 0, 1, 3, and 10 example configurations
- **Task types** - Translation, completion, prediction, explanation
- **Context levels** - From examples-only to complete documentation

## Trial Results

| Metric | Value |
|--------|-------|
| Framework version | 1.1 |
| Total evaluations | 13,270 |
| Models tested | 15 |
| Total cost | $12.04 |
| Total tokens | 9.5M in / 563K out |
| Total time | 18980.8s |

## Model Comparison: Cost vs Speed vs Accuracy

| Model | Accuracy | Avg Latency | Cost | Tokens |
|-------|----------|-------------|------|--------|
| gpt-4o | 89% | 829ms | $1.94 | 707K |
| qwen3-coder | 89% | 1.4s | $0.22 | 926K |
| gpt-3.5-turbo | 89% | 521ms | $0.40 | 762K |
| gpt-5 | 89% | 6.7s | $4.37 | 584K |
| gpt-4o-mini | 88% | 869ms | $0.10 | 630K |
| claude-3.5-sonnet | 88% | 1.8s | $2.21 | 667K |
| llama-3.1-70b | 86% | 1.1s | $0.21 | 609K |
| codestral | 86% | 541ms | $0.24 | 737K |
| deepseek-v3.1 | 85% | 2.0s | $0.10 | 617K |
| claude-4.5-sonnet | 77% | 1.5s | $1.74 | 518K |
| mercury-coder | 73% | 589ms | $0.22 | 834K |
| llama-3.1-8b | 73% | 915ms | $0.04 | 717K |
| claude-3-haiku | 70% | 970ms | $0.22 | 769K |
| llama-3.2-1b | 15% | 486ms | $0.03 | 778K |
| qwen2.5-coder-32b | 0% | 7.2s | $0.02 | 253K |

## Three-Level Grading Breakdown

**A** = typechecks, **B** = runs without error, **C** = correct output.

| Model | Tests | A (Typechecks) | B (Runs) | C (Correct) |
|-------|-------|----------------|----------|-------------|
| gpt-4o | 952 | 93% | 93% | 88% |
| qwen3-coder | 1068 | 94% | 94% | 89% |
| gpt-3.5-turbo | 1166 | 91% | 91% | 89% |
| gpt-5 | 672 | 93% | 93% | 89% |
| gpt-4o-mini | 895 | 93% | 93% | 88% |
| claude-3.5-sonnet | 840 | 95% | 95% | 88% |
| llama-3.1-70b | 870 | 91% | 91% | 86% |
| codestral | 964 | 93% | 92% | 86% |
| deepseek-v3.1 | 862 | 95% | 94% | 85% |
| claude-4.5-sonnet | 672 | 93% | 93% | 77% |
| mercury-coder | 840 | 76% | 76% | 73% |
| llama-3.1-8b | 1063 | 90% | 90% | 73% |
| claude-3-haiku | 946 | 92% | 92% | 70% |
| llama-3.2-1b | 1138 | 43% | 43% | 15% |
| qwen2.5-coder-32b | 282 | 29% | 29% | 0% |

## By Context Level

How much documentation context helps models learn Faber.

| Context | Tests | Accuracy |
|---------|-------|----------|
| examples-only | 2681 | 61% |
| grammar-only | 2662 | 82% |
| minimal | 2985 | 77% |
| basic | 2466 | 79% |
| complete | 2476 | 79% |

## By N-shot (Learning Curve)

Effect of few-shot examples on accuracy.

| Examples | Tests | Accuracy |
|----------|-------|----------|
| 0-shot | 3343 | 70% |
| 1-shot | 3073 | 71% |
| 3-shot | 3914 | 80% |
| 10-shot | 2940 | 81% |

## Error Distribution

Where failures occur (among failed trials only).

| Error Type | Count | % of Failures |
|------------|-------|---------------|
| type_error | 1360 | 42% |
| wrong_output | 1345 | 42% |
| no_response | 312 | 10% |
| syntax_error | 201 | 6% |
| runtime_error | 14 | 0% |

## By Task

| Task | Tests | Accuracy |
|------|-------|----------|
| faber_to_ts_functio_string | 305 | 95% |
| faber_to_ts_arithmetic | 303 | 94% |
| faber_to_ts_ex_pro | 304 | 93% |
| faber_to_ts_si_true | 305 | 92% |
| faber_to_ts_functio | 305 | 92% |
| complex_ts_to_faber_factorial | 12 | 92% |
| complex_ts_to_faber_fibonacci | 12 | 92% |
| complex_ts_to_faber_multi_function | 12 | 92% |
| complex_ts_to_faber_ternary_chain | 12 | 92% |
| complex_ts_to_faber_string_ops | 12 | 92% |
| complex_ts_to_faber_early_return | 12 | 92% |
| complex_ts_to_faber_accumulator | 12 | 92% |
| complex_ts_to_faber_prime_check | 12 | 92% |
| faber_to_ts_fixum | 304 | 91% |
| faber_to_ts_string | 305 | 91% |
| faber_to_ts_si_false | 305 | 91% |
| faber_to_ts_varia | 305 | 90% |
| faber_to_ts_dum | 303 | 89% |
| predict_const_value | 303 | 87% |
| faber_to_ts_boolean | 303 | 85% |
| ts_to_faber_const | 333 | 84% |
| complex_ts_to_faber_if_in_loop | 12 | 83% |
| complex_ts_to_faber_typed_params | 12 | 83% |
| complex_ts_to_faber_find_max | 12 | 83% |
| ts_to_faber_string | 332 | 82% |
| ts_to_faber_arithmetic | 331 | 82% |
| complete_const_keyword | 302 | 81% |
| ts_to_faber_let | 331 | 80% |
| complete_return_keyword | 302 | 79% |
| complete_let_keyword | 302 | 79% |
| ts_to_faber_if_false | 332 | 78% |
| complete_function_keyword | 301 | 78% |
| complete_while_keyword | 301 | 78% |
| ts_to_faber_if_true | 332 | 77% |
| predict_simple_output | 303 | 77% |
| complete_print_keyword | 300 | 77% |
| ts_to_faber_while | 331 | 76% |
| predict_function_math | 302 | 76% |
| predict_arithmetic_parens | 302 | 75% |
| predict_loop_sum | 302 | 75% |
| complex_ts_to_faber_fizzbuzz | 12 | 75% |
| complete_else_keyword | 301 | 74% |
| predict_conditional_true | 303 | 73% |
| complete_loop_keyword | 302 | 72% |
| predict_conditional_false | 304 | 71% |
| ts_to_faber_for_of | 332 | 67% |
| predict_arithmetic_precedence | 303 | 67% |
| complex_ts_to_faber_array_type | 12 | 67% |
| ts_to_faber_function | 332 | 65% |
| predict_loop_output | 302 | 65% |
| predict_function_call | 302 | 65% |
| ts_to_faber_boolean | 331 | 59% |
| complex_ts_to_faber_guard_clause | 12 | 58% |
| ts_to_faber_function_string | 332 | 57% |
| complex_ts_to_faber_loop_in_loop | 14 | 43% |
| complex_ts_to_faber_nested_if | 14 | 29% |
| predict_boolean_and | 302 | 16% |
| predict_boolean_or | 301 | 13% |
| complex_ts_to_faber_higher_order | 12 | 0% |
| complex_ts_to_faber_gcd | 12 | 0% |
| complex_ts_to_faber_binary_search | 14 | 0% |

## Methodology

- **Temperature:** 0.0 (deterministic)
- **Seed:** 42 (reproducible)
- **Dialects:** Latin keywords
- **Context levels:** examples-only, minimal, basic, complete

See [faber-romanus](https://github.com/ianzepp/faber-romanus) for raw data and methodology details.
