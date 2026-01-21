#!/usr/bin/env bun
/**
 * Sanity Check Tool - Validate codegen implementation vs test presence
 *
 * Compares fons/codegen/**\/*.ts implementations against proba/codegen/*.yaml tests.
 * Identifies missing tests and incomplete language coverage.
 *
 * NOTE: This checks "feature coverage" (does a test file exist with expectations?),
 *       NOT code/line coverage. Use `bun test --coverage` for actual code coverage.
 *
 * USAGE
 *   bun run sanity              Run all checks with summary
 *   bun run sanity --verbose    Show detailed per-file breakdown
 *   bun run sanity --missing    Only show implementations without tests
 *   bun run sanity --coverage   Only show feature coverage percentages
 *   bun run sanity --targets    Only show per-target feature coverage
 *   bun run sanity --json       Output JSON for tooling integration
 *
 * EXIT CODES
 *   0  All checks passed
 *   1  Issues found (missing tests, incomplete coverage)
 */

import { readdirSync, statSync, readFileSync, existsSync } from 'fs';
import { join, basename, relative } from 'path';
import { parse as parseYaml } from 'yaml';

// =============================================================================
// Types
// =============================================================================

interface Implementation {
    name: string; // e.g., "si", "binary"
    category: string; // "expressions" or "statements"
    targets: string[]; // which targets have this impl
    path: string; // relative path to one impl file
}

interface TestFile {
    name: string;
    category: string;
    path: string;
    testCount: number;
    targetCoverage: Record<string, number>; // target -> count of tests with that target
}

interface SanityReport {
    implementations: Implementation[];
    tests: TestFile[];
    missingTests: Implementation[]; // impls without any test file
    incompleteTests: Array<{
        impl: Implementation;
        test: TestFile;
        missingTargets: string[];
    }>;
    targetCoverage: Record<string, { total: number; covered: number; percentage: number }>;
    overallCoverage: { total: number; covered: number; percentage: number };
}

// =============================================================================
// Constants
// =============================================================================

const ROOT = join(import.meta.dir, '..');
const CODEGEN_DIR = join(ROOT, 'fons', 'faber-ts', 'codegen');
const TESTS_DIR = join(ROOT, 'fons', 'proba', 'codegen');
const TARGETS = ['ts', 'py', 'rs', 'cpp', 'zig'] as const;
type Target = (typeof TARGETS)[number];

// Files to ignore (not feature implementations)
const IGNORE_FILES = new Set(['index.ts', 'generator.ts', 'types.ts']);

// =============================================================================
// Discovery Functions
// =============================================================================

/**
 * Find all codegen implementation files grouped by feature name.
 *
 * WHY: Each feature (e.g., "si") has an implementation in each target directory.
 *      We want to know which features exist and which targets support them.
 */
function discoverImplementations(): Implementation[] {
    const implMap = new Map<string, Implementation>();

    for (const target of TARGETS) {
        const targetDir = join(CODEGEN_DIR, target);
        if (!existsSync(targetDir)) continue;

        for (const category of ['expressions', 'statements']) {
            const categoryDir = join(targetDir, category);
            if (!existsSync(categoryDir)) continue;

            for (const file of readdirSync(categoryDir)) {
                if (!file.endsWith('.ts') || IGNORE_FILES.has(file)) continue;

                const name = basename(file, '.ts');
                const key = `${category}/${name}`;

                if (!implMap.has(key)) {
                    implMap.set(key, {
                        name,
                        category,
                        targets: [],
                        path: relative(ROOT, join(categoryDir, file)),
                    });
                }

                implMap.get(key)!.targets.push(target);
            }
        }
    }

    return Array.from(implMap.values()).sort((a, b) =>
        a.category === b.category ? a.name.localeCompare(b.name) : a.category.localeCompare(b.category),
    );
}

/**
 * Find all YAML test files and analyze their coverage.
 */
function discoverTests(): TestFile[] {
    const tests: TestFile[] = [];

    for (const category of ['expressions', 'statements']) {
        const categoryDir = join(TESTS_DIR, category);
        if (!existsSync(categoryDir)) continue;

        for (const file of readdirSync(categoryDir)) {
            if (!file.endsWith('.yaml')) continue;

            const name = basename(file, '.yaml');
            const path = join(categoryDir, file);
            const content = readFileSync(path, 'utf-8');

            let cases: any[];
            try {
                cases = parseYaml(content) || [];
            } catch {
                cases = [];
            }

            // Count tests per target
            const targetCoverage: Record<string, number> = {};
            for (const target of TARGETS) {
                targetCoverage[target] = 0;
            }

            for (const tc of cases) {
                for (const target of TARGETS) {
                    // Check both modern (expect.ts) and legacy (tc.ts) formats
                    const hasExpectation = tc.expect ? tc.expect[target] !== undefined : tc[target] !== undefined;
                    const isSkipped = tc.skip?.includes(target);
                    if (hasExpectation && !isSkipped) {
                        const count = targetCoverage[target];
                        targetCoverage[target] = (count ?? 0) + 1;
                    }
                }
            }

            tests.push({
                name,
                category,
                path: relative(ROOT, path),
                testCount: cases.length,
                targetCoverage,
            });
        }
    }

    return tests.sort((a, b) => (a.category === b.category ? a.name.localeCompare(b.name) : a.category.localeCompare(b.category)));
}

// =============================================================================
// Analysis Functions
// =============================================================================

/**
 * Generate complete sanity report.
 */
function analyze(): SanityReport {
    const implementations = discoverImplementations();
    const tests = discoverTests();

    // Build lookup for tests by name
    const testMap = new Map<string, TestFile>();
    for (const test of tests) {
        testMap.set(`${test.category}/${test.name}`, test);
    }

    // Find implementations without tests
    const missingTests: Implementation[] = [];
    const incompleteTests: SanityReport['incompleteTests'] = [];

    for (const impl of implementations) {
        const key = `${impl.category}/${impl.name}`;
        const test = testMap.get(key);

        if (!test) {
            missingTests.push(impl);
        } else {
            // Check if all implemented targets have test coverage
            const missingTargets = impl.targets.filter(t => test.targetCoverage[t] === 0);
            if (missingTargets.length > 0) {
                incompleteTests.push({ impl, test, missingTargets });
            }
        }
    }

    // Calculate per-target coverage
    const targetCoverage: Record<string, { total: number; covered: number; percentage: number }> = {};

    for (const target of TARGETS) {
        // Total = implementations that have this target
        const total = implementations.filter(i => i.targets.includes(target)).length;
        // Covered = implementations that have both: this target AND a test with expectations for this target
        const covered = implementations.filter(i => {
            if (!i.targets.includes(target)) return false;
            const test = testMap.get(`${i.category}/${i.name}`);
            return test && (test.targetCoverage[target] ?? 0) > 0;
        }).length;
        const percentage = total > 0 ? Math.round((covered / total) * 100) : 100;
        targetCoverage[target] = { total, covered, percentage };
    }

    // Overall coverage: what % of impl files have corresponding test files?
    const overallTotal = implementations.length;
    const overallCovered = implementations.filter(i => testMap.has(`${i.category}/${i.name}`)).length;
    const overallPercentage = overallTotal > 0 ? Math.round((overallCovered / overallTotal) * 100) : 100;

    return {
        implementations,
        tests,
        missingTests,
        incompleteTests,
        targetCoverage,
        overallCoverage: { total: overallTotal, covered: overallCovered, percentage: overallPercentage },
    };
}

// =============================================================================
// Output Formatters
// =============================================================================

function printSummary(report: SanityReport): void {
    console.log('\n' + '='.repeat(70));
    console.log('SANITY CHECK REPORT');
    console.log('='.repeat(70));

    // Overall stats
    console.log('\nOVERVIEW');
    console.log(`  Implementations:        ${report.implementations.length}`);
    console.log(`  Test files:             ${report.tests.length}`);
    console.log(`  Feature coverage exists: ${report.overallCoverage.percentage}%`);

    // Target coverage
    console.log('\nFEATURE COVERAGE EXISTS (by target)');
    for (const target of TARGETS) {
        const cov = report.targetCoverage[target]!;
        const bar = progressBar(cov.percentage, 20);
        console.log(`  ${target.padEnd(4)} ${bar} ${cov.percentage}% (${cov.covered}/${cov.total})`);
    }

    // Missing tests
    if (report.missingTests.length > 0) {
        console.log('\nMISSING TESTS (implementations without test files)');
        for (const impl of report.missingTests) {
            console.log(`  ${impl.category}/${impl.name}`);
            console.log(`    -> targets: ${impl.targets.join(', ')}`);
        }
    } else {
        console.log('\nMISSING TESTS: None');
    }

    // Incomplete tests
    if (report.incompleteTests.length > 0) {
        console.log('\nINCOMPLETE COVERAGE (test exists but missing target expectations)');
        for (const { impl, test, missingTargets } of report.incompleteTests) {
            console.log(`  ${impl.category}/${impl.name}`);
            console.log(`    -> missing: ${missingTargets.join(', ')}`);
        }
    } else {
        console.log('\nINCOMPLETE COVERAGE: None');
    }

    console.log('\n' + '='.repeat(70));
}

function printVerbose(report: SanityReport): void {
    printSummary(report);

    console.log('\nDETAILED BREAKDOWN');

    // Group by category
    for (const category of ['expressions', 'statements']) {
        console.log(`\n  ${category.toUpperCase()}`);

        const impls = report.implementations.filter(i => i.category === category);

        for (const impl of impls) {
            const testKey = `${impl.category}/${impl.name}`;
            const test = report.tests.find(t => `${t.category}/${t.name}` === testKey);

            const status = test ? (test.testCount > 0 ? 'OK' : 'EMPTY') : 'MISSING';
            const statusColor = status === 'OK' ? '\x1b[32m' : status === 'EMPTY' ? '\x1b[33m' : '\x1b[31m';

            console.log(`    ${impl.name.padEnd(20)} [${statusColor}${status}\x1b[0m]`);

            if (test) {
                const targetStatus = TARGETS.map(t => {
                    const count = test.targetCoverage[t] ?? 0;
                    const hasImpl = impl.targets.includes(t);
                    if (!hasImpl) return `${t}:-`;
                    return count > 0 ? `${t}:${count}` : `\x1b[31m${t}:0\x1b[0m`;
                }).join(' ');
                console.log(`      ${targetStatus}`);
            } else {
                console.log(`      Implemented in: ${impl.targets.join(', ')}`);
            }
        }
    }
}

function printMissingOnly(report: SanityReport): void {
    if (report.missingTests.length === 0) {
        console.log('All implementations have corresponding test files.');
        return;
    }

    console.log('Implementations without test files:\n');

    for (const impl of report.missingTests) {
        console.log(`${impl.category}/${impl.name}`);
        console.log(`  path: ${impl.path}`);
        console.log(`  targets: ${impl.targets.join(', ')}`);
        console.log();
    }
}

function printCoverageOnly(report: SanityReport): void {
    console.log('Feature Coverage Exists:\n');

    console.log(`Overall: ${report.overallCoverage.percentage}% (${report.overallCoverage.covered}/${report.overallCoverage.total})`);
    console.log();

    for (const target of TARGETS) {
        const cov = report.targetCoverage[target]!;
        const bar = progressBar(cov.percentage, 30);
        console.log(`${target.padEnd(4)} ${bar} ${cov.percentage}%`);
    }
}

function printTargetsOnly(report: SanityReport): void {
    console.log('Feature Coverage Exists (by target):\n');

    for (const target of TARGETS) {
        console.log(`\n${target.toUpperCase()}`);
        console.log('-'.repeat(40));

        const cov = report.targetCoverage[target]!;
        console.log(`  Features with tests: ${cov.percentage}% (${cov.covered}/${cov.total})`);

        // List tests with this target
        const testsWithoutTarget = report.tests.filter(t => {
            // Only show if implementation exists for this target
            const impl = report.implementations.find(i => i.category === t.category && i.name === t.name && i.targets.includes(target));
            return impl && (t.targetCoverage[target] ?? 0) === 0;
        });

        if (testsWithoutTarget.length > 0) {
            console.log(`  Missing expectations:`);
            for (const test of testsWithoutTarget) {
                console.log(`    - ${test.category}/${test.name}`);
            }
        }
    }
}

function printJson(report: SanityReport): void {
    console.log(
        JSON.stringify(
            {
                summary: {
                    implementations: report.implementations.length,
                    tests: report.tests.length,
                    overallCoverage: report.overallCoverage,
                    targetCoverage: report.targetCoverage,
                },
                missingTests: report.missingTests.map(i => ({
                    name: `${i.category}/${i.name}`,
                    targets: i.targets,
                })),
                incompleteTests: report.incompleteTests.map(({ impl, missingTargets }) => ({
                    name: `${impl.category}/${impl.name}`,
                    missingTargets,
                })),
            },
            null,
            2,
        ),
    );
}

// =============================================================================
// Helpers
// =============================================================================

function progressBar(percentage: number, width: number): string {
    // Clamp to 0-100 to avoid negative repeat counts
    const clamped = Math.max(0, Math.min(100, percentage));
    const filled = Math.round((clamped / 100) * width);
    const empty = width - filled;
    return '[' + '#'.repeat(filled) + '-'.repeat(empty) + ']';
}

// =============================================================================
// Main
// =============================================================================

function main(): void {
    const args = process.argv.slice(2);

    const verbose = args.includes('--verbose') || args.includes('-v');
    const missingOnly = args.includes('--missing');
    const coverageOnly = args.includes('--coverage');
    const targetsOnly = args.includes('--targets');
    const jsonOutput = args.includes('--json');
    const help = args.includes('--help') || args.includes('-h');

    if (help) {
        console.log(`
Sanity Check Tool - Validate codegen implementation vs test coverage

USAGE
  bun run sanity              Run all checks with summary
  bun run sanity --verbose    Show detailed per-file breakdown
  bun run sanity --missing    Only show implementations without tests
  bun run sanity --coverage   Only show coverage percentages
  bun run sanity --targets    Only show per-target coverage
  bun run sanity --json       Output JSON for tooling integration
  bun run sanity --help       Show this help

EXIT CODES
  0  All checks passed
  1  Issues found (missing tests, incomplete coverage)
`);
        process.exit(0);
    }

    const report = analyze();

    if (jsonOutput) {
        printJson(report);
    } else if (missingOnly) {
        printMissingOnly(report);
    } else if (coverageOnly) {
        printCoverageOnly(report);
    } else if (targetsOnly) {
        printTargetsOnly(report);
    } else if (verbose) {
        printVerbose(report);
    } else {
        printSummary(report);
    }

    // Exit with error if issues found
    const hasIssues = report.missingTests.length > 0 || report.incompleteTests.length > 0;
    if (hasIssues && !jsonOutput) {
        process.exit(1);
    }
}

main();
