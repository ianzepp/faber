#!/usr/bin/env bun

// SQLite database for trial results analysis
// Usage:
//   bun run probationes/harness/db.ts import [run-id]   # Import specific run or all
//   bun run probationes/harness/db.ts query "SQL"       # Run ad-hoc query
//   bun run probationes/harness/db.ts summary           # Show summary stats

import { Database } from 'bun:sqlite'
import { readdirSync, readFileSync, existsSync } from 'fs'
import { join } from 'path'
import type { GradedResult, RawResponse } from './types'

const ROOT = join(import.meta.dir, '..')
const DB_PATH = join(ROOT, 'results', 'trials.db')

function getDb(): Database {
  const db = new Database(DB_PATH)
  db.exec('PRAGMA journal_mode = WAL')
  return db
}

function initSchema(db: Database): void {
  db.exec(`
    CREATE TABLE IF NOT EXISTS trials (
      -- Composite primary key for idempotency
      run_id TEXT NOT NULL,
      task_id TEXT NOT NULL,
      model TEXT NOT NULL,
      n_shot INTEGER NOT NULL DEFAULT 0,
      dialect TEXT NOT NULL DEFAULT 'latin',
      context TEXT NOT NULL DEFAULT 'unknown',

      -- Metadata
      timestamp TEXT NOT NULL,
      framework_version TEXT,
      git_sha TEXT,

      -- Three-level grading
      level_a INTEGER NOT NULL,  -- 0/1 boolean
      level_b INTEGER NOT NULL,
      level_c INTEGER NOT NULL,
      correct INTEGER NOT NULL,
      error_type TEXT,
      error_detail TEXT,

      -- Pipeline fields (nullable)
      verifier_model TEXT,
      drafter_correct INTEGER,
      drafter_error_type TEXT,
      verifier_correct INTEGER,
      verifier_error_type TEXT,
      transition TEXT,

      -- Cost tracking
      drafter_cost REAL,
      verifier_cost REAL,
      total_cost REAL,
      drafter_latency_ms INTEGER,
      verifier_latency_ms INTEGER,
      total_latency_ms INTEGER,

      PRIMARY KEY (run_id, task_id, model, n_shot, dialect, context)
    );

    CREATE TABLE IF NOT EXISTS raw_responses (
      run_id TEXT NOT NULL,
      task_id TEXT NOT NULL,
      model TEXT NOT NULL,
      n_shot INTEGER NOT NULL DEFAULT 0,
      dialect TEXT NOT NULL DEFAULT 'latin',
      context TEXT NOT NULL DEFAULT 'unknown',

      timestamp TEXT NOT NULL,
      framework_version TEXT,
      git_sha TEXT,
      prompt TEXT NOT NULL,
      response TEXT NOT NULL,
      tokens_in INTEGER NOT NULL,
      tokens_out INTEGER NOT NULL,
      latency_ms INTEGER NOT NULL,

      PRIMARY KEY (run_id, task_id, model, n_shot, dialect, context)
    );

    -- Useful indexes
    CREATE INDEX IF NOT EXISTS idx_trials_model ON trials(model);
    CREATE INDEX IF NOT EXISTS idx_trials_task ON trials(task_id);
    CREATE INDEX IF NOT EXISTS idx_trials_context ON trials(context);
    CREATE INDEX IF NOT EXISTS idx_trials_correct ON trials(correct);
    CREATE INDEX IF NOT EXISTS idx_trials_framework ON trials(framework_version);
  `)
}

function importRun(db: Database, runId: string): { trials: number; raw: number } {
  const runDir = join(ROOT, 'results', runId)

  if (!existsSync(runDir)) {
    console.error(`Run directory not found: ${runDir}`)
    return { trials: 0, raw: 0 }
  }

  let trialsImported = 0
  let rawImported = 0

  // Import graded results
  const gradedPath = join(runDir, 'graded_results.jsonl')
  if (existsSync(gradedPath)) {
    const insertTrial = db.prepare(`
      INSERT OR REPLACE INTO trials (
        run_id, task_id, model, n_shot, dialect, context,
        timestamp, framework_version, git_sha,
        level_a, level_b, level_c, correct, error_type, error_detail,
        verifier_model, drafter_correct, drafter_error_type,
        verifier_correct, verifier_error_type, transition,
        drafter_cost, verifier_cost, total_cost,
        drafter_latency_ms, verifier_latency_ms, total_latency_ms
      ) VALUES (
        $run_id, $task_id, $model, $n_shot, $dialect, $context,
        $timestamp, $framework_version, $git_sha,
        $level_a, $level_b, $level_c, $correct, $error_type, $error_detail,
        $verifier_model, $drafter_correct, $drafter_error_type,
        $verifier_correct, $verifier_error_type, $transition,
        $drafter_cost, $verifier_cost, $total_cost,
        $drafter_latency_ms, $verifier_latency_ms, $total_latency_ms
      )
    `)

    const lines = readFileSync(gradedPath, 'utf-8').trim().split('\n').filter(l => l)
    for (const line of lines) {
      const r: GradedResult = JSON.parse(line)
      insertTrial.run({
        $run_id: r.run_id,
        $task_id: r.task_id,
        $model: r.model,
        $n_shot: r.n_shot ?? 0,
        $dialect: r.dialect ?? 'latin',
        $context: r.context ?? 'unknown',
        $timestamp: r.timestamp,
        $framework_version: r.framework_version,
        $git_sha: r.git_sha ?? null,
        $level_a: r.level_a ? 1 : 0,
        $level_b: r.level_b ? 1 : 0,
        $level_c: r.level_c ? 1 : 0,
        $correct: r.correct ? 1 : 0,
        $error_type: r.error_type ?? null,
        $error_detail: r.error_detail ?? null,
        $verifier_model: r.verifier_model ?? null,
        $drafter_correct: r.drafter_correct != null ? (r.drafter_correct ? 1 : 0) : null,
        $drafter_error_type: r.drafter_error_type ?? null,
        $verifier_correct: r.verifier_correct != null ? (r.verifier_correct ? 1 : 0) : null,
        $verifier_error_type: r.verifier_error_type ?? null,
        $transition: r.transition ?? null,
        $drafter_cost: r.drafter_cost ?? null,
        $verifier_cost: r.verifier_cost ?? null,
        $total_cost: r.total_cost ?? null,
        $drafter_latency_ms: r.drafter_latency_ms ?? null,
        $verifier_latency_ms: r.verifier_latency_ms ?? null,
        $total_latency_ms: r.total_latency_ms ?? null,
      })
      trialsImported++
    }
  }

  // Import raw responses
  const rawPath = join(runDir, 'raw_responses.jsonl')
  if (existsSync(rawPath)) {
    const insertRaw = db.prepare(`
      INSERT OR REPLACE INTO raw_responses (
        run_id, task_id, model, n_shot, dialect, context,
        timestamp, framework_version, git_sha,
        prompt, response, tokens_in, tokens_out, latency_ms
      ) VALUES (
        $run_id, $task_id, $model, $n_shot, $dialect, $context,
        $timestamp, $framework_version, $git_sha,
        $prompt, $response, $tokens_in, $tokens_out, $latency_ms
      )
    `)

    const lines = readFileSync(rawPath, 'utf-8').trim().split('\n').filter(l => l)
    for (const line of lines) {
      const r: RawResponse = JSON.parse(line)
      insertRaw.run({
        $run_id: r.run_id,
        $task_id: r.task_id,
        $model: r.model,
        $n_shot: r.n_shot ?? 0,
        $dialect: r.dialect ?? 'latin',
        $context: r.context ?? 'unknown',
        $timestamp: r.timestamp,
        $framework_version: r.framework_version,
        $git_sha: r.git_sha ?? null,
        $prompt: r.prompt,
        $response: r.response,
        $tokens_in: r.tokens_in,
        $tokens_out: r.tokens_out,
        $latency_ms: r.latency_ms,
      })
      rawImported++
    }
  }

  return { trials: trialsImported, raw: rawImported }
}

function importAll(db: Database): void {
  const resultsDir = join(ROOT, 'results')
  if (!existsSync(resultsDir)) {
    console.error('No results directory found')
    return
  }

  const runs = readdirSync(resultsDir).filter(f =>
    f.match(/^\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}$/)
  )

  console.log(`Found ${runs.length} runs to import`)

  let totalTrials = 0
  let totalRaw = 0

  for (const runId of runs) {
    const { trials, raw } = importRun(db, runId)
    totalTrials += trials
    totalRaw += raw
    console.log(`  ${runId}: ${trials} trials, ${raw} raw`)
  }

  console.log(`\nTotal: ${totalTrials} trials, ${totalRaw} raw responses`)
}

function runQuery(db: Database, sql: string): void {
  try {
    const stmt = db.prepare(sql)
    const results = stmt.all()

    if (results.length === 0) {
      console.log('(no results)')
      return
    }

    // Print as table
    const keys = Object.keys(results[0] as object)
    console.log(keys.join('\t'))
    console.log(keys.map(() => '---').join('\t'))

    for (const row of results) {
      const r = row as Record<string, unknown>
      console.log(keys.map(k => String(r[k] ?? '')).join('\t'))
    }

    console.log(`\n(${results.length} rows)`)
  }
  catch (err) {
    console.error('Query error:', err)
  }
}

function showSummary(db: Database): void {
  console.log('\n=== Trial Database Summary ===\n')

  // Total counts
  const totals = db.prepare(`
    SELECT
      COUNT(*) as total_trials,
      COUNT(DISTINCT run_id) as runs,
      COUNT(DISTINCT model) as models,
      COUNT(DISTINCT task_id) as tasks,
      SUM(correct) as passed,
      ROUND(100.0 * SUM(correct) / COUNT(*), 1) as accuracy
    FROM trials
  `).get() as Record<string, number>

  console.log(`Total trials: ${totals.total_trials}`)
  console.log(`Runs: ${totals.runs}`)
  console.log(`Models: ${totals.models}`)
  console.log(`Tasks: ${totals.tasks}`)
  console.log(`Passed: ${totals.passed} (${totals.accuracy}%)`)

  // By framework version
  console.log('\n--- By Framework Version ---')
  const byVersion = db.prepare(`
    SELECT
      framework_version,
      COUNT(*) as trials,
      SUM(correct) as passed,
      ROUND(100.0 * SUM(correct) / COUNT(*), 1) as accuracy
    FROM trials
    GROUP BY framework_version
    ORDER BY framework_version
  `).all() as Array<Record<string, unknown>>

  for (const row of byVersion) {
    console.log(`  ${row.framework_version}: ${row.passed}/${row.trials} (${row.accuracy}%)`)
  }

  // By model (top 10)
  console.log('\n--- By Model (top 10) ---')
  const byModel = db.prepare(`
    SELECT
      model,
      COUNT(*) as trials,
      SUM(correct) as passed,
      ROUND(100.0 * SUM(correct) / COUNT(*), 1) as accuracy
    FROM trials
    GROUP BY model
    ORDER BY accuracy DESC
    LIMIT 10
  `).all() as Array<Record<string, unknown>>

  for (const row of byModel) {
    console.log(`  ${row.model}: ${row.passed}/${row.trials} (${row.accuracy}%)`)
  }

  // By context
  console.log('\n--- By Context ---')
  const byContext = db.prepare(`
    SELECT
      context,
      COUNT(*) as trials,
      SUM(correct) as passed,
      ROUND(100.0 * SUM(correct) / COUNT(*), 1) as accuracy
    FROM trials
    GROUP BY context
    ORDER BY accuracy DESC
  `).all() as Array<Record<string, unknown>>

  for (const row of byContext) {
    console.log(`  ${row.context}: ${row.passed}/${row.trials} (${row.accuracy}%)`)
  }

  // Error distribution
  console.log('\n--- Error Types ---')
  const byError = db.prepare(`
    SELECT
      COALESCE(error_type, 'success') as error_type,
      COUNT(*) as count,
      ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM trials), 1) as pct
    FROM trials
    GROUP BY error_type
    ORDER BY count DESC
  `).all() as Array<Record<string, unknown>>

  for (const row of byError) {
    console.log(`  ${row.error_type}: ${row.count} (${row.pct}%)`)
  }

  console.log('')
}

function showHelp(): void {
  console.log(`
Usage: bun run probationes/harness/db.ts <command> [args]

Commands:
  import [run-id]     Import trial results into SQLite
                      If run-id is omitted, imports all runs

  query "SQL"         Run an ad-hoc SQL query

  summary             Show summary statistics

  help                Show this help message

Examples:
  bun run probationes/harness/db.ts import
  bun run probationes/harness/db.ts import 2026-01-07T22-15-30
  bun run probationes/harness/db.ts query "SELECT model, AVG(correct) FROM trials GROUP BY model"
  bun run probationes/harness/db.ts summary

Database location: ${DB_PATH}

Useful queries:
  -- Accuracy by model and context
  SELECT model, context, ROUND(100.0 * SUM(correct) / COUNT(*), 1) as accuracy
  FROM trials GROUP BY model, context ORDER BY model, accuracy DESC

  -- Hardest tasks
  SELECT task_id, COUNT(*) as attempts, SUM(correct) as passed,
         ROUND(100.0 * SUM(correct) / COUNT(*), 1) as accuracy
  FROM trials GROUP BY task_id ORDER BY accuracy ASC LIMIT 10

  -- Compare framework versions
  SELECT framework_version, model, ROUND(100.0 * SUM(correct) / COUNT(*), 1) as accuracy
  FROM trials GROUP BY framework_version, model ORDER BY framework_version, accuracy DESC

  -- Error patterns for a specific model
  SELECT error_type, COUNT(*) as count FROM trials
  WHERE model = 'gpt-4o-mini' AND correct = 0
  GROUP BY error_type ORDER BY count DESC
`)
}

// Main
const command = process.argv[2]
const arg = process.argv[3]

const db = getDb()
initSchema(db)

switch (command) {
  case 'import':
    if (arg) {
      const { trials, raw } = importRun(db, arg)
      console.log(`Imported: ${trials} trials, ${raw} raw responses`)
    }
    else {
      importAll(db)
    }
    break

  case 'query':
    if (!arg) {
      console.error('Usage: db.ts query "SQL"')
      process.exit(1)
    }
    runQuery(db, arg)
    break

  case 'summary':
    showSummary(db)
    break

  case 'help':
  case '--help':
  case '-h':
  case undefined:
    showHelp()
    break

  default:
    console.error(`Unknown command: ${command}`)
    showHelp()
    process.exit(1)
}

db.close()
