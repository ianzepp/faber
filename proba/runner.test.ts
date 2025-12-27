/**
 * Shared test runner for cross-target codegen tests.
 *
 * Loads YAML test cases and runs them against multiple codegen targets.
 * Each test case specifies input and expected output per target.
 */

import { describe, test, expect } from 'bun:test';
import { parse as parseYaml } from 'yaml';
import { readFileSync, readdirSync } from 'fs';
import { join, basename } from 'path';

import { tokenize } from '../fons/tokenizer';
import { parse } from '../fons/parser';
import { analyze } from '../fons/semantic';
import { generate } from '../fons/codegen';

// Supported targets
const TARGETS = ['ts', 'py', 'cpp', 'rs', 'zig'] as const;
type Target = (typeof TARGETS)[number];

interface TargetExpectation {
    contains?: string[];
    not_contains?: string[];
    exact?: string;
}

interface TestCase {
    name: string;
    input: string;
    ts?: string | string[] | TargetExpectation;
    py?: string | string[] | TargetExpectation;
    zig?: string | string[] | TargetExpectation;
    cpp?: string | string[] | TargetExpectation;
    rs?: string | string[] | TargetExpectation;
    skip?: Target[];
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
 * Load and run all test cases from a YAML file.
 */
function runTestFile(filePath: string): void {
    const content = readFileSync(filePath, 'utf-8');
    const cases: TestCase[] = parseYaml(content);
    const suiteName = basename(filePath, '.yaml');

    describe(suiteName, () => {
        for (const target of TARGETS) {
            describe(target, () => {
                for (const tc of cases) {
                    const expected = tc[target];

                    // Skip if no expectation for this target
                    if (expected === undefined) continue;

                    // Skip if explicitly marked to skip
                    if (tc.skip?.includes(target)) continue;

                    test(tc.name, () => {
                        const output = compile(tc.input.trim(), target);
                        checkOutput(output, expected);
                    });
                }
            });
        }
    });
}

// Load all YAML test files from this directory
const testDir = import.meta.dir;
const yamlFiles = readdirSync(testDir).filter(f => f.endsWith('.yaml'));

for (const file of yamlFiles) {
    runTestFile(join(testDir, file));
}
