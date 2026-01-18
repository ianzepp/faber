#!/usr/bin/env bun
/**
 * Test compiler against golden corpus.
 *
 * For each .fab file in fons/corpus/:
 *   1. Compile with specified compiler
 *   2. Compare output to .ts.golden reference
 *   3. Report pass/fail
 *
 * Options:
 *   -c, --compiler <name>  Compiler to use: nanus, faber, rivus (default: nanus)
 *   --diff                 Show line-by-line diff for failures
 */

import { readdir } from 'fs/promises';
import { join, basename } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const CORPUS = join(ROOT, 'fons', 'corpus');
const BIN = join(ROOT, 'opus', 'bin');

interface TestResult {
    name: string;
    passed: boolean;
    error?: string;
    actual?: string;
    expected?: string;
}

function showDiff(actual: string, expected: string) {
    const a = actual.split('\n');
    const e = expected.split('\n');
    const maxLines = Math.max(a.length, e.length);

    for (let i = 0; i < maxLines; i++) {
        if (a[i] !== e[i]) {
            console.log(`    L${i + 1}:`);
            console.log(`      \x1b[31m- ${JSON.stringify(e[i] ?? '(missing)')}\x1b[0m`);
            console.log(`      \x1b[32m+ ${JSON.stringify(a[i] ?? '(missing)')}\x1b[0m`);
        }
    }
}

function parseArgs(args: string[]): { compiler: string; showDiffs: boolean } {
    let compiler = 'nanus';
    let showDiffs = false;

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === '-c' || arg === '--compiler') {
            compiler = args[++i];
        } else if (arg === '--diff') {
            showDiffs = true;
        }
    }

    return { compiler, showDiffs };
}

async function compile(compiler: string, fabPath: string): Promise<{ success: boolean; output?: string; error?: string }> {
    const bin = join(BIN, compiler);
    try {
        const result = await $`${bin} compile ${fabPath}`.quiet();
        return { success: true, output: result.text() };
    } catch (err: any) {
        return { success: false, error: err.stderr?.toString() || err.message };
    }
}

async function main() {
    const { compiler, showDiffs } = parseArgs(process.argv.slice(2));

    const validCompilers = ['nanus', 'faber', 'rivus'];
    if (!validCompilers.includes(compiler)) {
        console.error(`Invalid compiler: ${compiler}. Must be one of: ${validCompilers.join(', ')}`);
        process.exit(1);
    }

    const files = await readdir(CORPUS);
    const fabFiles = files.filter(f => f.endsWith('.fab')).sort();

    const results: TestResult[] = [];
    let passed = 0;
    let failed = 0;

    for (const fabFile of fabFiles) {
        const name = basename(fabFile, '.fab');
        const fabPath = join(CORPUS, fabFile);
        const goldenPath = join(CORPUS, `${name}.ts.golden`);

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

    // Report results
    console.log(`${compiler} golden corpus tests\n`);

    for (const r of results) {
        const status = r.passed ? '\x1b[32mPASS\x1b[0m' : '\x1b[31mFAIL\x1b[0m';
        const detail = r.error && r.error !== 'output mismatch' ? ` (${r.error})` : '';
        console.log(`  ${status}  ${r.name}${detail}`);

        if (showDiffs && !r.passed && r.actual && r.expected) {
            showDiff(r.actual, r.expected);
        }
    }

    console.log(`\n${passed} passed, ${failed} failed`);

    if (failed > 0) {
        process.exit(1);
    }
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
