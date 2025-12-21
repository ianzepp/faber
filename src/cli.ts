#!/usr/bin/env bun

/**
 * CLI - Command-line interface for Faber Romanus compiler
 *
 * COMPILER PHASE
 * ==============
 * Driver/orchestration - coordinates lexical, syntactic, and codegen phases
 *
 * ARCHITECTURE
 * ============
 * This module serves as the main entry point for the Faber Romanus compiler.
 * It orchestrates the compilation pipeline by invoking the tokenizer, parser,
 * and code generator in sequence, collecting errors at each phase.
 *
 * The CLI provides three primary commands:
 * - compile: Full compilation pipeline from .la source to target language
 * - run: Compile to TypeScript and execute immediately (TS target only)
 * - check: Validate source for errors without generating code
 *
 * Error handling follows the "never crash on bad input" principle - all
 * compilation errors are collected and reported with file positions before
 * exiting with a non-zero status code.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Command-line arguments (argv), .la source files from filesystem
 * OUTPUT: Generated target language source (stdout or file), error messages (stderr)
 * ERRORS: Tokenizer errors, parser errors, file I/O errors, invalid arguments
 *
 * INVARIANTS
 * ==========
 * INV-1: All compilation errors include file position (line:column)
 * INV-2: Process exits with code 1 on any compilation or runtime error
 * INV-3: Stdout is clean (only generated code or help text), errors go to stderr
 *
 * @module cli
 */

import { tokenize } from "./tokenizer"
import { parse } from "./parser"
import { generate, type CodegenTarget } from "./codegen"

// =============================================================================
// CONSTANTS
// =============================================================================

/**
 * Version string for Faber Romanus compiler.
 * WHY: Hardcoded until we integrate with package.json or build system.
 */
const VERSION = "0.2.0"

/**
 * Default compilation target.
 * WHY: TypeScript is the primary target as it's most accessible and runs
 *      directly via Bun without additional toolchain setup.
 */
const DEFAULT_TARGET: CodegenTarget = "ts"

/**
 * Valid compilation targets.
 * WHY: Defined as array for validation and help text generation.
 */
const VALID_TARGETS = ["ts", "zig"] as const

// =============================================================================
// ARGUMENT PARSING
// =============================================================================

const args = process.argv.slice(2)

// =============================================================================
// HELP AND VERSION
// =============================================================================

/**
 * Display usage information to stdout.
 *
 * OUTPUT FORMAT: Follows standard Unix conventions with commands first,
 *                then options, then examples.
 */
function printUsage(): void {
  console.log(`
Faber Romanus - The Roman Craftsman
A Latin programming language

Usage:
  faber <command> [options] <file>

Commands:
  compile <file.la>     Compile .la file to target language
  run <file.la>         Compile and execute (TS target only)
  check <file.la>       Check for errors without compiling

Options:
  -t, --target <lang>    Target language: ts (default), zig
  -o, --output <file>    Output file (default: stdout)
  -h, --help             Show this help
  -v, --version          Show version

Examples:
  faber compile hello.la                    # Compile to TS (stdout)
  faber compile hello.la -o hello.ts        # Compile to TS file
  faber compile hello.la --target zig       # Compile to Zig (stdout)
  faber compile hello.la -t zig -o hello.zig
  faber run hello.la                        # Compile to TS and execute
`)
}

// =============================================================================
// COMPILATION PIPELINE
// =============================================================================

/**
 * Execute full compilation pipeline: tokenize -> parse -> generate.
 *
 * PIPELINE STAGES:
 * 1. Tokenize: Source text -> token stream
 * 2. Parse: Tokens -> AST
 * 3. Generate: AST -> target language source
 *
 * ERROR HANDLING: Errors from each stage are collected and reported with
 *                 file positions. Process exits with code 1 on first error.
 *
 * TARGET DIFFERENCES:
 * - TS: Generates TypeScript with type annotations
 * - Zig: Generates Zig with explicit return types and error handling
 *
 * @param inputFile - Path to .la source file
 * @param target - Compilation target language
 * @param outputFile - Optional output file path (defaults to stdout)
 * @returns Generated source code as string
 */
async function compile(inputFile: string, target: CodegenTarget, outputFile?: string): Promise<string> {
  const source = await Bun.file(inputFile).text()

  // ---------------------------------------------------------------------------
  // Lexical Analysis
  // ---------------------------------------------------------------------------

  const { tokens, errors: tokenErrors } = tokenize(source)
  if (tokenErrors.length > 0) {
    console.error("Tokenizer errors:")
    for (const err of tokenErrors) {
      console.error(`  ${inputFile}:${err.position.line}:${err.position.column} - ${err.message}`)
    }
    process.exit(1)
  }

  // ---------------------------------------------------------------------------
  // Syntactic Analysis
  // ---------------------------------------------------------------------------

  const { program, errors: parseErrors } = parse(tokens)
  if (parseErrors.length > 0) {
    console.error("Parser errors:")
    for (const err of parseErrors) {
      console.error(`  ${inputFile}:${err.position.line}:${err.position.column} - ${err.message}`)
    }
    process.exit(1)
  }

  // EDGE: Parser can return null program on catastrophic failure
  if (!program) {
    console.error("Failed to parse program")
    process.exit(1)
  }

  // ---------------------------------------------------------------------------
  // Code Generation
  // ---------------------------------------------------------------------------

  const output = generate(program, { target })

  if (outputFile) {
    await Bun.write(outputFile, output)
    console.log(`Compiled: ${inputFile} -> ${outputFile} (${target})`)
  }
  else {
    // WHY: Write to stdout for Unix pipeline compatibility
    console.log(output)
  }

  return output
}

/**
 * Compile and immediately execute TypeScript output.
 *
 * RUNTIME: Uses Bun's native TypeScript execution capability via Function constructor.
 *
 * SAFETY: Generated code is executed in same context as CLI - no sandboxing.
 *         This is acceptable for a dev tool but would need isolation for production use.
 *
 * TARGET RESTRICTION: Only works with TypeScript target since Zig requires
 *                     separate compilation and linking.
 *
 * @param inputFile - Path to .la source file
 */
async function run(inputFile: string): Promise<void> {
  const ts = await compile(inputFile, "ts")

  // WHY: Bun can execute TypeScript directly without transpilation step
  try {
    const fn = new Function(ts)
    fn()
  }
  catch (err) {
    console.error("Runtime error:", err)
    process.exit(1)
  }
}

/**
 * Validate source file for errors without generating code.
 *
 * PHASES RUN: Tokenizer and parser only (skips codegen for performance)
 *
 * USE CASE: Fast syntax validation in editor plugins or pre-commit hooks
 *
 * OUTPUT: Reports error count and positions, exits 0 if no errors
 *
 * @param inputFile - Path to .la source file
 */
async function check(inputFile: string): Promise<void> {
  const source = await Bun.file(inputFile).text()

  const { tokens, errors: tokenErrors } = tokenize(source)
  const { program, errors: parseErrors } = parse(tokens)

  const allErrors = [...tokenErrors, ...parseErrors]

  if (allErrors.length === 0) {
    console.log(`${inputFile}: No errors`)
  }
  else {
    console.log(`${inputFile}: ${allErrors.length} error(s)`)
    for (const err of allErrors) {
      console.log(`  ${err.position.line}:${err.position.column} - ${err.message}`)
    }
    process.exit(1)
  }
}

// =============================================================================
// COMMAND DISPATCH
// =============================================================================

const command = args[0]

// ---------------------------------------------------------------------------
// Help and Version
// ---------------------------------------------------------------------------

if (!command || command === "-h" || command === "--help") {
  printUsage()
  process.exit(0)
}

if (command === "-v" || command === "--version") {
  console.log(`Faber Romanus v${VERSION}`)
  process.exit(0)
}

// ---------------------------------------------------------------------------
// Option Parsing
// ---------------------------------------------------------------------------

const inputFile = args[1]
let outputFile: string | undefined
let target: CodegenTarget = DEFAULT_TARGET

// WHY: Simple linear scan is sufficient for small option set
for (let i = 2; i < args.length; i++) {
  if (args[i] === "-o" || args[i] === "--output") {
    outputFile = args[++i]
  }
  else if (args[i] === "-t" || args[i] === "--target") {
    const t = args[++i]
    if (t !== "ts" && t !== "zig") {
      console.error(`Error: Unknown target '${t}'. Valid targets: ${VALID_TARGETS.join(", ")}`)
      process.exit(1)
    }
    target = t
  }
}

if (!inputFile) {
  console.error("Error: No input file specified")
  printUsage()
  process.exit(1)
}

// ---------------------------------------------------------------------------
// Command Execution
// ---------------------------------------------------------------------------

switch (command) {
  case "compile":
    await compile(inputFile, target, outputFile)
    break
  case "run":
    if (target !== "ts") {
      console.error("Error: 'run' command only works with TS target")
      process.exit(1)
    }
    await run(inputFile)
    break
  case "check":
    await check(inputFile)
    break
  default:
    console.error(`Unknown command: ${command}`)
    printUsage()
    process.exit(1)
}
