#!/usr/bin/env bun
/**
 * Run golden tests against multiple compilers.
 *
 * For each .fab file in fons/golden/:
 *   1. Compile with each specified compiler
 *   2. Compare output to .ts.golden reference
 *   3. Report pass/fail per compiler
 *
 * Usage:
 *   bun run build:golden                    # all compilers (default)
 *   bun run build:golden -c nanus-ts        # single compiler
 *   bun run build:golden -c nanus-ts,faber  # specific compilers
 *   bun run build:golden --diff             # show diffs on failure
 */

import { readdir } from 'fs/promises';
import { basename, join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const GOLDEN = join(ROOT, 'fons', 'golden');
const BIN = join(ROOT, 'opus', 'bin');

type Compiler = 'nanus-ts' | 'nanus-go' | 'nanus-rs' | 'nanus-py' | 'faber' | 'rivus';

const ALL_COMPILERS: Compiler[] = ['nanus-ts', 'nanus-go', 'nanus-rs', 'nanus-py', 'faber'];

interface Args {
    compilers: Compiler[];
    showDiffs: boolean;
}

interface TestResult {
    name: string;
    passed: boolean;
    error?: string;
    actual?: string;
    expected?: string;
}

interface CompilerResult {
    compiler: Compiler;
    passed: number;
    failed: number;
    results: TestResult[];
}

function parseArgs(): Args {
    const args = process.argv.slice(2);
    let compilers: Compiler[] = ALL_COMPILERS;
    let showDiffs = false;

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];

        if (arg === '-c' || arg === '--compiler') {
            const val = args[++i];
            compilers = val.split(',').map(c => c.trim()) as Compiler[];

            for (const c of compilers) {
                if (!ALL_COMPILERS.includes(c) && c !== 'rivus') {
                    console.error(`Invalid compiler: ${c}. Must be one of: ${ALL_COMPILERS.join(', ')}, rivus`);
                    process.exit(1);
                }
            }
        } else if (arg === '--diff') {
            showDiffs = true;
        }
    }

    return { compilers, showDiffs };
}

function showDiff(actual: string, expected: string) {
    const a = actual.split('\n');
    const e = expected.split('\n');
    const maxLines = Math.max(a.length, e.length);

    for (let i = 0; i < maxLines; i++) {
        if (a[i] !== e[i]) {
            console.log(`      L${i + 1}:`);
            console.log(`        \x1b[31m- ${JSON.stringify(e[i] ?? '(missing)')}\x1b[0m`);
            console.log(`        \x1b[32m+ ${JSON.stringify(a[i] ?? '(missing)')}\x1b[0m`);
        }
    }
}

async function compile(
    compiler: Compiler,
    fabPath: string,
): Promise<{ success: boolean; output?: string; error?: string }> {
    const bin = join(BIN, compiler);

    // Check if binary exists
    if (!(await Bun.file(bin).exists())) {
        return { success: false, error: `binary not found: ${bin}` };
    }

    try {
        // nanus-ts, nanus-go, nanus-rs, nanus-py use stdin/stdout
        if (compiler === 'nanus-ts' || compiler === 'nanus-go' || compiler === 'nanus-rs' || compiler === 'nanus-py') {
            const result = await $`cat ${fabPath} | ${bin} emit -t ts`.quiet();
            return { success: true, output: result.text() };
        }

        // faber, rivus use file args
        const result = await $`${bin} compile ${fabPath}`.quiet();
        return { success: true, output: result.text() };
    } catch (err: any) {
        return { success: false, error: err.stderr?.toString().trim() || err.message };
    }
}

async function runGoldenTests(compiler: Compiler, showDiffs: boolean): Promise<CompilerResult> {
    const files = await readdir(GOLDEN);
    const fabFiles = files.filter(f => f.endsWith('.fab')).sort();

    const results: TestResult[] = [];
    let passed = 0;
    let failed = 0;

    for (const fabFile of fabFiles) {
        const name = basename(fabFile, '.fab');
        const fabPath = join(GOLDEN, fabFile);
        const goldenPath = join(GOLDEN, `${name}.ts.golden`);

        const goldenFile = Bun.file(goldenPath);

        if (!(await goldenFile.exists())) {
            results.push({ name, passed: false, error: 'missing golden file' });
            failed++;
            continue;
        }

        const expected = await goldenFile.text();
        const result = await compile(compiler, fabPath);

        if (!result.success) {
            results.push({ name, passed: false, error: result.error });
            failed++;
            continue;
        }

        const actual = result.output!.trim();
        const expectedTrimmed = expected.trim();

        if (actual === expectedTrimmed) {
            results.push({ name, passed: true });
            passed++;
        } else {
            results.push({ name, passed: false, error: 'output mismatch', actual, expected: expectedTrimmed });
            failed++;
        }
    }

    return { compiler, passed, failed, results };
}

async function main() {
    const { compilers, showDiffs } = parseArgs();
    const start = performance.now();

    console.log(`Golden tests (compilers: ${compilers.join(', ')})\n`);

    const allResults: CompilerResult[] = [];
    let totalPassed = 0;
    let totalFailed = 0;

    for (const compiler of compilers) {
        const result = await runGoldenTests(compiler, showDiffs);
        allResults.push(result);
        totalPassed += result.passed;
        totalFailed += result.failed;

        // Print results for this compiler
        const status = result.failed === 0 ? '\x1b[32mOK\x1b[0m' : '\x1b[31mFAIL\x1b[0m';
        console.log(`[${compiler}] ${status} (${result.passed}/${result.passed + result.failed})`);

        // Show failures
        const failures = result.results.filter(r => !r.passed);
        if (failures.length > 0) {
            for (const f of failures) {
                const detail = f.error && f.error !== 'output mismatch' ? ` (${f.error})` : '';
                console.log(`    \x1b[31mFAIL\x1b[0m  ${f.name}${detail}`);

                if (showDiffs && f.actual && f.expected) {
                    showDiff(f.actual, f.expected);
                }
            }
        }
    }

    const elapsed = performance.now() - start;
    console.log(`\nTotal: ${totalPassed} passed, ${totalFailed} failed (${elapsed.toFixed(0)}ms)`);

    if (totalFailed > 0) {
        process.exit(1);
    }
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
