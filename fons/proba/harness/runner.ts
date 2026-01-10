#!/usr/bin/env bun
/**
 * Test harness runner - executes all YAML tests and records results to SQLite.
 *
 * USAGE
 *   bun run fons/proba/harness/runner.ts                    Run all tests
 *   bun run fons/proba/harness/runner.ts --verify           Run with verification
 *   bun run fons/proba/harness/runner.ts --db path/to.db    Custom database path
 *   bun run fons/proba/harness/runner.ts --compiler faber   Compiler (faber|rivus|artifex)
 *   bun run fons/proba/harness/runner.ts --targets ts,py    Specific targets only
 *   bun run fons/proba/harness/runner.ts --feature si       Filter by feature
 *
 * OUTPUT
 *   Results are stored in opus/proba/results.db by default.
 *   Use report.ts to generate feature matrix from results.
 */

import { parseArgs } from 'util';
import { resolve, dirname } from 'path';

import {
    EXECUTABLE_TARGETS,
    type ExecutableTarget,
    type TestCase,
    type TestFile,
    findYamlFiles,
    getSource,
    getExpectation,
    getFeatureName,
    shouldSkipFaber,
    hasErrata,
    compile,
    compileStrict,
    matchOutput,
    matchErrata,
} from '../shared';

import { openDatabase, createTestRun, insertResult, type Compiler } from './schema';
import { verify, getAvailableVerifiers } from './verify';

interface RunOptions {
    dbPath: string;
    compiler: Compiler;
    doVerify: boolean;
    targets: ExecutableTarget[];
    featureFilter?: string;
    verbose: boolean;
}

function parseOptions(): RunOptions {
    const { values } = parseArgs({
        args: Bun.argv.slice(2),
        options: {
            db: { type: 'string', default: 'opus/proba/results.db' },
            compiler: { type: 'string', default: 'faber' },
            verify: { type: 'boolean', default: false },
            targets: { type: 'string', default: '' },
            feature: { type: 'string', default: '' },
            verbose: { type: 'boolean', short: 'v', default: false },
        },
    });

    const targets = values.targets
        ? (values.targets.split(',') as ExecutableTarget[])
        : [...EXECUTABLE_TARGETS];

    return {
        dbPath: resolve(values.db!),
        compiler: values.compiler as Compiler,
        doVerify: values.verify!,
        targets,
        featureFilter: values.feature || undefined,
        verbose: values.verbose!,
    };
}

interface Stats {
    total: number;
    passed: number;
    failed: number;
    skipped: number;
    verified: number;
    verifyFailed: number;
}

function runTest(
    testFile: TestFile,
    tc: TestCase,
    target: ExecutableTarget,
    options: RunOptions
): {
    codegen_ok: boolean;
    verify_ok: boolean | null;
    passed: boolean;
    error_msg: string | null;
    codegen: string | null;
} {
    const input = getSource(tc);
    const expected = getExpectation(tc, target);

    // No expectation for this target
    if (expected === undefined) {
        return {
            codegen_ok: false,
            verify_ok: null,
            passed: false,
            error_msg: 'No expectation defined for target',
            codegen: null,
        };
    }

    // Compile
    const result = compile(input.trim(), target);

    if (!result.success) {
        return {
            codegen_ok: false,
            verify_ok: null,
            passed: false,
            error_msg: result.error ?? 'Compilation failed',
            codegen: null,
        };
    }

    // Check output matches expectation
    const match = matchOutput(result.output!, expected);

    if (!match.passed) {
        return {
            codegen_ok: true,
            verify_ok: null,
            passed: false,
            error_msg: match.errors.join('\n'),
            codegen: result.output!,
        };
    }

    // Verification (if enabled)
    let verify_ok: boolean | null = null;
    let verifyError: string | null = null;

    if (options.doVerify) {
        const verifyResult = verify(target, result.output!);
        verify_ok = verifyResult.valid;
        if (!verifyResult.valid) {
            verifyError = verifyResult.error ?? 'Verification failed';
        }
    }

    return {
        codegen_ok: true,
        verify_ok,
        passed: match.passed && (verify_ok === null || verify_ok),
        error_msg: verifyError,
        codegen: result.output!,
    };
}

function runErrataTest(
    testFile: TestFile,
    tc: TestCase & { errata: true | string | string[] }
): {
    codegen_ok: boolean;
    verify_ok: boolean | null;
    passed: boolean;
    error_msg: string | null;
    codegen: string | null;
} {
    const input = getSource(tc);
    const result = compileStrict(input.trim());

    if (result.success) {
        return {
            codegen_ok: true,
            verify_ok: null,
            passed: false,
            error_msg: 'Expected compilation to fail, but it succeeded',
            codegen: result.output!,
        };
    }

    const match = matchErrata(result.error!, tc.errata);

    return {
        codegen_ok: false,
        verify_ok: null,
        passed: match.passed,
        error_msg: match.passed ? null : match.errors.join('\n'),
        codegen: null,
    };
}

async function main() {
    const options = parseOptions();

    // Find test directory relative to this file
    const harnessDir = dirname(import.meta.path.replace('file://', ''));
    const probaDir = resolve(harnessDir, '..');
    const testFiles = findYamlFiles(probaDir, probaDir);

    // Filter test files if feature filter specified
    const filteredFiles = options.featureFilter
        ? testFiles.filter(f =>
            f.feature === options.featureFilter ||
            f.feature.startsWith(options.featureFilter + ':'))
        : testFiles;

    if (options.verbose) {
        console.log(`Database: ${options.dbPath}`);
        console.log(`Compiler: ${options.compiler}`);
        console.log(`Verify: ${options.doVerify}`);
        console.log(`Targets: ${options.targets.join(', ')}`);
        console.log(`Test files: ${filteredFiles.length}`);
        if (options.doVerify) {
            console.log(`Available verifiers: ${getAvailableVerifiers().join(', ')}`);
        }
        console.log();
    }

    // Open database and create test run
    const db = openDatabase(options.dbPath);
    const runId = createTestRun(db);

    console.log(`Test run #${runId} started\n`);

    const stats: Stats = {
        total: 0,
        passed: 0,
        failed: 0,
        skipped: 0,
        verified: 0,
        verifyFailed: 0,
    };

    for (const testFile of filteredFiles) {
        if (options.verbose) {
            console.log(`\n${testFile.feature}:`);
        }

        for (const tc of testFile.cases) {
            if (shouldSkipFaber(tc)) {
                stats.skipped++;
                continue;
            }

            const feature = getFeatureName(testFile.feature, tc);

            const source = getSource(tc);

            // Handle errata tests (expect compilation to fail)
            if (hasErrata(tc)) {
                stats.total++;
                const result = runErrataTest(testFile, tc);

                insertResult(db, runId, {
                    compiler: options.compiler,
                    feature,
                    target: 'errata',
                    file: testFile.suiteName,
                    test_name: tc.name,
                    source,
                    ...result,
                });

                if (result.passed) {
                    stats.passed++;
                    if (options.verbose) console.log(`  ✓ ${tc.name} (errata)`);
                }
                else {
                    stats.failed++;
                    if (options.verbose) console.log(`  ✗ ${tc.name} (errata): ${result.error_msg}`);
                }

                continue;
            }

            // Run against each target
            for (const target of options.targets) {
                const expected = getExpectation(tc, target);

                // Skip if no expectation defined
                if (expected === undefined) {
                    continue;
                }

                // Skip if explicitly marked to skip
                if (tc.skip?.includes(target)) {
                    stats.skipped++;
                    continue;
                }

                stats.total++;
                const result = runTest(testFile, tc, target, options);

                insertResult(db, runId, {
                    compiler: options.compiler,
                    feature,
                    target,
                    file: testFile.suiteName,
                    test_name: tc.name,
                    source,
                    ...result,
                });

                if (result.passed) {
                    stats.passed++;
                    if (result.verify_ok) stats.verified++;
                    if (options.verbose) {
                        const verifyMark = result.verify_ok ? ' [verified]' : '';
                        console.log(`  ✓ ${tc.name} @${target}${verifyMark}`);
                    }
                }
                else {
                    stats.failed++;
                    if (result.verify_ok === false) stats.verifyFailed++;
                    if (options.verbose) {
                        console.log(`  ✗ ${tc.name} @${target}: ${result.error_msg?.split('\n')[0]}`);
                    }
                }
            }
        }
    }

    db.close();

    // Print summary
    console.log('\n' + '='.repeat(60));
    console.log('SUMMARY');
    console.log('='.repeat(60));
    console.log(`Total:    ${stats.total}`);
    console.log(`Passed:   ${stats.passed}`);
    console.log(`Failed:   ${stats.failed}`);
    console.log(`Skipped:  ${stats.skipped}`);
    if (options.doVerify) {
        console.log(`Verified: ${stats.verified}`);
        console.log(`Verify failed: ${stats.verifyFailed}`);
    }
    console.log('='.repeat(60));
    console.log(`Results saved to: ${options.dbPath}`);
    console.log(`Run ID: ${runId}`);
    console.log();

    // Exit with error code if any tests failed
    process.exit(stats.failed > 0 ? 1 : 0);
}

main();
