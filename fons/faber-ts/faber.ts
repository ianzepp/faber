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
 * - emit: Full compilation pipeline from .fab source to target language
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

import { resolve, dirname, join, relative } from 'node:path';
import { mkdir } from 'node:fs/promises';
import { realpathSync } from 'node:fs';
import type { Program } from './parser/ast';
import { tokenize } from './tokenizer';
import { parse } from './parser';
import { analyze } from './semantic';
import { generate } from './codegen';

// =============================================================================
// CONSTANTS
// =============================================================================

/**
 * Version string for Faber Romanus compiler.
 * WHY: Hardcoded until we integrate with package.json or build system.
 */
const VERSION = '0.2.0';

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
A Latin programming language (TypeScript target)

Usage:
  <source> | faber <command> [options]
  faber build <file> [options]

Commands:
  emit, compile          Emit stdin as TypeScript
  run, curre             Compile stdin and execute immediately
  check, proba           Check stdin for errors without generating code
  format, forma          Format stdin with Prettier
  build, aedifica <file> Build entry file + dependencies to directory

Options:
  -o, --output <file>       Output file (default: stdout)
  -t, --target ts           Target language (only 'ts' supported)
  -c, --check               Check formatting without writing (format command)
  --strip-tests             Strip test blocks (probandum/proba) from output
  --stdin-filename <file>   Filename for error messages (default: <stdin>)
  -h, --help                Show this help
  -v, --version             Show version

For other targets (Python, Rust, Zig, C++), use the Rivus compiler.

Examples:
  cat hello.fab | faber emit                              # Emit as TS (stdout)
  cat hello.fab | faber emit -o hello.ts                  # Emit to TS file
  cat hello.fab | faber emit --stdin-filename hello.fab   # With filename in errors
  faber build main.fab -o dist/                           # Build entry + deps to dist/
  cat hello.fab | faber run                               # Compile and execute
  cat hello.fab | faber check                             # Check for parse/semantic errors
`);
}

// =============================================================================
// INPUT HANDLING
// =============================================================================

/**
 * Read source code from stdin.
 *
 * @returns Source code as string
 */
async function readStdin(): Promise<string> {
    const chunks: Uint8Array[] = [];
    for await (const chunk of Bun.stdin.stream()) {
        chunks.push(chunk);
    }
    const decoder = new TextDecoder();
    return chunks.map(c => decoder.decode(c)).join('');
}

/**
 * Read source code from file.
 *
 * @param filePath - Path to file
 * @returns Source code as string
 */
async function readFile(filePath: string): Promise<string> {
    return Bun.file(filePath).text();
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
 * @param displayName - Filename for error messages
 * @param outputFile - Optional output file path (defaults to stdout)
 * @param options - Emit options
 * @returns Generated TypeScript source code as string
 */
async function emit(displayName: string, outputFile?: string, options: { silent?: boolean; stripTests?: boolean } = {}): Promise<string> {
    const { silent = false, stripTests = false } = options;
    const source = await readStdin();

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

    // WHY: No file path for stdin - import resolution not supported
    const { errors: semanticErrors, subsidiaImports, resolvedModules } = analyze(program, { filePath: undefined });

    if (semanticErrors.length > 0) {
        console.error('Semantic errors:');
        for (const err of semanticErrors) {
            const errorFile = err.filePath ?? displayName;
            console.error(`  ${errorFile}:${err.position.line}:${err.position.column} - ${err.message}`);
        }

        process.exit(1);
    }

    // ---------------------------------------------------------------------------
    // Code Generation
    // ---------------------------------------------------------------------------

    let output: string;
    try {
        // WHY: No filePath for stdin - CLI module resolution not available
        // WHY: Pass subsidiaImports to enable HAL import resolution
        // WHY: keepRelativeImports=true because output directory structure mirrors source
        output = generate(program, { filePath: undefined, subsidiaImports, keepRelativeImports: true, stripTests });
    } catch (err) {
        // WHY: Codegen errors should display cleanly
        const message = err instanceof Error ? err.message : String(err);
        console.error(`Codegen error: ${message}`);
        process.exit(1);
    }

    if (outputFile) {
        await Bun.write(outputFile, output);
        console.log(`Compiled: ${displayName} -> ${outputFile}`);
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
 * @param displayName - Filename for error messages
 */
async function run(displayName: string): Promise<void> {
    const ts = await emit(displayName, undefined, { silent: true });

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
 * Build entry point and all dependencies to output directory.
 *
 * PIPELINE: For each local dependency discovered during semantic analysis,
 *           compile to target language and write to output directory
 *           preserving relative path structure.
 *
 * OUTPUT STRUCTURE:
 *   Given entry fons/cli/main.fab with import ./commands/greet.fab:
 *   dist/
 *     main.ts
 *     commands/
 *       greet.ts
 *
 * @param inputFile - Path to entry .fab source file
 * @param outputDir - Directory to write compiled files
 */

/**
 * Read source file content
 */
async function readSource(filePath: string): Promise<string> {
    return Bun.file(filePath).text();
}

/**
 * Get display name for file (used in error messages)
 */
function getDisplayName(filePath: string): string {
    return filePath;
}

async function build(inputFile: string, outputDir: string): Promise<void> {
    const entryPath = realpathSync(resolve(inputFile));
    const projectRoot = process.cwd();

    // Compile entry file and collect all resolved modules
    const source = await readSource(inputFile);
    const displayName = getDisplayName(inputFile);

    const { tokens, errors: tokenErrors } = tokenize(source);
    if (tokenErrors.length > 0) {
        console.error('Tokenizer errors:');
        for (const err of tokenErrors) {
            console.error(`  ${displayName}:${err.position.line}:${err.position.column} - ${err.text}`);
        }
        process.exit(1);
    }

    const { program, errors: parseErrors } = parse(tokens);
    if (parseErrors.length > 0) {
        console.error('Parser errors:');
        for (const err of parseErrors) {
            console.error(`  ${displayName}:${err.position.line}:${err.position.column} - ${err.message}`);
        }
        process.exit(1);
    }

    if (!program) {
        console.error('Failed to parse program');
        process.exit(1);
    }

    const { errors: semanticErrors, subsidiaImports, resolvedModules } = analyze(program, { filePath: entryPath });
    if (semanticErrors.length > 0) {
        console.error('Semantic errors:');
        for (const err of semanticErrors) {
            console.error(`  ${displayName}:${err.position.line}:${err.position.column} - ${err.message}`);
        }
        process.exit(1);
    }

    // Create output directory
    await mkdir(outputDir, { recursive: true });

    // Compile and write entry file
    // WHY: keepRelativeImports=true because output directory structure mirrors source
    const entryOutput = generate(program, { filePath: entryPath, subsidiaImports, keepRelativeImports: true });
    const entryRelPath = relative(projectRoot, entryPath);
    const entryOutPath = join(outputDir, entryRelPath.replace(/\.fab$/, '.ts'));
    await mkdir(dirname(entryOutPath), { recursive: true });
    await Bun.write(entryOutPath, entryOutput);
    console.log(`  ${displayName} -> ${entryOutPath}`);

    // Compile and write each dependency
    for (const [depPath, depProgram] of resolvedModules) {
        // WHY: Dependency codegen requires semantic types (e.g., ego.field -> lista)
        const depAnalysis = analyze(depProgram, { filePath: depPath });
        if (depAnalysis.errors.length > 0) {
            console.error('Semantic errors:');
            for (const err of depAnalysis.errors) {
                console.error(`  ${depPath}:${err.position.line}:${err.position.column} - ${err.message}`);
            }
            process.exit(1);
        }

        // Calculate relative path from project root (canonicalize to resolve symlinks)
        const canonicalDepPath = realpathSync(depPath);
        const relPath = relative(projectRoot, canonicalDepPath);
        const outPath = join(outputDir, relPath.replace(/\.fab$/, '.ts'));

        // Ensure subdirectory exists
        await mkdir(dirname(outPath), { recursive: true });

        // Generate code for dependency
        const depOutput = generate(depProgram, {
            filePath: depPath,
            subsidiaImports: depAnalysis.subsidiaImports,
            keepRelativeImports: true,
        });
        await Bun.write(outPath, depOutput);
        console.log(`  ${relPath} -> ${outPath}`);
    }

    console.log(`\nBuild complete: ${1 + resolvedModules.size} file(s) written to ${outputDir}`);
}

/**
 * Validate source for errors without generating code.
 *
 * PHASES RUN: Tokenizer, parser, and semantic analysis.
 *
 * USE CASE: Fast syntax validation in editor plugins or pre-commit hooks.
 *
 * OUTPUT: Reports error count and positions, exits 0 if no errors
 *
 * @param displayName - Filename for error messages
 */
async function check(displayName: string): Promise<void> {
    const source = await readStdin();

    const { tokens, errors: tokenErrors } = tokenize(source);
    const { program, errors: parseErrors } = parse(tokens);

    let semanticErrors: { message: string; position: { line: number; column: number }; filePath?: string }[] = [];

    if (program) {
        // WHY: No file path for stdin - import resolution not supported
        const result = analyze(program, { filePath: undefined });

        semanticErrors = result.errors;
    }

    // WHY: Normalize error formats - tokenizer uses 'text', others use 'message'
    const normalizedTokenErrors = tokenErrors.map(e => ({
        message: e.text,
        position: e.position,
    }));
    const allErrors = [...normalizedTokenErrors, ...parseErrors, ...semanticErrors];

    if (allErrors.length > 0) {
        console.log(`${displayName}: ${allErrors.length} error(s)`);
        for (const err of allErrors) {
            const errorFile = 'filePath' in err && err.filePath ? err.filePath : displayName;
            console.log(`  ${errorFile}:${err.position.line}:${err.position.column} - ${err.message}`);
        }

        process.exit(1);
    }

    console.log(`${displayName}: No errors`);
}

/**
 * Format source using Prettier with the Faber plugin.
 *
 * FORMATTING: Uses the Prettier plugin defined in fons/prettier/ to format
 *             .fab files with consistent style (4-space indent, Stroustrup braces).
 *
 * MODES:
 * - Default: Format and output to stdout
 * - Check: Verify formatting without writing (for CI)
 *
 * @param displayName - Filename for error messages
 * @param checkOnly - If true, check formatting without writing
 */
async function format(_displayName: string, _checkOnly: boolean): Promise<void> {
    console.error('Format command is temporarily disabled (prettier plugin archived)');
    process.exit(1);
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

let inputFile: string | undefined;  // Only used by build command
let outputFile: string | undefined;
let stdinFilename = '<stdin>';
let checkOnly = false;
let stripTests = false;

for (let i = 1; i < args.length; i++) {
    const arg = args[i]!;

    if (arg === '-o' || arg === '--output') {
        outputFile = args[++i];
    } else if (arg === '-t' || arg === '--target') {
        const target = args[++i];
        if (target !== 'ts') {
            console.error(`Error: Faber only supports TypeScript target. Use Rivus for other targets.`);
            process.exit(1);
        }
        // WHY: Accept -t ts for compatibility with build scripts, but ignore since ts is the only target
    } else if (arg === '-c' || arg === '--check') {
        checkOnly = true;
    } else if (arg === '--strip-tests') {
        stripTests = true;
    } else if (arg === '--stdin-filename') {
        stdinFilename = args[++i] ?? '<stdin>';
    } else if (arg.startsWith('-')) {
        console.error(`Error: Unknown option '${arg}'`);
        process.exit(1);
    } else {
        // Positional arg - only used by build command
        inputFile = arg;
    }
}

// ---------------------------------------------------------------------------
// Command Execution
// ---------------------------------------------------------------------------

switch (command) {
    case 'emit':
    case 'compile':
    case 'finge':
        await emit(stdinFilename, outputFile, { stripTests });
        break;
    case 'run':
    case 'curre':
        await run(stdinFilename);
        break;
    case 'check':
    case 'proba':
        await check(stdinFilename);
        break;
    case 'build':
    case 'aedifica':
        if (!inputFile) {
            console.error('Error: build command requires an input file');
            process.exit(1);
        }
        await build(inputFile, outputFile ?? './dist');
        break;
    case 'format':
    case 'forma':
        await format(stdinFilename, checkOnly);
        break;
    default:
        console.error(`Unknown command: ${command}`);
        printUsage();
        process.exit(1);
}
