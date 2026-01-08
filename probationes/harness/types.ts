// Core types for the trial harness

export interface Task {
  id: string
  type: TaskType
  category: string
  goal: string                    // Clear statement of what this task tests
  input?: string                  // TypeScript input (for ts_to_faber) or template (for complete)
  input_faber?: string            // Faber input (for faber_to_ts or predict)
  input_faber_file?: string       // Path to Faber file (for large inputs)
  expected_faber?: string         // Expected Faber output (legacy, not used with output grading)
  expected_ts?: string            // Expected TS output (legacy, not used with output grading)
  expected_ts_file?: string       // Path to expected TS file
  expected_output?: string        // Expected stdout from running the code
  expected_completion?: string    // Expected completion text (for complete_code tasks)
}

export type TaskType =
  | 'translate_ts_to_faber'
  | 'translate_faber_to_ts'
  | 'predict_output'
  | 'complete_code'
  | 'explain_code'
  | 'write_typescript'
  | 'write_faber'

export interface Example {
  id: string
  typescript: string
  faber: string
  explanation: string
}

export interface Model {
  id: string
  provider: string
  name: string
  cost_per_1m_input: number
  cost_per_1m_output: number
}

export interface TrialConfig {
  n_shots: number[]
  dialects: string[]
  contexts: string[]
  temperature: number
  max_tokens: number
  seed?: number
}

export interface RawResponse {
  timestamp: string
  run_id: string
  framework_version: string
  git_sha?: string
  task_id: string
  model: string
  n_shot: number
  dialect: string
  context: string
  prompt: string
  response: string
  tokens_in: number
  tokens_out: number
  latency_ms: number
}

export interface GradedResult {
  timestamp: string
  run_id: string
  framework_version: string
  git_sha?: string
  task_id: string
  model: string                 // primary model (drafter in pipeline mode)
  n_shot: number
  dialect: string
  context: string

  // Three-level grading (final result - verifier if present, else drafter)
  level_a: boolean              // typechecks
  level_b: boolean              // runs without error
  level_c: boolean              // correct output (or true if no output expected)
  correct: boolean              // all three levels pass
  error_type?: ErrorType
  error_detail?: string         // stderr, diff, or other diagnostic info

  // Pipeline fields (optional - present when verifier is used)
  verifier_model?: string       // verifier model ID
  drafter_correct?: boolean     // did drafter pass before verification?
  drafter_error_type?: ErrorType
  verifier_correct?: boolean    // did verifier pass? (same as correct when present)
  verifier_error_type?: ErrorType
  transition?: TransitionType   // preserved/damaged/recovered/failed

  // Cost tracking
  drafter_cost?: number
  verifier_cost?: number
  total_cost?: number
  drafter_latency_ms?: number
  verifier_latency_ms?: number
  total_latency_ms?: number

  // Token tracking
  drafter_tokens_in?: number
  drafter_tokens_out?: number
  verifier_tokens_in?: number
  verifier_tokens_out?: number
  total_tokens_in?: number
  total_tokens_out?: number
}

export type ErrorType =
  | 'syntax_error'          // failed to parse
  | 'type_error'            // typecheck failed
  | 'runtime_error'         // execution failed
  | 'wrong_output'          // ran but output incorrect
  | 'no_response'           // LLM returned nothing
  | 'api_error'             // API call failed

export type TransitionType =
  | 'preserved'             // drafter correct, verifier correct
  | 'damaged'               // drafter correct, verifier wrong
  | 'recovered'             // drafter wrong, verifier correct
  | 'failed'                // drafter wrong, verifier wrong

export interface PipelineResult {
  timestamp: string
  run_id: string
  framework_version: string
  git_sha?: string
  task_id: string
  drafter_model: string
  verifier_model: string
  n_shot: number
  dialect: string
  context: string

  // Drafter metrics
  drafter_response: string
  drafter_correct: boolean
  drafter_error_type?: ErrorType
  drafter_tokens_in: number
  drafter_tokens_out: number
  drafter_latency_ms: number
  drafter_cost: number

  // Verifier metrics
  verifier_response: string
  verifier_correct: boolean
  verifier_error_type?: ErrorType
  verifier_tokens_in: number
  verifier_tokens_out: number
  verifier_latency_ms: number
  verifier_cost: number

  // Transition analysis
  transition: TransitionType

  // Totals
  total_tokens_in: number
  total_tokens_out: number
  total_latency_ms: number
  total_cost: number
}

export interface Summary {
  run_id: string
  timestamp: string
  framework_version: string
  git_sha?: string
  config: {
    models: string[]
    n_shots: number[]
    dialects: string[]
    total_tasks: number
  }
  results: Record<string, Record<string, Record<string, {
    accuracy: number
    total: number
    level_a_pass: number    // typecheck pass rate
    level_b_pass: number    // runtime pass rate
    level_c_pass: number    // output correct rate
    errors: Record<ErrorType, number>
  }>>>
  total_cost: number
  total_latency_ms: number
}
