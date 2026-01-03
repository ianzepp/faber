/**
 * Test runner for rivus (bootstrap compiler).
 *
 * Runs YAML test cases through opus/bootstrap, TS target only.
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

import { fileURLToPath } from 'url';

// =============================================================================
// TYPES
// =============================================================================

interface TargetExpectation {
    contains?: string[];
    not_contains?: string[];
    exact?: string;
}

type ErrataExpectation = true | string | string[];

interface LegacyTestCase {
    name: string;
    input: string;
    wrap?: string;
    ts?: string | string[] | TargetExpectation;
    skip?: string[];
    errata?: ErrataExpectation;
}

interface ModernTestCase {
    name: string;
    faber: string;
    wrap?: string;
    expect?: {
        ts?: string | string[] | TargetExpectation;
    };
    skip?: string[];
    errata?: ErrataExpectation;
}

type TestCase = LegacyTestCase | ModernTestCase;

// =============================================================================
// HELPERS
// =============================================================================

function isModernTestCase(tc: TestCase): tc is ModernTestCase {
    return 'faber' in tc;
}

function getInput(tc: TestCase): string {
    const raw = isModernTestCase(tc) ? tc.faber : tc.input;
    if (tc.wrap) {
        return tc.wrap.replace('$', raw);
    }
    return raw;
}

function getExpectation(tc: TestCase): string | string[] | TargetExpectation | undefined {
    return isModernTestCase(tc) ? tc.expect?.ts : tc.ts;
}

function hasErrata(tc: TestCase): boolean {
    return 'errata' in tc && tc.errata !== undefined;
}

function shouldSkip(tc: TestCase): boolean {
    return tc.skip?.includes('ts') ?? false;
}

// =============================================================================
// COMPILER
// =============================================================================

const repoRoot = join(import.meta.dir, '..', '..');
const compileScript = fileURLToPath(new URL('./rivus-compile.ts', import.meta.url));
const COMPILE_TIMEOUT_MS = 3500;

async function readTextWithTimeout(
    stream: ReadableStream,
    timeoutMs: number,
    label: string,
): Promise<string> {
    const timer = setTimeout(() => {
        try {
            stream.cancel(`${label} read timeout`);
        } catch {}
    }, timeoutMs);

    try {
        return await new Response(stream).text();
    } finally {
        clearTimeout(timer);
    }
}

async function compile(code: string, timeoutMs: number = COMPILE_TIMEOUT_MS): Promise<string> {
    const proc = Bun.spawn({
        cmd: ['bun', compileScript],
        cwd: repoRoot,
        stdin: 'pipe',
        stdout: 'pipe',
        stderr: 'pipe',
    });

    proc.stdin.write(code);
    proc.stdin.end();

    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    const timeout = new Promise<never>((_, reject) => {
        timeoutId = setTimeout(() => {
            try {
                proc.kill('SIGKILL');
            } catch {}
            try {
                proc.stdout.cancel('compile timeout');
            } catch {}
            try {
                proc.stderr.cancel('compile timeout');
            } catch {}
            reject(new Error(`Timeout after ${timeoutMs}ms`));
        }, timeoutMs);
    });

    const exitCode = await Promise.race([proc.exited, timeout]).finally(() => {
        if (timeoutId !== undefined) {
            clearTimeout(timeoutId);
        }
    });
    const [stdout, stderr] = await Promise.all([
        readTextWithTimeout(proc.stdout, 250, 'stdout'),
        readTextWithTimeout(proc.stderr, 250, 'stderr'),
    ]);

    const errText = stderr.trim();
    if (exitCode !== 0) {
        throw new Error(errText || `Compile failed (exit ${exitCode})`);
    }
    if (errText.length > 0) {
        throw new Error(errText);
    }
    return stdout;
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

function loadYamlFiles(dir: string): { file: string; cases: TestCase[] }[] {
    const results: { file: string; cases: TestCase[] }[] = [];

    function walk(d: string) {
        for (const entry of readdirSync(d)) {
            const path = join(d, entry);
            const stat = statSync(path);

            if (stat.isDirectory()) {
                walk(path);
            } else if (entry.endsWith('.yaml') || entry.endsWith('.yml')) {
                const content = readFileSync(path, 'utf-8');
                const cases = parseYaml(content) as TestCase[];
                if (Array.isArray(cases)) {
                    results.push({ file: path, cases });
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

for (const { file, cases } of yamlFiles) {
    const suiteName = basename(file, '.yaml');

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
                const input = getInput(tc);

                if (hasErrata(tc)) {
                    try {
                        await compile(input.trim());
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
