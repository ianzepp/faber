/**
 * Test runner for rivus (bootstrap compiler).
 *
 * Runs YAML test cases through opus/rivus, TS target only.
 * Used to identify codegen gaps between faber and rivus.
 *
 * USAGE
 *   bun test proba/rivus.test.ts              Run all tests
 *   bun test proba/rivus.test.ts -t "binary"  Run tests matching "binary"
 */

import { describe, test, expect } from 'bun:test';
import { parse as parseYaml } from 'yaml';
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, basename } from 'path';

import { lexare } from '../../opus/rivus/fons/ts/lexor/index';
import { resolvere } from '../../opus/rivus/fons/ts/parser/index';
import { analyze } from '../../opus/rivus/fons/ts/semantic/index';
import { generateTs } from '../../opus/rivus/fons/ts/codegen/ts/index';

import type { Programma } from '../../opus/rivus/fons/ts/ast/radix';
import type { Sententia } from '../../opus/rivus/fons/ts/ast/sententia';

// =============================================================================
// TYPES
// =============================================================================

interface TargetExpectation {
    contains?: string[];
    not_contains?: string[];
    exact?: string;
}

type ErrataExpectation = true | string | string[];

interface TestCase {
    name: string;
    source: string;
    wrap?: string;
    expect?: {
        ts?: string | string[] | TargetExpectation;
    };
    // Legacy top-level expectation (deprecated)
    ts?: string | string[] | TargetExpectation;
    skip?: string[];
    errata?: ErrataExpectation;
    rivus?: boolean; // Set to false to skip this test for rivus compiler
}

// =============================================================================
// HELPERS
// =============================================================================

function getSource(tc: TestCase): string {
    if (tc.wrap) {
        return tc.wrap.replace('$', tc.source);
    }
    return tc.source;
}

function getExpectation(tc: TestCase): string | string[] | TargetExpectation | undefined {
    return tc.expect?.ts ?? tc.ts;
}

function hasErrata(tc: TestCase): boolean {
    return 'errata' in tc && tc.errata !== undefined;
}

function shouldSkip(tc: TestCase): boolean {
    if (tc.skip?.includes('ts')) return true;
    if (tc.rivus === false) return true;
    return false;
}

// =============================================================================
// COMPILER
// =============================================================================

async function compile(code: string, options: { strictSemantic?: boolean } = {}): Promise<string> {
    const strictSemantic = options.strictSemantic ?? false;

    const lexResult = lexare(code);
    if (lexResult.errores.length > 0) {
        const msgs = lexResult.errores.map((e: any) => `${e.codice ?? 'L???'} ${e.textus || String(e)}`).join('; ');
        throw new Error(`Tokenizer errors: ${msgs}`);
    }

    const parseResult = resolvere(lexResult.symbola);
    if (parseResult.errores.length > 0) {
        const msgs = parseResult.errores.map((e: any) => `${e.codice ?? 'P???'} ${e.nuntius || String(e)}`).join('; ');
        throw new Error(`Parse errors: ${msgs}`);
    }
    if (!parseResult.programma) {
        throw new Error('Parse failed: no program');
    }

    const semanticResult = analyze(parseResult.programma as Programma);
    if (strictSemantic && semanticResult.errores.length > 0) {
        const msgs = semanticResult.errores.map((e: any) => e.nuntius || e.textus || String(e)).join('; ');
        throw new Error(`Semantic errors: ${msgs}`);
    }

    return generateTs((parseResult.programma as Programma).corpus as Sententia[]);
}

// =============================================================================
// EXPECTATION CHECKING
// =============================================================================

function checkOutput(output: string, expected: string | string[] | TargetExpectation): void {
    if (typeof expected === 'string') {
        expect(output.trim()).toBe(expected);
    } else if (Array.isArray(expected)) {
        for (const fragment of expected) {
            expect(output).toContain(fragment);
        }
    } else {
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

function checkErrata(tc: TestCase, error: Error): void {
    const errata = (tc as any).errata;
    if (errata === true) {
        // Any error is fine
        return;
    }
    if (typeof errata === 'string') {
        expect(error.message).toBe(errata);
    } else if (Array.isArray(errata)) {
        for (const fragment of errata) {
            expect(error.message).toContain(fragment);
        }
    }
}

// =============================================================================
// YAML LOADING
// =============================================================================

interface FileMetadata {
    rivus?: boolean;
}

function loadYamlFiles(dir: string): { file: string; cases: TestCase[]; meta?: FileMetadata }[] {
    const results: { file: string; cases: TestCase[]; meta?: FileMetadata }[] = [];

    function walk(d: string) {
        for (const entry of readdirSync(d)) {
            const path = join(d, entry);
            const stat = statSync(path);

            if (stat.isDirectory()) {
                walk(path);
            } else if (entry.endsWith('.yaml') || entry.endsWith('.yml')) {
                const content = readFileSync(path, 'utf-8');
                const parsed = parseYaml(content);

                // Support file-level metadata: first entry with rivus: false and no name
                if (Array.isArray(parsed)) {
                    let cases = parsed as TestCase[];
                    let meta: FileMetadata | undefined;

                    // Check if first entry is metadata (has rivus but no name/input/faber)
                    const first = cases[0] as any;
                    if (first && 'rivus' in first && !('name' in first) && !('input' in first) && !('faber' in first)) {
                        meta = { rivus: first.rivus };
                        cases = cases.slice(1);
                    }

                    results.push({ file: path, cases, meta });
                }
            }
        }
    }

    walk(dir);
    return results;
}

// =============================================================================
// TEST EXECUTION
// =============================================================================

const probaDir = join(import.meta.dir, '.');
const yamlFiles = loadYamlFiles(probaDir);

for (const { file, cases, meta } of yamlFiles) {
    const suiteName = basename(file, '.yaml');

    // Skip entire file if file-level rivus: false
    if (meta?.rivus === false) {
        describe.skip(`[rivus] ${suiteName}`, () => {});
        continue;
    }

    describe(`[rivus] ${suiteName}`, () => {
        for (const tc of cases) {
            // Skip tests that explicitly skip 'ts'
            if (shouldSkip(tc)) {
                test.skip(`${tc.name} @ts`, () => {});
                continue;
            }

            const expectation = getExpectation(tc);

            // Skip if no TS expectation defined
            if (!expectation && !hasErrata(tc)) {
                continue;
            }

            test(`${tc.name} @ts`, async () => {
                const input = getSource(tc);

                if (hasErrata(tc)) {
                    try {
                        await compile(input.trim(), { strictSemantic: true });
                        throw new Error('Expected compilation to fail');
                    } catch (error: any) {
                        checkErrata(tc, error);
                    }
                } else {
                    // Expect compilation to succeed and match
                    const output = await compile(input.trim());
                    checkOutput(output, expectation!);
                }
            });
        }
    });
}
