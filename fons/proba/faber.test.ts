/**
 * Bun test runner for faber compiler (TypeScript target only).
 *
 * Loads YAML test cases and runs them against faber's TS codegen.
 * For other targets, see Rivus.
 *
 * USAGE
 *   bun test fons/proba/faber.test.ts              Run all tests
 *   bun test fons/proba/faber.test.ts -t "binary"  Run tests matching "binary"
 *
 * YAML TEST FORMAT
 *   - name: test name
 *     source: |
 *       fixum x = 1 + 2
 *     wrap: 'cura arena fit alloc { $ }'  # optional: wrap input ($ = placeholder)
 *     expect:
 *       ts: "const x = (1 + 2);"
 *     faber: false  # optional: skip this test for faber compiler
 *
 * ERROR TEST FORMAT (errata)
 *   - name: error test
 *     source: 'invalid code'
 *     errata: true                    # any error
 *     errata: 'exact error message'   # exact match
 *     errata: ['fragment1', 'frag2']  # all fragments must be present
 *
 * EXPECTATION FORMATS
 *   string          Exact match (after trimming)
 *   string[]        All fragments must be present (contains)
 *   { exact }       Exact match
 *   { contains }    All fragments must be present
 *   { not_contains} Fragments must NOT be present
 */

import { describe, test, expect } from 'bun:test';

import {
    type TestCase,
    type TargetExpectation,
    type ErrataExpectation,
    findYamlFiles,
    getSource,
    getExpectation,
    shouldSkipFaber,
    hasErrata,
    compile,
    compileStrict,
    matchErrata,
} from './shared';

/**
 * Check if output matches expectation (throws on failure).
 */
function checkOutput(output: string, expected: string | string[] | TargetExpectation): void {
    if (typeof expected === 'string') {
        expect(output.trim()).toBe(expected);
    }
    else if (Array.isArray(expected)) {
        for (const fragment of expected) {
            expect(output).toContain(fragment);
        }
    }
    else {
        if (expected.exact !== undefined) {
            expect(output.trim()).toBe(expected.exact);
        }
        if (expected.contains) {
            for (const fragment of expected.contains) {
                expect(output).toContain(fragment);
            }
        }
        if (expected.not_contains) {
            for (const fragment of expected.not_contains) {
                expect(output).not.toContain(fragment);
            }
        }
    }
}

/**
 * Run all test cases from a loaded test file.
 */
function runTestFile(suiteName: string, cases: TestCase[]): void {
    describe(suiteName, () => {
        for (const tc of cases) {
            if (shouldSkipFaber(tc)) continue;

            // Errata tests: expect compilation to fail
            if (hasErrata(tc)) {
                test(tc.name, () => {
                    const input = getSource(tc);
                    const result = compileStrict(input.trim());

                    if (result.success) {
                        throw new Error('Expected compilation to fail, but it succeeded');
                    }

                    const match = matchErrata(result.error!, tc.errata);
                    if (!match.passed) {
                        throw new Error(match.errors.join('\n'));
                    }
                });
                continue;
            }

            // Normal tests: compile and check output
            const expected = getExpectation(tc);
            if (expected === undefined) continue;
            if (tc.skip?.includes('ts')) continue;

            test(tc.name, () => {
                const input = getSource(tc);
                const result = compile(input.trim());

                if (!result.success) {
                    throw new Error(result.error);
                }

                checkOutput(result.output!, expected);
            });
        }
    });
}

// Load all YAML test files
const testDir = import.meta.dir;
const testFiles = findYamlFiles(testDir, testDir);

for (const { suiteName, cases } of testFiles) {
    runTestFile(suiteName, cases);
}
