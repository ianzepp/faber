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
 * - compile: Full compilation pipeline from .fab source to target language
 * - run: Compile to TypeScript and execute immediately (TS target only)
 * - check: Validate source for errors without generating code
 *
 * Error handling follows the "never crash on bad input" principle - all
 * compilation errors are collected and reported with file positions before
 * exiting with a non-zero status code.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Command-line arguments (argv), .fab source files from filesystem
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

import { tokenize } from './tokenizer';
import { parse } from './parser';
import { analyze } from './semantic';
import { generate, type CodegenTarget } from './codegen';
import * as prettier from 'prettier';
import prettierPlugin from './prettier/index.ts';

// =============================================================================
// CONSTANTS
// =============================================================================

/**
 * Version string for Faber Romanus compiler.
 * WHY: Hardcoded until we integrate with package.json or build system.
 */
const VERSION = '0.2.0';

/**
 * Default compilation target.
 * WHY: TypeScript is the primary target as it's most accessible and runs
 *      directly via Bun without additional toolchain setup.
 */
const DEFAULT_TARGET: CodegenTarget = 'ts';

/**
 * Valid compilation targets.
 * WHY: Defined as array for validation and help text generation.
 */
const VALID_TARGETS = ['ts', 'zig', 'py', 'rs', 'cpp'] as const;

// =============================================================================
// ARGUMENT PARSING
// =============================================================================

const args = process.argv.slice(2);

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
  faber <command> <file> [options]

Commands:
  compile, finge <file>  Compile .fab file to target language
  run, curre <file>      Compile and execute (TS target only)
  check, proba <file>    Check for errors without compiling
  format, forma <file>   Format source file with Prettier

Options:
  -t, --target <lang>    Target language: ts (default), zig, py, rs, cpp
  -o, --output <file>    Output file (default: stdout)
  -c, --check            Check formatting without writing (format command)
  -h, --help             Show this help
  -v, --version          Show version

Reads from stdin if no file specified (or use '-' explicitly).

Examples:
  faber compile hello.fab                     # Compile to TS (stdout)
  faber compile hello.fab -o hello.ts         # Compile to TS file
  faber compile hello.fab --target zig        # Compile to Zig
  faber compile hello.fab -t py -o hello.py   # Compile to Python
  faber run hello.fab                         # Compile to TS and execute
  faber format hello.fab                      # Format file in place
  faber format hello.fab --check              # Check if file is formatted
  echo 'scribe "hello"' | faber compile        # Compile from stdin
`);
}

// =============================================================================
// INPUT HANDLING
// =============================================================================

/**
 * Read source code from file or stdin.
 *
 * WHY: Unix convention uses '-' to mean stdin, allowing pipeline usage:
 *      echo 'scribe "hi"' | faber compile -
 *
 * @param inputFile - Path to file, or '-' for stdin
 * @returns Source code as string
 */
async function readSource(inputFile: string): Promise<string> {
    if (inputFile === '-') {
        // Read from stdin
        const chunks: Uint8Array[] = [];
        for await (const chunk of Bun.stdin.stream()) {
            chunks.push(chunk);
        }
        const decoder = new TextDecoder();
        return chunks.map(c => decoder.decode(c)).join('');
    }
    return Bun.file(inputFile).text();
}

/**
 * Get display name for error messages.
 */
function getDisplayName(inputFile: string): string {
    return inputFile === '-' ? '<stdin>' : inputFile;
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
 * @param inputFile - Path to .fab source file
 * @param target - Compilation target language
 * @param outputFile - Optional output file path (defaults to stdout)
 * @param silent - If true, don't print to stdout (for use by run command)
 * @returns Generated source code as string
 */
async function compile(inputFile: string, target: CodegenTarget, outputFile?: string, silent = false): Promise<string> {
    const source = await readSource(inputFile);
    const displayName = getDisplayName(inputFile);

    // ---------------------------------------------------------------------------
    // Lexical Analysis
    // ---------------------------------------------------------------------------

    const { tokens, errors: tokenErrors } = tokenize(source);

    if (tokenErrors.length > 0) {
        console.error('Tokenizer errors:');
        for (const err of tokenErrors) {
            console.error(`  ${displayName}:${err.position.line}:${err.position.column} - ${err.text}`);
        }

        process.exit(1);
    }

    // ---------------------------------------------------------------------------
    // Syntactic Analysis
    // ---------------------------------------------------------------------------

    const { program, errors: parseErrors } = parse(tokens);

    if (parseErrors.length > 0) {
        console.error('Parser errors:');
        for (const err of parseErrors) {
            console.error(`  ${displayName}:${err.position.line}:${err.position.column} - ${err.message}`);
        }

        process.exit(1);
    }

    // EDGE: Parser can return null program on catastrophic failure
    if (!program) {
        console.error('Failed to parse program');
        process.exit(1);
    }

    // ---------------------------------------------------------------------------
    // Semantic Analysis
    // ---------------------------------------------------------------------------

    const { errors: semanticErrors } = analyze(program);

    if (semanticErrors.length > 0) {
        console.error('Semantic errors:');
        for (const err of semanticErrors) {
            console.error(`  ${displayName}:${err.position.line}:${err.position.column} - ${err.message}`);
        }

        process.exit(1);
    }

    // ---------------------------------------------------------------------------
    // Code Generation
    // ---------------------------------------------------------------------------

    const output = generate(program, { target });

    if (outputFile) {
        await Bun.write(outputFile, output);
        console.log(`Compiled: ${displayName} -> ${outputFile} (${target})`);
    } else if (!silent) {
        // WHY: Write to stdout for Unix pipeline compatibility
        console.log(output);
    }

    return output;
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
 * @param inputFile - Path to .fab source file
 */
async function run(inputFile: string): Promise<void> {
    const ts = await compile(inputFile, 'ts', undefined, true);

    // WHY: Bun can execute TypeScript directly - write to temp file and run
    const tempFile = `/tmp/faber-${Date.now()}.ts`;

    try {
        await Bun.write(tempFile, ts);
        const proc = Bun.spawn(['bun', tempFile], {
            stdout: 'inherit',
            stderr: 'inherit',
        });
        const exitCode = await proc.exited;

        if (exitCode !== 0) {
            process.exit(exitCode);
        }
    } catch (err) {
        console.error('Runtime error:', err);
        process.exit(1);
    } finally {
        // Clean up temp file
        (await Bun.file(tempFile).exists()) && (await Bun.write(tempFile, ''));
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
 * @param inputFile - Path to .fab source file
 */
async function check(inputFile: string): Promise<void> {
    const source = await readSource(inputFile);
    const displayName = getDisplayName(inputFile);

    const { tokens, errors: tokenErrors } = tokenize(source);
    const { program, errors: parseErrors } = parse(tokens);

    let semanticErrors: { message: string; position: { line: number; column: number } }[] = [];

    if (program) {
        const result = analyze(program);

        semanticErrors = result.errors;
    }

    // WHY: Normalize error formats - tokenizer uses 'text', others use 'message'
    const normalizedTokenErrors = tokenErrors.map(e => ({
        message: e.text,
        position: e.position,
    }));
    const allErrors = [...normalizedTokenErrors, ...parseErrors, ...semanticErrors];

    if (allErrors.length === 0) {
        console.log(`${displayName}: No errors`);
    } else {
        console.log(`${displayName}: ${allErrors.length} error(s)`);
        for (const err of allErrors) {
            console.log(`  ${err.position.line}:${err.position.column} - ${err.message}`);
        }

        process.exit(1);
    }
}

/**
 * Format source file using Prettier with the Faber plugin.
 *
 * FORMATTING: Uses the Prettier plugin defined in fons/prettier/ to format
 *             .fab files with consistent style (4-space indent, Stroustrup braces).
 *
 * MODES:
 * - Default: Format file in place
 * - Check: Verify formatting without writing (for CI)
 *
 * @param inputFile - Path to .fab source file
 * @param checkOnly - If true, check formatting without writing
 */
async function format(inputFile: string, checkOnly: boolean): Promise<void> {
    const source = await readSource(inputFile);
    const displayName = getDisplayName(inputFile);
    const isStdin = inputFile === '-';

    try {
        const formatted = await prettier.format(source, {
            parser: 'faber',
            plugins: [prettierPlugin],
            tabWidth: 4,
            useTabs: false,
            printWidth: 100,
        });

        if (isStdin) {
            // For stdin, just output the formatted code
            console.log(formatted);
        } else if (checkOnly) {
            if (source === formatted) {
                console.log(`${displayName}: Formatted`);
            } else {
                console.log(`${displayName}: Needs formatting`);
                process.exit(1);
            }
        } else {
            if (source === formatted) {
                console.log(`${displayName}: Already formatted`);
            } else {
                await Bun.write(inputFile, formatted);
                console.log(`${displayName}: Formatted`);
            }
        }
    } catch (err) {
        console.error(`${displayName}: Format error`);
        if (err instanceof Error) {
            console.error(`  ${err.message}`);
        }
        process.exit(1);
    }
}

// =============================================================================
// COMMAND DISPATCH
// =============================================================================

const command = args[0];

// ---------------------------------------------------------------------------
// Help and Version
// ---------------------------------------------------------------------------

if (!command || command === '-h' || command === '--help') {
    printUsage();
    process.exit(0);
}

if (command === '-v' || command === '--version') {
    console.log(`Faber Romanus v${VERSION}`);
    process.exit(0);
}

// ---------------------------------------------------------------------------
// Option Parsing
// ---------------------------------------------------------------------------

let inputFile: string | undefined;
let outputFile: string | undefined;
let target: CodegenTarget = DEFAULT_TARGET;
let checkOnly = false;

// WHY: Scan all args, options can appear anywhere, non-option is the file
for (let i = 1; i < args.length; i++) {
    const arg = args[i];

    if (arg === '-o' || arg === '--output') {
        outputFile = args[++i];
    }
    else if (arg === '-t' || arg === '--target') {
        const t = args[++i];

        if (!t || !VALID_TARGETS.includes(t as (typeof VALID_TARGETS)[number])) {
            console.error(`Error: Unknown target '${t}'. Valid targets: ${VALID_TARGETS.join(', ')}`);
            process.exit(1);
        }

        target = t as CodegenTarget;
    }
    else if (arg === '-c' || arg === '--check') {
        checkOnly = true;
    }
    else if (!arg.startsWith('-') || arg === '-') {
        // Non-option arg is the file, or explicit '-' for stdin
        inputFile = arg;
    }
    else {
        console.error(`Error: Unknown option '${arg}'`);
        process.exit(1);
    }
}

// WHY: Default to stdin when no file specified, enabling: echo 'code' | faber compile
const effectiveInputFile = inputFile ?? '-';

// ---------------------------------------------------------------------------
// Command Execution
// ---------------------------------------------------------------------------

switch (command) {
    case 'compile':
    case 'finge':
        await compile(effectiveInputFile, target, outputFile);
        break;
    case 'run':
    case 'curre':
        if (target !== 'ts') {
            console.error("Error: 'run' command only works with TS target");
            process.exit(1);
        }

        await run(effectiveInputFile);
        break;
    case 'check':
    case 'proba':
        await check(effectiveInputFile);
        break;
    case 'format':
    case 'forma':
        await format(effectiveInputFile, checkOnly);
        break;
    default:
        console.error(`Unknown command: ${command}`);
        printUsage();
        process.exit(1);
}
