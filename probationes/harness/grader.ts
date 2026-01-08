// Grade task responses using compile-based validation
//
// Three-level grading:
//   (a) Typecheck - does the code parse and typecheck?
//   (b) Run - does it execute without runtime errors?
//   (c) Output - does it produce the correct output (if any expected)?

import { $ } from 'bun'
import { join } from 'path'
import { tmpdir } from 'os'
import type { Task, GradedResult, ErrorType } from './types'

// Path to faber binary - can be overridden via FABER_BIN env var
const FABER_BIN = process.env.FABER_BIN || join(import.meta.dir, '../../opus/bin/faber')

interface GradeInput {
  task: Task
  response: string
  runId: string
  model: string
  nShot: number
  dialect: string
  context: string
}

interface LevelResult {
  level_a: boolean
  level_b: boolean
  level_c: boolean
  error_type?: ErrorType
  error_detail?: string
}

export async function gradeTask(
  task: Task,
  response: string,
  runId: string,
  model: string,
  nShot: number,
  dialect: string,
  context: string
): Promise<Omit<GradedResult, 'timestamp' | 'framework_version' | 'git_sha'>> {
  const input: GradeInput = { task, response, runId, model, nShot, dialect, context }

  // For prediction tasks, use raw response (just trim whitespace)
  // For code tasks, clean to extract code from markdown
  const cleaned = task.type === 'predict_output'
    ? response.trim()
    : cleanResponse(response)

  if (!cleaned) {
    return makeResult(input, {
      level_a: false,
      level_b: false,
      level_c: false,
      error_type: 'no_response',
      error_detail: 'Empty response after cleaning',
    })
  }

  let result: LevelResult

  switch (task.type) {
    case 'translate_ts_to_faber':
      result = await gradeFaber(cleaned, task.expected_output)
      break

    case 'translate_faber_to_ts':
      result = await gradeTypeScript(cleaned, task.expected_output)
      break

    case 'predict_output':
      result = gradePrediction(cleaned, task.expected_output)
      break

    case 'complete_code':
      result = await gradeCompletion(task, cleaned)
      break

    case 'write_typescript':
      result = await gradeTypeScript(cleaned, task.expected_output)
      break

    case 'write_faber':
      result = await gradeFaber(cleaned, task.expected_output)
      break

    default:
      result = {
        level_a: false,
        level_b: false,
        level_c: false,
        error_type: 'api_error',
        error_detail: `Unknown task type: ${task.type}`,
      }
  }

  return makeResult(input, result)
}

function makeResult(input: GradeInput, result: LevelResult): Omit<GradedResult, 'timestamp' | 'framework_version' | 'git_sha'> {
  const correct = result.level_a && result.level_b && result.level_c

  return {
    run_id: input.runId,
    task_id: input.task.id,
    model: input.model,
    n_shot: input.nShot,
    dialect: input.dialect,
    context: input.context,
    level_a: result.level_a,
    level_b: result.level_b,
    level_c: result.level_c,
    correct,
    error_type: result.error_type,
    error_detail: result.error_detail,
  }
}

function cleanResponse(response: string): string {
  let cleaned = response

  // Remove markdown code fences
  cleaned = cleaned.replace(/```(?:faber|typescript|ts)?\n?/g, '')
  cleaned = cleaned.replace(/```\n?/g, '')

  // Remove leading/trailing explanation text (keep only code)
  // Look for code-like content
  const lines = cleaned.split('\n')
  const codeLines: string[] = []
  let inCode = false

  for (const line of lines) {
    // Heuristic: code lines typically start with keywords, whitespace+code, or are empty
    const isCodeLike = /^(\s*$|fixum|varia|functio|si|secus|dum|ex|redde|scribe|genus|const|let|var|function|if|else|for|while|return|class|import|export|console|async|await|try|catch|throw)/.test(line)
      || /^\s+\S/.test(line) // indented content
      || /^[{}()\[\];]/.test(line.trim()) // brackets/punctuation

    if (isCodeLike || inCode) {
      codeLines.push(line)
      inCode = true
    }
  }

  return codeLines.join('\n').trim()
}

async function gradeFaber(code: string, expectedOutput?: string): Promise<LevelResult> {
  const tmpFile = join(tmpdir(), `faber-trial-${Date.now()}.fab`)

  try {
    await Bun.write(tmpFile, code)

    // Level (a): Typecheck
    const check = await $`${FABER_BIN} check ${tmpFile}`.quiet().nothrow()

    if (check.exitCode !== 0) {
      return {
        level_a: false,
        level_b: false,
        level_c: false,
        error_type: check.stderr.toString().includes('parse') ? 'syntax_error' : 'type_error',
        error_detail: check.stderr.toString().trim(),
      }
    }

    // Level (b): Run
    const run = await $`${FABER_BIN} run ${tmpFile}`.quiet().nothrow()

    if (run.exitCode !== 0) {
      return {
        level_a: true,
        level_b: false,
        level_c: false,
        error_type: 'runtime_error',
        error_detail: run.stderr.toString().trim(),
      }
    }

    // Level (c): Output comparison (if expected)
    const actualOutput = run.stdout.toString().trim()

    if (expectedOutput !== undefined) {
      const expected = expectedOutput.trim()
      if (actualOutput === expected) {
        return { level_a: true, level_b: true, level_c: true }
      }
      else {
        return {
          level_a: true,
          level_b: true,
          level_c: false,
          error_type: 'wrong_output',
          error_detail: `Expected: ${expected}\nGot: ${actualOutput}`,
        }
      }
    }

    // No expected output - running without error is success
    return { level_a: true, level_b: true, level_c: true }
  }
  finally {
    // Cleanup temp file
    try {
      await Bun.write(tmpFile, '') // truncate
      const { unlink } = await import('fs/promises')
      await unlink(tmpFile)
    }
    catch {
      // Ignore cleanup errors
    }
  }
}

async function gradeTypeScript(code: string, expectedOutput?: string): Promise<LevelResult> {
  const tmpFile = join(tmpdir(), `ts-trial-${Date.now()}.ts`)

  try {
    await Bun.write(tmpFile, code)

    // Level (a) + (b): Run directly with bun (it handles TS natively)
    // Bun will fail on both type errors and runtime errors
    const run = await $`bun run ${tmpFile}`.quiet().nothrow()

    if (run.exitCode !== 0) {
      const stderr = run.stderr.toString()
      const stdout = run.stdout.toString()
      const output = stderr || stdout

      // Determine error type from output
      const isSyntaxError = output.includes('SyntaxError') ||
        output.includes('error: Expected') ||
        output.includes('error: Unexpected')
      const isTypeError = output.includes('TypeError') ||
        output.includes('is not defined') ||
        output.includes('error:') // Bun's TS errors

      return {
        level_a: !isSyntaxError && !isTypeError,
        level_b: false,
        level_c: false,
        error_type: isSyntaxError ? 'syntax_error' : isTypeError ? 'type_error' : 'runtime_error',
        error_detail: output.trim().slice(0, 500),
      }
    }

    // Level (c): Output comparison (if expected)
    const actualOutput = run.stdout.toString().trim()

    if (expectedOutput !== undefined) {
      const expected = expectedOutput.trim()
      if (actualOutput === expected) {
        return { level_a: true, level_b: true, level_c: true }
      }
      else {
        return {
          level_a: true,
          level_b: true,
          level_c: false,
          error_type: 'wrong_output',
          error_detail: `Expected: ${expected}\nGot: ${actualOutput}`,
        }
      }
    }

    // No expected output - running without error is success
    return { level_a: true, level_b: true, level_c: true }
  }
  finally {
    // Cleanup temp file
    try {
      const { unlink } = await import('fs/promises')
      await unlink(tmpFile)
    }
    catch {
      // Ignore cleanup errors
    }
  }
}

function gradePrediction(response: string, expectedOutput?: string): LevelResult {
  // For prediction tasks, we just compare the output directly
  // No compilation step - the model is predicting what code would output

  if (!expectedOutput) {
    return {
      level_a: true,
      level_b: true,
      level_c: false,
      error_type: 'api_error',
      error_detail: 'No expected output defined for prediction task',
    }
  }

  const actual = response.trim()
  const expected = expectedOutput.trim()

  if (actual === expected) {
    return { level_a: true, level_b: true, level_c: true }
  }

  return {
    level_a: true,
    level_b: true,
    level_c: false,
    error_type: 'wrong_output',
    error_detail: `Expected: ${expected}\nGot: ${actual}`,
  }
}

async function gradeCompletion(task: Task, completion: string): Promise<LevelResult> {
  // For completion tasks, we need to insert the completion into the template
  // and then grade the full code

  const template = task.input || task.input_faber || ''
  const placeholder = '___'

  if (!template.includes(placeholder)) {
    return {
      level_a: false,
      level_b: false,
      level_c: false,
      error_type: 'api_error',
      error_detail: 'Completion task missing placeholder in input',
    }
  }

  // Replace placeholder with completion (handle both ___ and ___())
  let fullCode = template
  if (template.includes('___()')) {
    fullCode = template.replace('___(', completion.trim() + '(')
  }
  else {
    fullCode = template.replace('___', completion.trim())
  }

  // Completion tasks are Faber code
  return await gradeFaber(fullCode, task.expected_output)
}
