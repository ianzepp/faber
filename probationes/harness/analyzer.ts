// Post-trial analysis using AI

import { readFileSync } from 'fs'
import { join } from 'path'
import { callModel } from './api'
import type { RawResponse, GradedResult, ErrorType } from './types'

const ROOT = join(import.meta.dir, '..')

export async function analyzeResults(runId: string): Promise<string> {
  // Load results
  const rawFile = readFileSync(join(ROOT, 'results', runId, 'raw_responses.jsonl'), 'utf-8')
  const gradedFile = readFileSync(join(ROOT, 'results', runId, 'graded_results.jsonl'), 'utf-8')

  const rawLines = rawFile.trim().split('\n').filter(l => l).map(line => JSON.parse(line) as RawResponse)
  const gradedLines = gradedFile.trim().split('\n').filter(l => l).map(line => JSON.parse(line) as GradedResult)

  // Build analysis data
  const summary = buildSummary(gradedLines)
  const failures = extractFailures(rawLines, gradedLines)

  // Build prompt for analysis
  const prompt = buildAnalysisPrompt(summary, failures)

  // Call model for analysis
  const response = await callModel(
    'anthropic/claude-3-haiku', // Fast and cheap for analysis
    prompt,
    0.7,
    2000
  )

  return response.content
}

interface LevelStats {
  total: number
  level_a: number  // typecheck pass
  level_b: number  // runtime pass
  level_c: number  // output correct
  correct: number  // all three
  errors: Partial<Record<ErrorType, number>>
}

interface Summary {
  totalTrials: number
  byContext: Record<string, LevelStats & { accuracy: number }>
  byModel: Record<string, LevelStats & { accuracy: number }>
  byTask: Record<string, LevelStats & { accuracy: number }>
  overall: LevelStats & { accuracy: number }
}

function emptyStats(): LevelStats {
  return { total: 0, level_a: 0, level_b: 0, level_c: 0, correct: 0, errors: {} }
}

function addResult(stats: LevelStats, r: GradedResult): void {
  stats.total++
  if (r.level_a) stats.level_a++
  if (r.level_b) stats.level_b++
  if (r.level_c) stats.level_c++
  if (r.correct) stats.correct++
  if (r.error_type) {
    stats.errors[r.error_type] = (stats.errors[r.error_type] || 0) + 1
  }
}

function withAccuracy(stats: LevelStats): LevelStats & { accuracy: number } {
  return { ...stats, accuracy: stats.total > 0 ? stats.correct / stats.total : 0 }
}

function buildSummary(results: GradedResult[]): Summary {
  const byContext: Record<string, LevelStats> = {}
  const byModel: Record<string, LevelStats> = {}
  const byTask: Record<string, LevelStats> = {}
  const overall = emptyStats()

  for (const r of results) {
    // By context
    if (!byContext[r.context]) byContext[r.context] = emptyStats()
    addResult(byContext[r.context], r)

    // By model
    if (!byModel[r.model]) byModel[r.model] = emptyStats()
    addResult(byModel[r.model], r)

    // By task
    if (!byTask[r.task_id]) byTask[r.task_id] = emptyStats()
    addResult(byTask[r.task_id], r)

    // Overall
    addResult(overall, r)
  }

  // Add accuracies
  const contextSummary: Record<string, LevelStats & { accuracy: number }> = {}
  for (const [ctx, stats] of Object.entries(byContext)) {
    contextSummary[ctx] = withAccuracy(stats)
  }

  const modelSummary: Record<string, LevelStats & { accuracy: number }> = {}
  for (const [model, stats] of Object.entries(byModel)) {
    modelSummary[model] = withAccuracy(stats)
  }

  const taskSummary: Record<string, LevelStats & { accuracy: number }> = {}
  for (const [task, stats] of Object.entries(byTask)) {
    taskSummary[task] = withAccuracy(stats)
  }

  return {
    totalTrials: results.length,
    byContext: contextSummary,
    byModel: modelSummary,
    byTask: taskSummary,
    overall: withAccuracy(overall)
  }
}

interface Failure {
  task: string
  model: string
  context: string
  nShot: number
  input: string
  output: string
  level_a: boolean
  level_b: boolean
  level_c: boolean
  error_type?: ErrorType
  error_detail?: string
}

function extractFailures(raw: RawResponse[], graded: GradedResult[]): Failure[] {
  const failures: Failure[] = []

  // Create map for quick lookup
  const gradedMap = new Map<string, GradedResult>()
  for (const g of graded) {
    const key = `${g.task_id}-${g.model}-${g.context}-${g.n_shot}`
    gradedMap.set(key, g)
  }

  for (const r of raw) {
    const key = `${r.task_id}-${r.model}-${r.context}-${r.n_shot}`
    const g = gradedMap.get(key)

    if (g && !g.correct) {
      // Extract input from prompt (rough heuristic)
      const inputMatch = r.prompt.match(/```(?:typescript|faber)\n([\s\S]+?)\n```/)
      const input = inputMatch ? inputMatch[1].trim() : 'unknown'

      failures.push({
        task: r.task_id,
        model: r.model,
        context: r.context,
        nShot: r.n_shot,
        input,
        output: r.response.trim(),
        level_a: g.level_a,
        level_b: g.level_b,
        level_c: g.level_c,
        error_type: g.error_type,
        error_detail: g.error_detail
      })
    }
  }

  return failures.slice(0, 20) // Limit to first 20 failures for token efficiency
}

function formatLevelStats(stats: LevelStats & { accuracy: number }): string {
  const pct = (n: number) => stats.total > 0 ? ((n / stats.total) * 100).toFixed(1) : '0'
  return `${stats.correct}/${stats.total} correct (${pct(stats.correct)}%) | ` +
    `typecheck: ${pct(stats.level_a)}% | ` +
    `runs: ${pct(stats.level_b)}% | ` +
    `output: ${pct(stats.level_c)}%`
}

function buildAnalysisPrompt(summary: Summary, failures: Failure[]): string {
  const errorSummary = Object.entries(summary.overall.errors)
    .map(([type, count]) => `- ${type}: ${count}`)
    .join('\n')

  return `You are analyzing the results of an LLM evaluation testing whether models can learn Faber, a Latin-inspired programming language.

## Grading Levels

Each response is graded on three levels:
- **Level A (Typecheck)**: Does the code parse and typecheck?
- **Level B (Runs)**: Does it execute without runtime errors?
- **Level C (Output)**: Does it produce the correct output?

A response is "correct" only if all three levels pass.

## Summary Statistics

Total trials: ${summary.totalTrials}
Overall: ${formatLevelStats(summary.overall)}

### Error Types
${errorSummary || 'No errors'}

### By Context (documentation level)
${Object.entries(summary.byContext).map(([ctx, stats]) =>
  `- ${ctx}: ${formatLevelStats(stats)}`
).join('\n')}

### By Model
${Object.entries(summary.byModel).map(([model, stats]) =>
  `- ${model}: ${formatLevelStats(stats)}`
).join('\n')}

### By Task
${Object.entries(summary.byTask).map(([task, stats]) =>
  `- ${task}: ${formatLevelStats(stats)}`
).join('\n')}

## Sample Failures (first 10)

${failures.slice(0, 10).map((f, i) => `
### Failure ${i + 1}: ${f.task}
- Model: ${f.model}
- Context: ${f.context}
- N-shot: ${f.nShot}
- Levels: A=${f.level_a ? 'pass' : 'FAIL'}, B=${f.level_b ? 'pass' : 'FAIL'}, C=${f.level_c ? 'pass' : 'FAIL'}
- Error: ${f.error_type || 'none'}${f.error_detail ? ` - ${f.error_detail.slice(0, 150)}` : ''}
- Input: \`${f.input.slice(0, 80)}${f.input.length > 80 ? '...' : ''}\`
- Output: \`${f.output.slice(0, 80)}${f.output.length > 80 ? '...' : ''}\`
`).join('\n')}

## Analysis Task

Based on these results, provide:

1. **Key Findings** - What patterns emerge about learnability?
2. **Level Analysis** - Where do models fail most: syntax, runtime, or output?
3. **Context Impact** - How does documentation level affect performance?
4. **Common Errors** - What mistakes do models make most often?
5. **Model Differences** - How do different models compare?
6. **Recommendations** - What does this tell us about Faber's design?

Focus on actionable insights about language design and LLM learnability.`
}
