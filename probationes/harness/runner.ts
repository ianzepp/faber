#!/usr/bin/env bun

// Main trial runner

import { loadTasks, loadExamples, loadConfig } from './loader'
import { buildPrompt, buildVerificationPrompt } from './prompt'
import { callModel } from './api'
import { gradeTask } from './grader'
import { Logger } from './logger'
import { analyzeResults } from './analyzer'
import { writeFileSync } from 'fs'
import { join } from 'path'
import { $ } from 'bun'
import type { RawResponse, Summary, Task, Model, TransitionType, GradedResult } from './types'

const ROOT = join(import.meta.dir, '..')

async function getGitSha(): Promise<string | undefined> {
  try {
    const result = await $`git rev-parse --short HEAD`.quiet().nothrow()
    if (result.exitCode === 0) {
      return result.stdout.toString().trim()
    }
  }
  catch {
    // Not a git repo or git not available
  }
  return undefined
}

interface CLIArgs {
  model?: string
  verifier?: string
  task?: string
  category?: string
  nShot?: number[]
  dialect?: string
  context?: string
  verbose?: boolean
  passes?: number
}

function parseArgs(): CLIArgs {
  const args: CLIArgs = {}
  const argv = process.argv.slice(2)

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    const nextArg = argv[i + 1]

    if (arg === '--model' && nextArg) {
      args.model = nextArg
      i++
    } else if (arg === '--verifier' && nextArg) {
      args.verifier = nextArg
      i++
    } else if (arg === '--task' && nextArg) {
      args.task = nextArg
      i++
    } else if (arg === '--category' && nextArg) {
      args.category = nextArg
      i++
    } else if (arg === '--n-shot' && nextArg) {
      args.nShot = nextArg.split(',').map(n => parseInt(n.trim(), 10))
      i++
    } else if (arg === '--dialect' && nextArg) {
      args.dialect = nextArg
      i++
    } else if (arg === '--context' && nextArg) {
      args.context = nextArg
      i++
    } else if (arg === '--verbose' || arg === '-v') {
      args.verbose = true
    } else if (arg === '--passes' && nextArg) {
      args.passes = parseInt(nextArg, 10)
      i++
    }
  }

  return args
}

function matchesPattern(value: string, pattern: string): boolean {
  // Convert glob pattern to regex
  const regexPattern = pattern
    .replace(/\*/g, '.*')
    .replace(/\?/g, '.')
  return new RegExp(`^${regexPattern}$`).test(value)
}

function filterModels(models: Model[], pattern?: string): Model[] {
  if (!pattern) return models
  return models.filter(m => matchesPattern(m.id, pattern))
}

function filterTasks(tasks: Task[], taskPattern?: string, categoryPattern?: string): Task[] {
  let filtered = tasks

  if (taskPattern) {
    filtered = filtered.filter(t => matchesPattern(t.id, taskPattern))
  }

  if (categoryPattern) {
    filtered = filtered.filter(t => matchesPattern(t.category, categoryPattern))
  }

  return filtered
}

function filterNShots(nShots: number[], filter?: number[]): number[] {
  if (!filter) return nShots
  return nShots.filter(n => filter.includes(n))
}

function filterDialects(dialects: string[], filter?: string): string[] {
  if (!filter) return dialects
  return dialects.filter(d => d === filter)
}

function filterContexts(contexts: string[], filter?: string): string[] {
  if (!filter) return contexts
  return contexts.filter(c => c === filter)
}

function computeTransition(drafterCorrect: boolean, verifierCorrect: boolean): TransitionType {
  if (drafterCorrect && verifierCorrect) return 'preserved'
  if (drafterCorrect && !verifierCorrect) return 'damaged'
  if (!drafterCorrect && verifierCorrect) return 'recovered'
  return 'failed'
}

function computeCost(model: Model, tokensIn: number, tokensOut: number): number {
  return (tokensIn / 1_000_000) * model.cost_per_1m_input +
         (tokensOut / 1_000_000) * model.cost_per_1m_output
}

function buildRefinementPrompt(originalPrompt: string, previousResponse: string): string {
  return `${originalPrompt}

---

Your previous attempt was:

\`\`\`
${previousResponse}
\`\`\`

Please review and provide a corrected answer. Provide only the code or output, no explanation.`
}

async function main() {
  const cliArgs = parseArgs()
  const verbose = cliArgs.verbose ?? false
  const numPasses = cliArgs.passes ?? 1

  if (verbose) console.log('Loading tasks, examples, and config...')

  const allTasks = await loadTasks()
  const examples = await loadExamples()
  const { models: allModels, trials, framework_version } = await loadConfig()
  const git_sha = await getGitSha()

  if (verbose) console.log(`Framework version: ${framework_version}${git_sha ? ` (${git_sha})` : ''}`)

  // Apply filters
  const models = filterModels(allModels, cliArgs.model)
  const tasks = filterTasks(allTasks, cliArgs.task, cliArgs.category)
  const nShots = filterNShots(trials.n_shots, cliArgs.nShot)
  const dialects = filterDialects(trials.dialects, cliArgs.dialect)
  const contexts = filterContexts(trials.contexts, cliArgs.context)

  // Look up verifier model if specified
  const verifierModel = cliArgs.verifier
    ? allModels.find(m => m.id === cliArgs.verifier)
    : undefined

  if (cliArgs.verifier && !verifierModel) {
    console.error(`Unknown verifier model: ${cliArgs.verifier}`)
    process.exit(1)
  }

  if (models.length === 0) {
    console.error('No models match the filter')
    process.exit(1)
  }
  if (tasks.length === 0) {
    console.error('No tasks match the filter')
    process.exit(1)
  }
  if (nShots.length === 0) {
    console.error('No N-shot values match the filter')
    process.exit(1)
  }

  if (verbose) {
    console.log(`Loaded ${tasks.length} tasks, ${examples.length} examples`)
    console.log(`Testing ${models.length} models at ${nShots.join(', ')}-shot`)
    if (verifierModel) {
      console.log(`Pipeline mode: verifier = ${verifierModel.id}`)
    }
    if (Object.keys(cliArgs).length > 0) {
      console.log('Filters:', cliArgs)
    }
  }

  const runId = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
  const logger = new Logger(runId)
  await logger.init()

  if (verbose) {
    console.log(`Run ID: ${runId}`)
    console.log('Starting trials...\n')
  }

  let totalCost = 0
  let totalLatency = 0
  let completed = 0
  let passed = 0
  const total = models.length * nShots.length * dialects.length * contexts.length * tasks.length

  for (const model of models) {
    for (const nShot of nShots) {
      for (const dialect of dialects) {
        for (const context of contexts) {
          for (const task of tasks) {
            completed++

            try {
              // Build initial prompt
              const basePrompt = buildPrompt(task, examples, nShot, dialect, context)

              // Multi-pass loop for drafter
              let currentPrompt = basePrompt
              let drafterResponse = ''
              let drafterCost = 0
              let drafterLatency = 0
              let drafterTokensIn = 0
              let drafterTokensOut = 0

              for (let pass = 1; pass <= numPasses; pass++) {
                const apiResponse = await callModel(
                  model.name,
                  currentPrompt,
                  trials.temperature,
                  trials.max_tokens,
                  trials.seed
                )

                // Accumulate metrics
                drafterCost += computeCost(model, apiResponse.tokens_in, apiResponse.tokens_out)
                drafterLatency += apiResponse.latency_ms
                drafterTokensIn += apiResponse.tokens_in
                drafterTokensOut += apiResponse.tokens_out
                drafterResponse = apiResponse.content

                // Build refinement prompt for next pass
                if (pass < numPasses) {
                  currentPrompt = buildRefinementPrompt(basePrompt, apiResponse.content)
                }
              }

              // Grade drafter response
              const drafterGrade = await gradeTask(task, drafterResponse, runId, model.id, nShot, dialect, context)

              // Initialize final values from drafter
              let finalResponse = drafterResponse
              let finalCorrect = drafterGrade.correct
              let finalErrorType = drafterGrade.error_type
              let trialCost = drafterCost
              let trialLatency = drafterLatency

              // Pipeline fields (only set if verifier is used)
              let verifierCost: number | undefined
              let verifierLatency: number | undefined
              let verifierCorrect: boolean | undefined
              let verifierErrorType = drafterGrade.error_type
              let transition: TransitionType | undefined

              // Run verifier if specified
              if (verifierModel) {
                const verifyPrompt = buildVerificationPrompt(basePrompt, drafterResponse)
                const verifierApiResponse = await callModel(
                  verifierModel.name,
                  verifyPrompt,
                  trials.temperature,
                  trials.max_tokens,
                  trials.seed
                )

                verifierCost = computeCost(verifierModel, verifierApiResponse.tokens_in, verifierApiResponse.tokens_out)
                verifierLatency = verifierApiResponse.latency_ms
                finalResponse = verifierApiResponse.content

                // Grade verifier response
                const verifierGrade = await gradeTask(task, finalResponse, runId, verifierModel.id, nShot, dialect, context)
                verifierCorrect = verifierGrade.correct
                verifierErrorType = verifierGrade.error_type
                finalCorrect = verifierCorrect
                finalErrorType = verifierErrorType

                // Compute transition
                transition = computeTransition(drafterGrade.correct, verifierCorrect)

                // Update totals
                trialCost += verifierCost
                trialLatency += verifierLatency
              }

              totalCost += trialCost
              totalLatency += trialLatency

              // Log raw response
              const rawResponse: RawResponse = {
                timestamp: new Date().toISOString(),
                run_id: runId,
                framework_version,
                git_sha,
                task_id: task.id,
                model: model.id,
                n_shot: nShot,
                dialect,
                context,
                prompt: verifierModel ? buildVerificationPrompt(basePrompt, drafterResponse) : currentPrompt,
                response: finalResponse,
                tokens_in: drafterTokensIn + (verifierModel ? drafterTokensIn : 0),
                tokens_out: drafterTokensOut + (verifierModel ? drafterTokensOut : 0),
                latency_ms: trialLatency,
              }
              await logger.logRaw(rawResponse)

              // Build graded result with optional pipeline fields
              const gradedResult: GradedResult = {
                timestamp: new Date().toISOString(),
                run_id: runId,
                framework_version,
                git_sha,
                task_id: task.id,
                model: model.id,
                n_shot: nShot,
                dialect,
                context,
                level_a: drafterGrade.level_a,
                level_b: drafterGrade.level_b,
                level_c: drafterGrade.level_c,
                correct: finalCorrect,
                error_type: finalErrorType,
                error_detail: drafterGrade.error_detail,
                // Pipeline fields
                verifier_model: verifierModel?.id,
                drafter_correct: verifierModel ? drafterGrade.correct : undefined,
                drafter_error_type: verifierModel ? drafterGrade.error_type : undefined,
                verifier_correct: verifierCorrect,
                verifier_error_type: verifierModel ? verifierErrorType : undefined,
                transition,
                drafter_cost: verifierModel ? drafterCost : undefined,
                verifier_cost: verifierCost,
                total_cost: verifierModel ? trialCost : undefined,
                drafter_latency_ms: verifierModel ? drafterLatency : undefined,
                verifier_latency_ms: verifierLatency,
                total_latency_ms: verifierModel ? trialLatency : undefined,
              }
              await logger.logGraded(gradedResult)

              // Progress
              if (finalCorrect) passed++

              if (verbose) {
                const status = finalCorrect ? '✓' : '✗'
                const pipelineLabel = verifierModel ? ` → ${verifierModel.id}` : ''
                const passLabel = numPasses > 1 ? ` (${numPasses} passes)` : ''
                const transitionLabel = transition ? ` [${transition}]` : ''
                console.log(
                  `[${completed}/${total}] ${status} ${model.id}${pipelineLabel} ${nShot}-shot ${context} ${task.id}${passLabel}${transitionLabel} (${trialLatency}ms, $${trialCost.toFixed(4)})`
                )
              }
              else {
                if (verifierModel && transition) {
                  const symbol = transition === 'preserved' ? '.'
                    : transition === 'recovered' ? '+'
                    : transition === 'damaged' ? '-'
                    : 'x'
                  process.stdout.write(symbol)
                }
                else {
                  process.stdout.write(finalCorrect ? '.' : 'x')
                }
              }

            } catch (error) {
              if (verbose) {
                console.error(`Error on ${task.id}:`, error)
              } else {
                process.stdout.write('E')
              }
            }
        }
      }
    }
  }
  }

  // Newline after dots
  if (!verbose) console.log('')

  const pct = total > 0 ? Math.round((passed / total) * 100) : 0
  console.log(`\n${passed}/${total} passed (${pct}%) in ${(totalLatency / 1000).toFixed(1)}s — $${totalCost.toFixed(2)}`)

  // Generate AI analysis
  if (completed > 0) {
    try {
      if (verbose) console.log('Generating AI analysis...')
      const analysis = await analyzeResults(runId)
      writeFileSync(join(ROOT, 'results', runId, 'analysis.md'), analysis)
      console.log(`Analysis: probationes/results/${runId}/analysis.md`)
    } catch (error) {
      console.error('Failed to generate analysis:', error)
    }
  }
}

main().catch(console.error)
