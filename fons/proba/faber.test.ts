/**
 * Bun test runner for cross-target codegen tests (faber compiler).
 *
 * Loads YAML test cases and runs them against multiple codegen targets.
 * Each test case specifies input and expected output per target.
 *
 * USAGE
 *   bun test proba/faber.test.ts              Run all tests + coverage report
 *   bun test proba/faber.test.ts -t "binary"  Run tests matching "binary"
 *   bun test proba/faber.test.ts -t "@ts"     Run only TypeScript target tests
 *   bun test proba/faber.test.ts -t "@py"     Run only Python target tests
 *   bun test proba/faber.test.ts -t "@zig"    Run only Zig target tests
 *   bun test proba/faber.test.ts -t "@rs"     Run only Rust target tests
 *   bun test proba/faber.test.ts -t "@cpp"    Run only C++ target tests
 *
 * ENVIRONMENT VARIABLES
 *   STRICT_COVERAGE=1           Fail if ANY test is missing target expectations
 *   STRICT_COVERAGE=<pattern>   Fail only for tests matching regex pattern
 *   COVERAGE_DETAILS=1          Show per-suite breakdown in coverage report
 *
 * EXAMPLES
 *   STRICT_COVERAGE=1 bun test proba/faber.test.ts
 *     Fail on any test missing ts/py/cpp/rs/zig expectations
 *
 *   STRICT_COVERAGE=operator bun test proba/faber.test.ts
 *     Fail only for tests with "operator" in suite or test name
 *
 *   STRICT_COVERAGE="binary|unary" bun test proba/faber.test.ts
 *     Fail for tests matching "binary" or "unary"
 *
 * YAML TEST FORMAT
 *   - name: test name
 *     variant: optional-variant  # Creates feature "filename: optional-variant"
 *     source: |
 *       fixum x = 1 + 2
 *     wrap: 'cura arena fit alloc { $ }'  # optional: wrap input ($ = placeholder)
 *     expect:
 *       ts: "const x = (1 + 2);"
 *       py: "x = (1 + 2)"
 *       rs:
 *         - "let x"
 *         - "(1 + 2)"
 *       cpp:
 *         contains: ["const auto x"]
 *         not_contains: ["var"]
 *       zig:
 *         exact: "const x = (1 + 2);"
 *     skip: [cpp]  # optional: skip specific targets
 *     faber: false # optional: skip this test for faber compiler
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
 *
 * COVERAGE REPORT
 *   Printed after all tests showing:
 *   - Missing expectations by target (ts: N tests, py: N tests, ...)
 *   - Details by suite with specific test names (when COVERAGE_DETAILS=1)
 */

import { describe, test, expect, afterAll } from 'bun:test';

import {
    EXECUTABLE_TARGETS,
    type ExecutableTarget,
    type Target,
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
    matchOutput,
    matchErrata,
} from './shared';

// Coverage tracking
interface CoverageGap {
    suite: string;
    test: string;
    missingTargets: ExecutableTarget[];
}

const coverageGaps: CoverageGap[] = [];

// Track which tests actually ran (to filter coverage report)
const testsRun = new Set<string>();

function testKey(suite: string, testName: string): string {
    return `${suite}::${testName}`;
}

// Strict mode: fail on missing targets
// STRICT_COVERAGE=1 means all tests, STRICT_COVERAGE=pattern means regex match
const strictCoverage = process.env.STRICT_COVERAGE;
const strictPattern = strictCoverage && strictCoverage !== '1' ? new RegExp(strictCoverage, 'i') : null;

/**
 * Check if output matches expectation (bun:test version - throws on failure).
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
 * Check if error matches errata expectation (bun:test version - throws on failure).
 */
function checkErrata(error: unknown, expected: ErrataExpectation): void {
    const message = error instanceof Error ? error.message : String(error);

    if (expected === true) {
        return;
    }
    else if (typeof expected === 'string') {
        expect(message).toBe(expected);
    }
    else {
        for (const fragment of expected) {
            expect(message).toContain(fragment);
        }
    }
}

/**
 * Check if a test should enforce strict coverage (all targets required).
 */
function shouldEnforceStrict(suiteName: string, testName: string): boolean {
    if (!strictCoverage) return false;
    if (strictCoverage === '1') return true;

    const fullPath = `${suiteName}/${testName}`;
    return strictPattern?.test(fullPath) ?? false;
}

/**
 * Run all test cases from a loaded test file.
 */
function runTestFile(suiteName: string, cases: TestCase[]): void {
    describe(suiteName, () => {
        // Errata tests: expect compilation to fail
        describe('errata', () => {
            for (const tc of cases) {
                if (!hasErrata(tc)) continue;
                if (shouldSkipFaber(tc)) continue;

                test(tc.name, () => {
                    testsRun.add(testKey(suiteName, tc.name));
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
            }
        });

        // Per-target output tests
        // WHY: Use @target prefix for easy filtering: `bun test -t "@zig"`
        for (const target of EXECUTABLE_TARGETS) {
            describe(`@${target}`, () => {
                for (const tc of cases) {
                    if (hasErrata(tc)) continue;
                    if (shouldSkipFaber(tc)) continue;

                    const expected = getExpectation(tc, target);
                    const isSkipped = tc.skip?.includes(target);

                    // Track missing coverage (not skipped, just missing)
                    if (expected === undefined && !isSkipped) {
                        let gap = coverageGaps.find(g => g.suite === suiteName && g.test === tc.name);
                        if (!gap) {
                            gap = { suite: suiteName, test: tc.name, missingTargets: [] };
                            coverageGaps.push(gap);
                        }
                        gap.missingTargets.push(target);
                    }

                    if (expected === undefined) continue;
                    if (isSkipped) continue;

                    test(tc.name, () => {
                        testsRun.add(testKey(suiteName, tc.name));
                        const input = getSource(tc);
                        const result = compile(input.trim(), target);

                        if (!result.success) {
                            throw new Error(result.error);
                        }

                        checkOutput(result.output!, expected);
                    });
                }
            });
        }

        // In strict mode, add a test that fails if any targets are missing
        for (const tc of cases) {
            if (hasErrata(tc)) continue;

            if (shouldEnforceStrict(suiteName, tc.name)) {
                const missingTargets = EXECUTABLE_TARGETS.filter(t => {
                    const exp = getExpectation(tc, t);
                    const skipped = tc.skip?.includes(t);
                    return exp === undefined && !skipped;
                });

                if (missingTargets.length > 0) {
                    test(`${tc.name} [coverage]`, () => {
                        throw new Error(`Missing expectations for targets: ${missingTargets.join(', ')}`);
                    });
                }
            }
        }
    });
}

// Load all YAML test files from this directory and subdirectories
const testDir = import.meta.dir;
const testFiles = findYamlFiles(testDir, testDir);

for (const { suiteName, cases } of testFiles) {
    runTestFile(suiteName, cases);
}

// Print coverage report after all tests
afterAll(() => {
    const relevantGaps = coverageGaps.filter(gap => testsRun.has(testKey(gap.suite, gap.test)));

    if (relevantGaps.length === 0) {
        console.log('\n✓ Full target coverage: all tests have expectations for all targets\n');
        return;
    }

    console.log('\n' + '='.repeat(70));
    console.log('COVERAGE REPORT: Tests missing target expectations');
    console.log('='.repeat(70));

    const bySuite = new Map<string, CoverageGap[]>();
    for (const gap of relevantGaps) {
        const list = bySuite.get(gap.suite) ?? [];
        list.push(gap);
        bySuite.set(gap.suite, list);
    }

    const targetCounts: Record<ExecutableTarget, number> = { ts: 0, py: 0, cpp: 0, rs: 0, zig: 0 };
    for (const gap of relevantGaps) {
        for (const t of gap.missingTargets) {
            targetCounts[t]++;
        }
    }

    console.log('\nMissing by target:');
    for (const t of EXECUTABLE_TARGETS) {
        if (targetCounts[t] > 0) {
            console.log(`  ${t}: ${targetCounts[t]} tests`);
        }
    }

    if (process.env.COVERAGE_DETAILS) {
        console.log('\nDetails by suite:');
        for (const [suite, gaps] of bySuite) {
            console.log(`\n  ${suite}:`);
            for (const gap of gaps) {
                console.log(`    - ${gap.test}: missing ${gap.missingTargets.join(', ')}`);
            }
        }
    }

    console.log('\n' + '='.repeat(70));
    console.log(`Total: ${relevantGaps.length} tests with incomplete coverage`);
    console.log('='.repeat(70) + '\n');
});
