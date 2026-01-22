#!/usr/bin/env bun
/**
 * Full build pipeline in three stages:
 *
 *   Stage 1: nanus-ts, nanus-go, nanus-rs, nanus-py (bootstrap compilers) + norma
 *   Stage 2: rivus via nanus-ts (must succeed)
 *   Stage 3: rivus via nanus-go, nanus-rs, nanus-py (optional, failures noted)
 *
 * Prework: wipes opus/* for clean builds.
 *
 * Usage:
 *   bun run build              # full build
 *   bun run build --verbose    # show subprocess output
 */

import { rm } from 'fs/promises';
import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const OPUS = join(ROOT, 'opus');

function parseArgs(): { verbose: boolean } {
    const verbose = process.argv.slice(2).some(arg => arg === '-v' || arg === '--verbose');
    return { verbose };
}

interface StepResult {
    name: string;
    success: boolean;
    elapsed: number;
    error?: string;
}

/**
 * Execute a build step with timing. Returns result instead of throwing.
 */
async function step(name: string, verbose: boolean, fn: () => Promise<void>, allowFailure = false): Promise<StepResult> {
    const start = performance.now();

    if (verbose) {
        console.log(`\n=== ${name} ===\n`);
    } else {
        process.stdout.write(`${name}... `);
    }

    try {
        await fn();
        const elapsed = performance.now() - start;

        if (verbose) {
            console.log(`\n=== ${name} OK (${elapsed.toFixed(0)}ms) ===`);
        } else {
            console.log(`OK (${elapsed.toFixed(0)}ms)`);
        }

        return { name, success: true, elapsed };
    } catch (err: any) {
        const elapsed = performance.now() - start;
        const error = err.stderr?.toString().trim() || err.message || 'unknown error';

        if (verbose) {
            console.log(`\n=== ${name} FAILED (${elapsed.toFixed(0)}ms) ===`);
            console.error(error);
        } else {
            console.log(`FAILED (${elapsed.toFixed(0)}ms)`);
        }

        if (!allowFailure) {
            throw err;
        }

        return { name, success: false, elapsed, error };
    }
}

async function main() {
    const { verbose } = parseArgs();
    const start = performance.now();
    const rivusResults: StepResult[] = [];

    console.log('Build\n');

    // =============================================================================
    // PREWORK: Clean opus directory
    // =============================================================================

    await step('clean opus/*', verbose, async () => {
        await rm(OPUS, { recursive: true, force: true });
    });

    // =============================================================================
    // STAGE 1: Bootstrap compilers (nanus-*) + norma
    // =============================================================================

    console.log('\n--- Stage 1: Bootstrap compilers + norma ---\n');

    await step('build:nanus-ts', verbose, async () => {
        if (verbose) {
            await $`bun run build:nanus-ts`;
        } else {
            await $`bun run build:nanus-ts`.quiet();
        }
    });

    await step('build:nanus-go', verbose, async () => {
        if (verbose) {
            await $`bun run build:nanus-go`;
        } else {
            await $`bun run build:nanus-go`.quiet();
        }
    });

    await step('build:nanus-rs', verbose, async () => {
        if (verbose) {
            await $`bun run build:nanus-rs`;
        } else {
            await $`bun run build:nanus-rs`.quiet();
        }
    });

    await step('build:nanus-py', verbose, async () => {
        if (verbose) {
            await $`bun run build:nanus-py`;
        } else {
            await $`bun run build:nanus-py`.quiet();
        }
    });

    await step('build:norma', verbose, async () => {
        if (verbose) {
            await $`bun run build:norma`;
        } else {
            await $`bun run build:norma`.quiet();
        }
    });

    // =============================================================================
    // STAGE 2: Rivus via nanus-ts (must succeed)
    // =============================================================================

    console.log('\n--- Stage 2: Rivus (nanus-ts) ---\n');

    await step('build:rivus (nanus-ts)', verbose, async () => {
        if (verbose) {
            await $`bun run build:rivus -- -c nanus-ts`;
        } else {
            await $`bun run build:rivus -- -c nanus-ts`.quiet();
        }
    });

    // =============================================================================
    // STAGE 3: Rivus via other nanus compilers (optional)
    // =============================================================================

    console.log('\n--- Stage 3: Rivus (other compilers) ---\n');

    const optionalCompilers = ['nanus-go', 'nanus-rs', 'nanus-py'] as const;

    for (const compiler of optionalCompilers) {
        const result = await step(
            `build:rivus (${compiler})`,
            verbose,
            async () => {
                if (verbose) {
                    await $`bun run build:rivus -- -c ${compiler}`;
                } else {
                    await $`bun run build:rivus -- -c ${compiler}`.quiet();
                }
            },
            true, // allow failure
        );
        rivusResults.push(result);
    }

    // =============================================================================
    // SUMMARY
    // =============================================================================

    const elapsed = performance.now() - start;
    console.log(`\nBuild complete (${(elapsed / 1000).toFixed(1)}s)`);

    if (rivusResults.length > 0) {
        const passed = rivusResults.filter(r => r.success).length;
        const failed = rivusResults.filter(r => !r.success);

        console.log(`\nRivus builds: ${passed}/${rivusResults.length} succeeded`);

        if (failed.length > 0) {
            console.log('Failed:');
            for (const f of failed) {
                console.log(`  - ${f.name}`);
            }
        }
    }
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
