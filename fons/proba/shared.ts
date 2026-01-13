/**
 * Shared utilities for cross-target codegen tests.
 *
 * Used by both the bun:test runner (faber.test.ts) and the SQLite harness.
 */

import { parse as parseYaml } from 'yaml';
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, basename, relative } from 'path';

import { tokenize } from '../faber/tokenizer';
import { parse } from '../faber/parser';
import { analyze } from '../faber/semantic';
import { generate } from '../faber/codegen';

// Faber targets TypeScript only (see consilia/compiler-roles.md)
export type Target = 'ts';

export interface TargetExpectation {
    contains?: string[];
    not_contains?: string[];
    exact?: string;
}

// Error expectation: true (any error), string (exact), or string[] (contains all)
export type ErrataExpectation = true | string | string[];

export interface TestCase {
    name: string;
    source: string;
    wrap?: string;
    variant?: string; // Optional sub-feature: "si" + variant "cape-secus" → "si: cape-secus"
    expect?: {
        ts?: string | string[] | TargetExpectation;
    };
    // Legacy top-level expectation (deprecated)
    ts?: string | string[] | TargetExpectation;
    skip?: Target[];
    errata?: ErrataExpectation;
    faber?: boolean; // Set to false to skip this test for faber compiler
}

export interface TestFile {
    path: string;
    feature: string; // Derived from filename (kebab-case)
    suiteName: string; // Full path for bun:test compatibility
    cases: TestCase[];
}

export interface CompileResult {
    success: boolean;
    output?: string;
    error?: string;
}

export interface MatchResult {
    passed: boolean;
    errors: string[];
}

/**
 * Get the effective source code, applying wrap template if present.
 */
export function getSource(tc: TestCase): string {
    if (tc.wrap) {
        return tc.wrap.replace('$', tc.source);
    }
    return tc.source;
}

/**
 * Get the TS expectation for a test case.
 */
export function getExpectation(tc: TestCase): string | string[] | TargetExpectation | undefined {
    return tc.expect?.ts ?? tc.ts;
}

/**
 * Check if test should be skipped for faber compiler.
 */
export function shouldSkipFaber(tc: TestCase): boolean {
    return tc.faber === false;
}

/**
 * Check if test is an errata (error expectation) test.
 */
export function hasErrata(tc: TestCase): tc is TestCase & { errata: ErrataExpectation } {
    return 'errata' in tc && tc.errata !== undefined;
}

/**
 * Get the feature name for a test case.
 * Combines file-level feature with optional variant.
 */
export function getFeatureName(fileFeature: string, tc: TestCase): string {
    if (tc.variant) {
        return `${fileFeature}: ${tc.variant}`;
    }
    return fileFeature;
}

/**
 * Extract feature name from filename (kebab-case).
 */
export function featureFromFilename(filename: string): string {
    return basename(filename).replace(/\.yaml$/, '');
}

/**
 * Compile Faber source to TypeScript (lenient mode).
 * Ignores semantic errors for snippet tests with undefined vars.
 */
export function compile(code: string): CompileResult {
    try {
        const { tokens } = tokenize(code);
        const { program } = parse(tokens);

        if (!program) {
            return { success: false, error: 'Parse failed' };
        }

        const { program: analyzedProgram } = analyze(program);
        const output = generate(analyzedProgram);
        return { success: true, output };
    }
    catch (err) {
        return { success: false, error: err instanceof Error ? err.message : String(err) };
    }
}

/**
 * Compile Faber source strictly - returns error on any tokenizer, parse, or semantic error.
 * Used for errata tests that expect compilation to fail.
 */
export function compileStrict(code: string): CompileResult {
    try {
        const { tokens, errors: tokenErrors } = tokenize(code);

        if (tokenErrors.length > 0) {
            const messages = tokenErrors.map(e => `${e.code}: ${e.text}`).join('; ');
            return { success: false, error: `Tokenizer errors: ${messages}` };
        }

        const { program, errors: parseErrors } = parse(tokens);

        if (parseErrors.length > 0) {
            const messages = parseErrors.map(e => `${e.code}: ${e.message}`).join('; ');
            return { success: false, error: `Parse errors: ${messages}` };
        }

        if (!program) {
            return { success: false, error: 'Parse failed: no program' };
        }

        const { program: analyzedProgram, errors: semanticErrors } = analyze(program);

        if (semanticErrors.length > 0) {
            const messages = semanticErrors.map(e => e.message).join('; ');
            return { success: false, error: `Semantic errors: ${messages}` };
        }

        const output = generate(analyzedProgram);
        return { success: true, output };
    }
    catch (err) {
        return { success: false, error: err instanceof Error ? err.message : String(err) };
    }
}

/**
 * Check if output matches expectation.
 * Returns a result object instead of throwing (for harness use).
 */
export function matchOutput(output: string, expected: string | string[] | TargetExpectation): MatchResult {
    const errors: string[] = [];

    if (typeof expected === 'string') {
        if (output.trim() !== expected) {
            errors.push(`Expected exact match:\n  expected: ${JSON.stringify(expected)}\n  actual: ${JSON.stringify(output.trim())}`);
        }
    }
    else if (Array.isArray(expected)) {
        for (const fragment of expected) {
            if (!output.includes(fragment)) {
                errors.push(`Missing fragment: ${JSON.stringify(fragment)}`);
            }
        }
    }
    else {
        if (expected.exact !== undefined) {
            if (output.trim() !== expected.exact) {
                errors.push(`Expected exact match:\n  expected: ${JSON.stringify(expected.exact)}\n  actual: ${JSON.stringify(output.trim())}`);
            }
        }
        if (expected.contains) {
            for (const fragment of expected.contains) {
                if (!output.includes(fragment)) {
                    errors.push(`Missing fragment: ${JSON.stringify(fragment)}`);
                }
            }
        }
        if (expected.not_contains) {
            for (const fragment of expected.not_contains) {
                if (output.includes(fragment)) {
                    errors.push(`Unexpected fragment present: ${JSON.stringify(fragment)}`);
                }
            }
        }
    }

    return { passed: errors.length === 0, errors };
}

/**
 * Check if error matches errata expectation.
 */
export function matchErrata(errorMessage: string, expected: ErrataExpectation): MatchResult {
    const errors: string[] = [];

    if (expected === true) {
        // Any error is acceptable
        return { passed: true, errors: [] };
    }
    else if (typeof expected === 'string') {
        if (errorMessage !== expected) {
            errors.push(`Expected exact error:\n  expected: ${JSON.stringify(expected)}\n  actual: ${JSON.stringify(errorMessage)}`);
        }
    }
    else {
        for (const fragment of expected) {
            if (!errorMessage.includes(fragment)) {
                errors.push(`Error missing fragment: ${JSON.stringify(fragment)}`);
            }
        }
    }

    return { passed: errors.length === 0, errors };
}

/**
 * Load test cases from a YAML file.
 */
export function loadTestFile(filePath: string, baseDir: string): TestFile {
    const content = readFileSync(filePath, 'utf-8');
    const cases: TestCase[] = parseYaml(content) ?? [];
    const feature = featureFromFilename(filePath);
    const relPath = relative(baseDir, filePath);
    const suiteName = relPath.replace(/\.yaml$/, '');

    return { path: filePath, feature, suiteName, cases };
}

/**
 * Recursively find all YAML test files in a directory.
 */
export function findYamlFiles(dir: string, baseDir: string): TestFile[] {
    const results: TestFile[] = [];

    for (const entry of readdirSync(dir)) {
        const fullPath = join(dir, entry);
        const stat = statSync(fullPath);

        if (stat.isDirectory()) {
            results.push(...findYamlFiles(fullPath, baseDir));
        }
        else if (entry.endsWith('.yaml')) {
            results.push(loadTestFile(fullPath, baseDir));
        }
    }

    return results;
}
