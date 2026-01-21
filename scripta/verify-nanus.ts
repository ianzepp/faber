#!/usr/bin/env bun
/**
 * verify-nanus: Cross-compiler consistency check for nanus compilers.
 *
 * Verifies that nanus-ts, nanus-go, and nanus-rs produce identical output
 * when emitting Faber source (`-t fab`). This catches implementation drift
 * between the three bootstrap compilers.
 *
 * How it works:
 *   1. For each .fab file, pipe through all three nanus compilers with `-t fab`
 *   2. Compare outputs across compilers (not against original source)
 *   3. Report files where outputs differ
 *
 * Failure modes:
 *   - "all compilers failed": File uses syntax nanus doesn't support (expected
 *     for advanced features like `ab`, destructuring, `ergo`, etc.)
 *   - "output mismatch": All compilers parsed successfully but produced
 *     different output (indicates a bug in one or more emitters)
 *   - "<compiler> failed": Only some compilers failed (parser inconsistency)
 *
 * Usage:
 *   bun run verify:nanus                       # bulk check all fons/exempla/
 *   bun run verify:nanus path/to/file.fab     # single file (shows all outputs)
 *   bun run verify:nanus --diff                # show line-by-line diffs on mismatch
 *   bun run verify:nanus -x                    # exit on first failure
 *   bun run verify:nanus -q                    # quiet: single-line output, skip passes
 *   bun run verify:nanus -q -x                 # combined: compact + fail fast
 *
 * Single file mode:
 *   Shows the fab output from each compiler side-by-side, useful for debugging
 *   specific differences. Example:
 *     bun run verify:nanus fons/exempla/si/si.fab
 *
 * Exit codes:
 *   0 - All checked files consistent
 *   1 - At least one mismatch or failure
 *
 * Known limitations:
 *   - nanus compilers only support a subset of Faber (enough to compile rivus)
 *   - Files using unsupported syntax will show "all compilers failed"
 *   - Whitespace/indentation differences count as mismatches
 */

import { readdir, stat } from 'fs/promises';
import { join, relative, isAbsolute } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const EXEMPLA = join(ROOT, 'fons', 'exempla');
const BIN = join(ROOT, 'opus', 'bin');

const COMPILERS = ['nanus-ts', 'nanus-go', 'nanus-rs'] as const;
type Compiler = (typeof COMPILERS)[number];

interface Args {
    file: string | null;
    showDiffs: boolean;
    quiet: boolean;
    failFast: boolean;
}

interface CompileResult {
    compiler: Compiler;
    success: boolean;
    output?: string;
    error?: string;
}

interface FileResult {
    file: string;
    results: CompileResult[];
    consistent: boolean;
    error?: string;
}

function printHelp(): void {
    console.log(`verify-nanus: Cross-compiler consistency check for nanus compilers.

Verifies that nanus-ts, nanus-go, and nanus-rs produce identical output
when emitting Faber source (\`-t fab\`).

Usage:
  bun run verify:nanus                     Bulk check all fons/exempla/
  bun run verify:nanus <file.fab>          Single file (shows all outputs)

Options:
  --diff          Show line-by-line diffs on mismatch
  -x, --fail-fast Exit on first failure
  -q, --quiet     Single-line output, skip passes
  -h, --help      Show this help message

Exit codes:
  0  All checked files consistent
  1  At least one mismatch or failure`);
}

function parseArgs(): Args {
    const args = process.argv.slice(2);
    let file: string | null = null;
    let showDiffs = false;
    let quiet = false;
    let failFast = false;

    for (const arg of args) {
        if (arg === '--help' || arg === '-h') {
            printHelp();
            process.exit(0);
        } else if (arg === '--diff') showDiffs = true;
        else if (arg === '--quiet' || arg === '-q') quiet = true;
        else if (arg === '--fail-fast' || arg === '-x') failFast = true;
        else if (!arg.startsWith('-')) file = arg;
    }

    return { file, showDiffs, quiet, failFast };
}

async function findFiles(dir: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        const fullPath = join(dir, entry);
        const s = await stat(fullPath);
        if (s.isDirectory()) {
            files.push(...(await findFiles(fullPath)));
        } else if (entry.endsWith('.fab')) {
            files.push(fullPath);
        }
    }

    return files.sort();
}

async function compile(compiler: Compiler, fabPath: string): Promise<CompileResult> {
    const bin = join(BIN, compiler);

    if (!(await Bun.file(bin).exists())) {
        return { compiler, success: false, error: `binary not found` };
    }

    try {
        const result = await $`cat ${fabPath} | ${bin} emit -t fab`.quiet();
        return { compiler, success: true, output: result.text().trim() };
    } catch (err: any) {
        return {
            compiler,
            success: false,
            error: err.stderr?.toString().trim() || err.message,
        };
    }
}

function showDiff(a: string, b: string, compilerA: string, compilerB: string): void {
    const linesA = a.split('\n');
    const linesB = b.split('\n');
    const maxLines = Math.max(linesA.length, linesB.length);

    console.log(`      ${compilerA} vs ${compilerB}:`);
    for (let i = 0; i < maxLines; i++) {
        if (linesA[i] !== linesB[i]) {
            console.log(`        L${i + 1}:`);
            console.log(`          \x1b[31m- ${JSON.stringify(linesA[i] ?? '(missing)')}\x1b[0m`);
            console.log(`          \x1b[32m+ ${JSON.stringify(linesB[i] ?? '(missing)')}\x1b[0m`);
        }
    }
}

async function checkFile(fabPath: string): Promise<FileResult> {
    const file = relative(EXEMPLA, fabPath);
    const results: CompileResult[] = [];

    // Compile with each compiler
    for (const compiler of COMPILERS) {
        results.push(await compile(compiler, fabPath));
    }

    // Check if all succeeded
    const succeeded = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);

    if (succeeded.length === 0) {
        return {
            file,
            results,
            consistent: false,
            error: 'all compilers failed',
        };
    }

    if (failed.length > 0 && succeeded.length > 0) {
        return {
            file,
            results,
            consistent: false,
            error: `${failed.map(f => f.compiler).join(', ')} failed`,
        };
    }

    // Compare outputs - all must match
    const outputs = succeeded.map(r => r.output!);
    const allMatch = outputs.every(o => o === outputs[0]);

    return {
        file,
        results,
        consistent: allMatch,
        error: allMatch ? undefined : 'output mismatch',
    };
}

async function runSingleFile(fabPath: string, showDiffs: boolean): Promise<void> {
    // Resolve path
    const fullPath = isAbsolute(fabPath) ? fabPath : join(process.cwd(), fabPath);

    if (!(await Bun.file(fullPath).exists())) {
        console.error(`File not found: ${fabPath}`);
        process.exit(1);
    }

    // Check binaries exist
    const missing: string[] = [];
    for (const compiler of COMPILERS) {
        const bin = join(BIN, compiler);
        if (!(await Bun.file(bin).exists())) {
            missing.push(compiler);
        }
    }

    if (missing.length > 0) {
        console.error(`Missing binaries: ${missing.join(', ')}`);
        console.error('Run `bun run build` first.');
        process.exit(1);
    }

    console.log(`Checking: ${fabPath}\n`);

    // Compile with each compiler and show output
    for (const compiler of COMPILERS) {
        const result = await compile(compiler, fullPath);
        if (result.success) {
            console.log(`--- ${compiler} ---`);
            console.log(result.output);
            console.log('');
        } else {
            console.log(`--- ${compiler} (FAILED) ---`);
            console.log(result.error);
            console.log('');
        }
    }

    // Check consistency
    const fileResult = await checkFile(fullPath);
    if (fileResult.consistent) {
        console.log('\x1b[32mOK\x1b[0m - all compilers produce identical output');
    } else {
        console.log(`\x1b[31mFAIL\x1b[0m - ${fileResult.error}`);
        if (showDiffs && fileResult.error === 'output mismatch') {
            const succeeded = fileResult.results.filter(r => r.success);
            for (let i = 1; i < succeeded.length; i++) {
                if (succeeded[i].output !== succeeded[0].output) {
                    showDiff(
                        succeeded[0].output!,
                        succeeded[i].output!,
                        succeeded[0].compiler,
                        succeeded[i].compiler,
                    );
                }
            }
        }
        process.exit(1);
    }
}

async function runBulk(showDiffs: boolean, quiet: boolean, failFast: boolean): Promise<void> {
    const start = performance.now();

    if (!quiet) {
        console.log('Verifying nanus compiler consistency\n');
    }

    // Check binaries exist
    const missing: string[] = [];
    for (const compiler of COMPILERS) {
        const bin = join(BIN, compiler);
        if (!(await Bun.file(bin).exists())) {
            missing.push(compiler);
        }
    }

    if (missing.length > 0) {
        console.error(`Missing binaries: ${missing.join(', ')}`);
        console.error('Run `bun run build` first.');
        process.exit(1);
    }

    // Find all .fab files
    const files = await findFiles(EXEMPLA);
    if (!quiet) {
        console.log(`Found ${files.length} .fab files in fons/exempla/\n`);
    }

    // Check each file
    let passed = 0;
    let failed = 0;

    for (const fabPath of files) {
        const result = await checkFile(fabPath);

        if (result.consistent) {
            passed++;
            // Skip output for passing files (quiet or default)
        } else {
            failed++;

            if (quiet) {
                // Single-line output
                console.log(`FAIL ${result.file} (${result.error})`);
            } else {
                console.log(`  \x1b[31mFAIL\x1b[0m  ${result.file} (${result.error})`);

                // Show which compilers failed
                const failedCompilers = result.results.filter(r => !r.success);
                for (const f of failedCompilers) {
                    console.log(`      ${f.compiler}: ${f.error}`);
                }

                // Show diffs if requested and we have output mismatch
                if (showDiffs && result.error === 'output mismatch') {
                    const succeeded = result.results.filter(r => r.success);
                    for (let i = 1; i < succeeded.length; i++) {
                        if (succeeded[i].output !== succeeded[0].output) {
                            showDiff(
                                succeeded[0].output!,
                                succeeded[i].output!,
                                succeeded[0].compiler,
                                succeeded[i].compiler,
                            );
                        }
                    }
                }
            }

            if (failFast) {
                if (!quiet) {
                    console.log('\nStopping on first failure (-x)');
                }
                process.exit(1);
            }
        }
    }

    const elapsed = performance.now() - start;
    console.log(`\n${passed} passed, ${failed} failed (${elapsed.toFixed(0)}ms)`);

    if (failed > 0) {
        process.exit(1);
    }
}

async function main() {
    const { file, showDiffs, quiet, failFast } = parseArgs();

    if (file) {
        await runSingleFile(file, showDiffs);
    } else {
        await runBulk(showDiffs, quiet, failFast);
    }
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
