/**
 * Shared test runner for cross-target codegen tests.
 *
 * Loads YAML test cases and runs them against multiple codegen targets.
 * Each test case specifies input and expected output per target.
 *
 * USAGE
 *   bun test proba/runner.test.ts              Run all tests + coverage report
 *   bun test proba/runner.test.ts -t "binary"  Run tests matching "binary"
 *
 * ENVIRONMENT VARIABLES
 *   STRICT_COVERAGE=1           Fail if ANY test is missing target expectations
 *   STRICT_COVERAGE=<pattern>   Fail only for tests matching regex pattern
 *
 * EXAMPLES
 *   STRICT_COVERAGE=1 bun test proba/runner.test.ts
 *     Fail on any test missing ts/py/cpp/rs/zig expectations
 *
 *   STRICT_COVERAGE=operator bun test proba/runner.test.ts
 *     Fail only for tests with "operator" in suite or test name
 *
 *   STRICT_COVERAGE="binary|unary" bun test proba/runner.test.ts
 *     Fail for tests matching "binary" or "unary"
 *
 * YAML TEST FORMAT
 *   - name: test name
 *     faber: |
 *       fixum x = 1 + 2
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
 *   - Details by suite with specific test names
 */

import { describe, test, expect, afterAll } from 'bun:test';
import { parse as parseYaml } from 'yaml';
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, basename, relative } from 'path';

import { tokenize } from '../fons/tokenizer';
import { parse } from '../fons/parser';
import { analyze } from '../fons/semantic';
import { generate } from '../fons/codegen';

// Supported targets
const TARGETS = ['ts', 'py', 'cpp', 'rs', 'zig'] as const;
type Target = (typeof TARGETS)[number];

// Coverage tracking
interface CoverageGap {
    suite: string;
    test: string;
    missingTargets: Target[];
}

const coverageGaps: CoverageGap[] = [];

// Strict mode: fail on missing targets
// STRICT_COVERAGE=1 means all tests, STRICT_COVERAGE=pattern means regex match
const strictCoverage = process.env.STRICT_COVERAGE;
const strictPattern = strictCoverage && strictCoverage !== '1' ? new RegExp(strictCoverage, 'i') : null;

interface TargetExpectation {
    contains?: string[];
    not_contains?: string[];
    exact?: string;
}

// Legacy format: input + top-level target keys
interface LegacyTestCase {
    name: string;
    input: string;
    ts?: string | string[] | TargetExpectation;
    py?: string | string[] | TargetExpectation;
    zig?: string | string[] | TargetExpectation;
    cpp?: string | string[] | TargetExpectation;
    rs?: string | string[] | TargetExpectation;
    skip?: Target[];
}

// New format: faber + expect object
interface ModernTestCase {
    name: string;
    faber: string;
    expect: {
        ts?: string | string[] | TargetExpectation;
        py?: string | string[] | TargetExpectation;
        zig?: string | string[] | TargetExpectation;
        cpp?: string | string[] | TargetExpectation;
        rs?: string | string[] | TargetExpectation;
    };
    skip?: Target[];
}

type TestCase = LegacyTestCase | ModernTestCase;

function isModernTestCase(tc: TestCase): tc is ModernTestCase {
    return 'faber' in tc && 'expect' in tc;
}

function getInput(tc: TestCase): string {
    return isModernTestCase(tc) ? tc.faber : tc.input;
}

function getExpectation(tc: TestCase, target: Target): string | string[] | TargetExpectation | undefined {
    return isModernTestCase(tc) ? tc.expect[target] : tc[target];
}

/**
 * Compile Faber source to target language.
 */
function compile(code: string, target: Target = 'ts'): string {
    const { tokens } = tokenize(code);
    const { program } = parse(tokens);

    if (!program) {
        throw new Error('Parse failed');
    }

    const { program: analyzedProgram } = analyze(program);
    return generate(analyzedProgram, { target });
}

/**
 * Check if output matches expectation.
 * - String: exact match (after trimming)
 * - Array: all fragments must be present (contains)
 * - Object: { contains?: [], not_contains?: [], exact?: string }
 */
function checkOutput(output: string, expected: string | string[] | TargetExpectation): void {
    if (typeof expected === 'string') {
        expect(output.trim()).toBe(expected);
    } else if (Array.isArray(expected)) {
        for (const fragment of expected) {
            expect(output).toContain(fragment);
        }
    } else {
        // Object form with contains/not_contains/exact
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
 * Check if a test should enforce strict coverage (all targets required).
 */
function shouldEnforceStrict(suiteName: string, testName: string): boolean {
    if (!strictCoverage) return false;
    if (strictCoverage === '1') return true;

    // Match against suite/test path
    const fullPath = `${suiteName}/${testName}`;
    return strictPattern?.test(fullPath) ?? false;
}

/**
 * Load and run all test cases from a YAML file.
 */
function runTestFile(filePath: string, suiteName: string): void {
    const content = readFileSync(filePath, 'utf-8');
    const cases: TestCase[] = parseYaml(content);

    describe(suiteName, () => {
        for (const target of TARGETS) {
            describe(target, () => {
                for (const tc of cases) {
                    const expected = getExpectation(tc, target);
                    const isSkipped = tc.skip?.includes(target);

                    // Track missing coverage (not skipped, just missing)
                    if (expected === undefined && !isSkipped) {
                        // Find or create gap entry for this test
                        let gap = coverageGaps.find(g => g.suite === suiteName && g.test === tc.name);
                        if (!gap) {
                            gap = { suite: suiteName, test: tc.name, missingTargets: [] };
                            coverageGaps.push(gap);
                        }
                        gap.missingTargets.push(target);
                    }

                    // Skip if no expectation for this target
                    if (expected === undefined) continue;

                    // Skip if explicitly marked to skip
                    if (isSkipped) continue;

                    test(tc.name, () => {
                        const input = getInput(tc);
                        const output = compile(input.trim(), target);
                        checkOutput(output, expected);
                    });
                }
            });
        }

        // In strict mode, add a test that fails if any targets are missing
        for (const tc of cases) {
            if (shouldEnforceStrict(suiteName, tc.name)) {
                const missingTargets = TARGETS.filter(t => {
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

/**
 * Recursively find all YAML files in a directory.
 */
function findYamlFiles(dir: string, baseDir: string): Array<{ path: string; name: string }> {
    const results: Array<{ path: string; name: string }> = [];

    for (const entry of readdirSync(dir)) {
        const fullPath = join(dir, entry);
        const stat = statSync(fullPath);

        if (stat.isDirectory()) {
            results.push(...findYamlFiles(fullPath, baseDir));
        } else if (entry.endsWith('.yaml')) {
            // Build suite name from relative path: codegen/expressions/identifier
            const relPath = relative(baseDir, fullPath);
            const suiteName = relPath.replace(/\.yaml$/, '').replace(/\//g, '/');
            results.push({ path: fullPath, name: suiteName });
        }
    }

    return results;
}

// Load all YAML test files from this directory and subdirectories
const testDir = import.meta.dir;
const yamlFiles = findYamlFiles(testDir, testDir);

for (const { path, name } of yamlFiles) {
    runTestFile(path, name);
}

// Print coverage report after all tests
afterAll(() => {
    if (coverageGaps.length === 0) {
        console.log('\n✓ Full target coverage: all tests have expectations for all targets\n');
        return;
    }

    console.log('\n' + '='.repeat(70));
    console.log('COVERAGE REPORT: Tests missing target expectations');
    console.log('='.repeat(70));

    // Group by suite
    const bySuite = new Map<string, CoverageGap[]>();
    for (const gap of coverageGaps) {
        const list = bySuite.get(gap.suite) ?? [];
        list.push(gap);
        bySuite.set(gap.suite, list);
    }

    // Summary by target
    const targetCounts: Record<Target, number> = { ts: 0, py: 0, cpp: 0, rs: 0, zig: 0 };
    for (const gap of coverageGaps) {
        for (const t of gap.missingTargets) {
            targetCounts[t]++;
        }
    }

    console.log('\nMissing by target:');
    for (const t of TARGETS) {
        if (targetCounts[t] > 0) {
            console.log(`  ${t}: ${targetCounts[t]} tests`);
        }
    }

    console.log('\nDetails by suite:');
    for (const [suite, gaps] of bySuite) {
        console.log(`\n  ${suite}:`);
        for (const gap of gaps) {
            console.log(`    - ${gap.test}: missing ${gap.missingTargets.join(', ')}`);
        }
    }

    console.log('\n' + '='.repeat(70));
    console.log(`Total: ${coverageGaps.length} tests with incomplete coverage`);
    console.log('='.repeat(70) + '\n');
});
