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
 *   bun run verify:nanus                       # bulk check all .fab in fons/rivus
 *   bun run verify:nanus path/to/file.fab      # single file (shows all outputs)
 *   bun run verify:nanus path/to/dir           # bulk check all .fab in directory
 *   bun run verify:nanus *.fab                 # multiple files via shell glob
 *   bun run verify:nanus -s                    # grouped error summary
 *   bun run verify:nanus --diff                # show line-by-line diffs on mismatch
 *   bun run verify:nanus -x                    # exit on first failure
 *   bun run verify:nanus -q                    # quiet: single-line output, skip passes
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
const BIN = join(ROOT, 'opus', 'bin');
const DEFAULT_DIR = join(ROOT, 'fons', 'rivus');

const COMPILERS = ['nanus-ts', 'nanus-go', 'nanus-rs', 'nanus-py'] as const;
type Compiler = (typeof COMPILERS)[number];

interface Args {
    paths: string[];
    showDiffs: boolean;
    quiet: boolean;
    failFast: boolean;
    summary: boolean;
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
  bun run verify:nanus                     Bulk check all .fab in fons/rivus
  bun run verify:nanus <file.fab>          Single file (shows all outputs)
  bun run verify:nanus <dir>               Bulk check all .fab in directory
  bun run verify:nanus *.fab               Multiple files (shell glob)

Options:
  --diff          Show line-by-line diffs on mismatch
  -x, --fail-fast Exit on first failure
  -q, --quiet     Single-line output, skip passes
  -s, --summary   Show grouped error summary at end
  -h, --help      Show this help message

Exit codes:
  0  All checked files consistent
  1  At least one mismatch or failure`);
}

function parseArgs(): Args {
    const args = process.argv.slice(2);
    const paths: string[] = [];
    let showDiffs = false;
    let quiet = false;
    let failFast = false;
    let summary = false;

    for (const arg of args) {
        if (arg === '--help' || arg === '-h') {
            printHelp();
            process.exit(0);
        } else if (arg === '--diff') showDiffs = true;
        else if (arg === '--quiet' || arg === '-q') quiet = true;
        else if (arg === '--fail-fast' || arg === '-x') failFast = true;
        else if (arg === '--summary' || arg === '-s') summary = true;
        else if (!arg.startsWith('-')) paths.push(arg);
    }

    return { paths, showDiffs, quiet, failFast, summary };
}

async function findFiles(dir: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        if (entry.startsWith('.')) continue; // skip hidden files/dirs
        const fullPath = join(dir, entry);
        const s = await stat(fullPath).catch(() => null);
        if (!s) continue; // skip broken symlinks
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

function extractErrorPattern(error: string): string {
    const match = error.match(/error: (.+?)$/m);
    return match ? match[1] : error;
}

function printSummary(results: FileResult[]): void {
    const failures = results.filter(r => !r.consistent);
    if (failures.length === 0) return;

    console.log('\n\x1b[1mFailure Summary\x1b[0m\n');

    // Group by high-level failure type
    const byType = new Map<string, FileResult[]>();
    for (const f of failures) {
        const type = f.error ?? 'unknown';
        if (!byType.has(type)) byType.set(type, []);
        byType.get(type)!.push(f);
    }

    // Print high-level breakdown
    console.log('By failure type:');
    for (const [type, files] of [...byType.entries()].sort((a, b) => b[1].length - a[1].length)) {
        console.log(`  ${files.length.toString().padStart(3)}  ${type}`);
    }

    // For "all compilers failed", break down by error pattern
    const allFailed = byType.get('all compilers failed');
    if (allFailed && allFailed.length > 0) {
        console.log('\nParse errors (all compilers failed):');

        const byPattern = new Map<string, string[]>();
        for (const f of allFailed) {
            const tsResult = f.results.find(r => r.compiler === 'nanus-ts');
            const pattern = tsResult?.error ? extractErrorPattern(tsResult.error) : 'unknown';
            if (!byPattern.has(pattern)) byPattern.set(pattern, []);
            byPattern.get(pattern)!.push(f.file);
        }

        for (const [pattern, files] of [...byPattern.entries()].sort((a, b) => b[1].length - a[1].length)) {
            console.log(`  ${files.length.toString().padStart(3)}  ${pattern}`);
        }
    }

    // For output mismatches, list files
    const mismatches = byType.get('output mismatch');
    if (mismatches && mismatches.length > 0) {
        console.log('\nOutput mismatches (emitter drift):');
        for (const f of mismatches.slice(0, 10)) {
            console.log(`        ${f.file}`);
        }
        if (mismatches.length > 10) {
            console.log(`        ... and ${mismatches.length - 10} more`);
        }
    }

    // Partial failures (one compiler disagrees)
    const partial = failures.filter(f => f.error !== 'all compilers failed' && f.error !== 'output mismatch');
    if (partial.length > 0) {
        console.log('\nPartial failures (parser inconsistency):');
        for (const f of partial) {
            console.log(`        ${f.file} (${f.error})`);
        }
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

async function checkFile(fabPath: string, baseDir: string): Promise<FileResult> {
    const file = relative(baseDir, fabPath);
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
    const fileResult = await checkFile(fullPath, process.cwd());
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

interface BulkOptions {
    files?: string[];
    dir?: string;
    showDiffs: boolean;
    quiet: boolean;
    failFast: boolean;
    summary: boolean;
}

async function runBulk(opts: BulkOptions): Promise<void> {
    const { showDiffs, quiet, failFast, summary } = opts;
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

    // Determine files to check and base directory for display
    let files: string[];
    let baseDir: string;

    if (opts.files && opts.files.length > 0) {
        files = opts.files.map(f => isAbsolute(f) ? f : join(process.cwd(), f));
        baseDir = process.cwd();
        if (!quiet) {
            console.log(`Checking ${files.length} file(s)\n`);
        }
    } else {
        const dir = opts.dir ?? DEFAULT_DIR;
        baseDir = dir;
        files = await findFiles(dir);
        if (!quiet) {
            const displayDir = relative(process.cwd(), dir) || '.';
            console.log(`Found ${files.length} .fab files in ${displayDir}/\n`);
        }
    }

    if (files.length === 0) {
        console.log('No .fab files found');
        return;
    }

    // Check each file
    const allResults: FileResult[] = [];
    let passed = 0;
    let failed = 0;

    for (const fabPath of files) {
        const result = await checkFile(fabPath, baseDir);
        allResults.push(result);

        if (result.consistent) {
            passed++;
        } else {
            failed++;

            if (!summary) {
                if (quiet) {
                    console.log(`FAIL ${result.file} (${result.error})`);
                } else {
                    console.log(`  \x1b[31mFAIL\x1b[0m  ${result.file} (${result.error})`);

                    const failedCompilers = result.results.filter(r => !r.success);
                    for (const f of failedCompilers) {
                        console.log(`      ${f.compiler}: ${f.error}`);
                    }

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

    if (summary) {
        printSummary(allResults);
    }

    if (failed > 0) {
        process.exit(1);
    }
}

async function main() {
    const { paths, showDiffs, quiet, failFast, summary } = parseArgs();

    if (paths.length === 0) {
        // No args: search CWD
        await runBulk({ showDiffs, quiet, failFast, summary });
    } else if (paths.length === 1) {
        // Single arg: could be file or directory
        const p = isAbsolute(paths[0]) ? paths[0] : join(process.cwd(), paths[0]);
        const s = await stat(p).catch(() => null);

        if (!s) {
            console.error(`Not found: ${paths[0]}`);
            process.exit(1);
        }

        if (s.isDirectory()) {
            await runBulk({ dir: p, showDiffs, quiet, failFast, summary });
        } else {
            await runSingleFile(paths[0], showDiffs);
        }
    } else {
        // Multiple args: treat as file list
        await runBulk({ files: paths, showDiffs, quiet, failFast, summary });
    }
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
