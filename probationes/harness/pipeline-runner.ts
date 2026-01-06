#!/usr/bin/env bun

// Pipeline trial runner: drafter → verifier
// Tests whether small+large model pipelines outperform single large model

import { loadTasks, loadExamples, loadConfig } from './loader'
import { buildPrompt } from './prompt'
import { callModel } from './api'
import { gradeTask } from './grader'
import { $ } from 'bun'
import { appendFileSync, mkdirSync, existsSync } from 'fs'
import { join } from 'path'
import type { Task, Model, PipelineResult, TransitionType } from './types'

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
  drafter: string
  verifier: string
  task?: string
  nShot?: number
  verbose?: boolean
}

function parseArgs(): CLIArgs {
  const args: CLIArgs = { drafter: '', verifier: '' }
  const argv = process.argv.slice(2)

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    const nextArg = argv[i + 1]

    if (arg === '--drafter' && nextArg) {
      args.drafter = nextArg
      i++
    }
    else if (arg === '--verifier' && nextArg) {
      args.verifier = nextArg
      i++
    }
    else if (arg === '--task' && nextArg) {
      args.task = nextArg
      i++
    }
    else if (arg === '--n-shot' && nextArg) {
      args.nShot = parseInt(nextArg, 10)
      i++
    }
    else if (arg === '--verbose' || arg === '-v') {
      args.verbose = true
    }
  }

  return args
}

function findModel(models: Model[], id: string): Model | undefined {
  return models.find(m => m.id === id)
}

function buildVerificationPrompt(originalPrompt: string, drafterResponse: string): string {
  return `${originalPrompt}

---

A previous model produced this response:

\`\`\`
${drafterResponse}
\`\`\`

Review this code. If it is correct, output it unchanged. If it contains errors, provide the corrected version.

Output only the code, no explanation.`
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

async function main() {
  const cliArgs = parseArgs()
  const verbose = cliArgs.verbose ?? false

  if (!cliArgs.drafter || !cliArgs.verifier) {
    console.error('Usage: bun run harness/pipeline-runner.ts --drafter <model> --verifier <model>')
    console.error('Options:')
    console.error('  --drafter <model>   Drafter model ID (small/fast)')
    console.error('  --verifier <model>  Verifier model ID (large/accurate)')
    console.error('  --task <pattern>    Task ID filter (default: ts_to_faber tasks)')
    console.error('  --n-shot <n>        Number of examples (default: 3)')
    console.error('  --verbose, -v       Show detailed progress')
    process.exit(1)
  }

  if (verbose) console.log('Loading tasks, examples, and config...')

  const allTasks = await loadTasks()
  const examples = await loadExamples()
  const { models, trials, framework_version } = await loadConfig()
  const git_sha = await getGitSha()

  // Find models
  const drafterModel = findModel(models, cliArgs.drafter)
  const verifierModel = findModel(models, cliArgs.verifier)

  if (!drafterModel) {
    console.error(`Unknown drafter model: ${cliArgs.drafter}`)
    process.exit(1)
  }
  if (!verifierModel) {
    console.error(`Unknown verifier model: ${cliArgs.verifier}`)
    process.exit(1)
  }

  // Filter to ts_to_faber tasks by default (hardest direction)
  const taskPattern = cliArgs.task || 'ts_to_faber*'
  const tasks = allTasks.filter(t => {
    if (taskPattern.includes('*')) {
      const regex = new RegExp('^' + taskPattern.replace(/\*/g, '.*') + '$')
      return regex.test(t.id)
    }
    return t.id === taskPattern
  })

  // Exclude predict_output tasks
  const filteredTasks = tasks.filter(t => t.type !== 'predict_output')

  if (filteredTasks.length === 0) {
    console.error('No tasks match the filter')
    process.exit(1)
  }

  const nShot = cliArgs.nShot ?? 3
  const dialect = 'latin'
  const context = 'grammar-only'

  if (verbose) {
    console.log(`Framework version: ${framework_version}${git_sha ? ` (${git_sha})` : ''}`)
    console.log(`Drafter: ${drafterModel.id}`)
    console.log(`Verifier: ${verifierModel.id}`)
    console.log(`Tasks: ${filteredTasks.length} (${taskPattern})`)
    console.log(`N-shot: ${nShot}, Context: ${context}`)
    console.log('')
  }

  // Setup output
  const runId = `pipeline-${new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)}`
  const resultsDir = join(ROOT, 'results', runId)

  if (!existsSync(resultsDir)) {
    mkdirSync(resultsDir, { recursive: true })
  }

  const resultsFile = `${resultsDir}/pipeline_results.jsonl`

  // Counters
  let completed = 0
  const total = filteredTasks.length
  const transitions: Record<TransitionType, number> = {
    preserved: 0,
    damaged: 0,
    recovered: 0,
    failed: 0,
  }
  let totalCost = 0
  let totalLatency = 0
  let drafterOnlyCorrect = 0
  let verifierFinalCorrect = 0

  console.log(`Running ${total} pipeline trials: ${drafterModel.id} → ${verifierModel.id}`)
  console.log('')

  for (const task of filteredTasks) {
    completed++

    try {
      // Build base prompt
      const basePrompt = buildPrompt(task, examples, nShot, dialect, context)

      // Call drafter
      const drafterStart = Date.now()
      const drafterResponse = await callModel(
        drafterModel.name,
        basePrompt,
        trials.temperature,
        trials.max_tokens,
        trials.seed
      )
      const drafterLatency = Date.now() - drafterStart

      // Grade drafter
      const drafterGrade = await gradeTask(
        task,
        drafterResponse.content,
        runId,
        drafterModel.id,
        nShot,
        dialect,
        context
      )
      const drafterCorrect = drafterGrade.correct
      const drafterCost = computeCost(drafterModel, drafterResponse.tokens_in, drafterResponse.tokens_out)

      if (drafterCorrect) drafterOnlyCorrect++

      // Build verification prompt and call verifier
      const verifyPrompt = buildVerificationPrompt(basePrompt, drafterResponse.content)
      const verifierStart = Date.now()
      const verifierResponse = await callModel(
        verifierModel.name,
        verifyPrompt,
        trials.temperature,
        trials.max_tokens,
        trials.seed
      )
      const verifierLatency = Date.now() - verifierStart

      // Grade verifier
      const verifierGrade = await gradeTask(
        task,
        verifierResponse.content,
        runId,
        verifierModel.id,
        nShot,
        dialect,
        context
      )
      const verifierCorrect = verifierGrade.correct
      const verifierCost = computeCost(verifierModel, verifierResponse.tokens_in, verifierResponse.tokens_out)

      if (verifierCorrect) verifierFinalCorrect++

      // Compute transition
      const transition = computeTransition(drafterCorrect, verifierCorrect)
      transitions[transition]++

      // Totals
      const trialCost = drafterCost + verifierCost
      const trialLatency = drafterLatency + verifierLatency
      totalCost += trialCost
      totalLatency += trialLatency

      // Build result
      const result: PipelineResult = {
        timestamp: new Date().toISOString(),
        run_id: runId,
        framework_version,
        git_sha,
        task_id: task.id,
        drafter_model: drafterModel.id,
        verifier_model: verifierModel.id,
        n_shot: nShot,
        dialect,
        context,
        drafter_response: drafterResponse.content,
        drafter_correct: drafterCorrect,
        drafter_error_type: drafterGrade.error_type,
        drafter_tokens_in: drafterResponse.tokens_in,
        drafter_tokens_out: drafterResponse.tokens_out,
        drafter_latency_ms: drafterLatency,
        drafter_cost: drafterCost,
        verifier_response: verifierResponse.content,
        verifier_correct: verifierCorrect,
        verifier_error_type: verifierGrade.error_type,
        verifier_tokens_in: verifierResponse.tokens_in,
        verifier_tokens_out: verifierResponse.tokens_out,
        verifier_latency_ms: verifierLatency,
        verifier_cost: verifierCost,
        transition,
        total_tokens_in: drafterResponse.tokens_in + verifierResponse.tokens_in,
        total_tokens_out: drafterResponse.tokens_out + verifierResponse.tokens_out,
        total_latency_ms: trialLatency,
        total_cost: trialCost,
      }

      // Log result
      appendFileSync(resultsFile, JSON.stringify(result) + '\n')

      // Progress
      if (verbose) {
        const dStatus = drafterCorrect ? '✓' : '✗'
        const vStatus = verifierCorrect ? '✓' : '✗'
        console.log(
          `[${completed}/${total}] ${task.id}: D=${dStatus} V=${vStatus} (${transition}) ${trialLatency}ms $${trialCost.toFixed(4)}`
        )
      }
      else {
        const symbol = transition === 'preserved' ? '.'
          : transition === 'recovered' ? '+'
          : transition === 'damaged' ? '-'
          : 'x'
        process.stdout.write(symbol)
      }
    }
    catch (error) {
      if (verbose) {
        console.error(`Error on ${task.id}:`, error)
      }
      else {
        process.stdout.write('E')
      }
    }
  }

  // Newline after progress
  if (!verbose) console.log('')

  // Summary
  console.log('')
  console.log('=== Pipeline Results ===')
  console.log(`Drafter: ${drafterModel.id}`)
  console.log(`Verifier: ${verifierModel.id}`)
  console.log('')
  console.log('Transitions:')
  console.log(`  preserved (D✓→V✓): ${transitions.preserved}`)
  console.log(`  damaged   (D✓→V✗): ${transitions.damaged}`)
  console.log(`  recovered (D✗→V✓): ${transitions.recovered}`)
  console.log(`  failed    (D✗→V✗): ${transitions.failed}`)
  console.log('')

  const recoveryRate = transitions.recovered + transitions.failed > 0
    ? (transitions.recovered / (transitions.recovered + transitions.failed) * 100).toFixed(1)
    : 'N/A'
  const damageRate = transitions.preserved + transitions.damaged > 0
    ? (transitions.damaged / (transitions.preserved + transitions.damaged) * 100).toFixed(1)
    : 'N/A'

  console.log(`Recovery rate (when D wrong): ${recoveryRate}%`)
  console.log(`Damage rate (when D correct): ${damageRate}%`)
  console.log('')
  console.log(`Drafter alone: ${drafterOnlyCorrect}/${total} (${(drafterOnlyCorrect / total * 100).toFixed(1)}%)`)
  console.log(`Pipeline final: ${verifierFinalCorrect}/${total} (${(verifierFinalCorrect / total * 100).toFixed(1)}%)`)
  console.log('')
  console.log(`Total time: ${(totalLatency / 1000).toFixed(1)}s`)
  console.log(`Total cost: $${totalCost.toFixed(4)}`)
  console.log('')
  console.log(`Results: ${resultsFile}`)
}

main().catch(console.error)
