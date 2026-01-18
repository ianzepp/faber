#!/usr/bin/env bun
/**
 * Test nanus compiler against golden corpus.
 *
 * For each .fab file in fons/nanus/corpus/:
 *   1. Compile with nanus
 *   2. Compare output to .ts.golden reference
 *   3. Report pass/fail
 *
 * Options:
 *   --diff    Show line-by-line diff for failures
 */

import { readdir } from 'fs/promises';
import { join, basename } from 'path';
import { compile } from '../fons/nanus';

const ROOT = join(import.meta.dir, '..');
const CORPUS = join(ROOT, 'fons', 'corpus');

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

async function main() {
    const args = process.argv.slice(2);
    const showDiffs = args.includes('--diff');

    const files = await readdir(CORPUS);
    const fabFiles = files.filter(f => f.endsWith('.fab')).sort();

    const results: TestResult[] = [];
    let passed = 0;
    let failed = 0;

    for (const fabFile of fabFiles) {
        const name = basename(fabFile, '.fab');
        const fabPath = join(CORPUS, fabFile);
        const goldenPath = join(CORPUS, `${name}.ts.golden`);

        const source = await Bun.file(fabPath).text();
        const goldenFile = Bun.file(goldenPath);

        if (!(await goldenFile.exists())) {
            results.push({ name, passed: false, error: 'missing golden file' });
            failed++;
            continue;
        }

        const expected = await goldenFile.text();
        const result = compile(source, { filename: fabFile });

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
    console.log('nanus golden corpus tests\n');

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
