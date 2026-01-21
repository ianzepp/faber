#!/usr/bin/env bun
/**
 * Full build pipeline in three stages:
 *
 *   Stage 1: nanus-ts, nanus-go, nanus-rs, nanus-py (bootstrap compilers)
 *   Stage 2: norma (stdlib registry) + faber (main compiler)
 *   Stage 3: rivus built with each nanus compiler (failures noted, not fatal)
 *
 * Prework: wipes opus/* for clean builds.
 *
 * Usage:
 *   bun run build              # full build
 *   bun run build --verbose    # show subprocess output
 *   bun run build --no-faber   # skip faber (stage 2)
 *   bun run build --no-rivus   # skip rivus (stage 3)
 */

import { rm } from 'fs/promises';
import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const OPUS = join(ROOT, 'opus');

interface BuildOptions {
    faber: boolean;
    rivus: boolean;
    verbose: boolean;
}

function parseArgs(): BuildOptions {
    const args = process.argv.slice(2);
    let faber = true;
    let rivus = true;
    let verbose = false;

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];

        if (arg === '--faber') {
            faber = true;
        } else if (arg === '--no-faber') {
            faber = false;
        } else if (arg === '--rivus') {
            rivus = true;
        } else if (arg === '--no-rivus') {
            rivus = false;
        } else if (arg === '-v' || arg === '--verbose') {
            verbose = true;
        }
    }

    return { faber, rivus, verbose };
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
async function step(
    name: string,
    verbose: boolean,
    fn: () => Promise<void>,
    allowFailure = false,
): Promise<StepResult> {
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
    const { faber, rivus, verbose } = parseArgs();
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
    // STAGE 1: Bootstrap compilers (nanus-ts, nanus-go, nanus-rs, nanus-py)
    // =============================================================================

    console.log('\n--- Stage 1: Bootstrap compilers ---\n');

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

    // =============================================================================
    // STAGE 2: Norma stdlib + Faber compiler
    // =============================================================================

    if (faber) {
        console.log('\n--- Stage 2: Norma + Faber ---\n');

        await step('build:norma', verbose, async () => {
            if (verbose) {
                await $`bun run build:norma`;
            } else {
                await $`bun run build:norma`.quiet();
            }
        });

        await step('build:faber-ts', verbose, async () => {
            if (verbose) {
                await $`bun run build:faber-ts`;
            } else {
                await $`bun run build:faber-ts`.quiet();
            }
        });
    }

    // =============================================================================
    // STAGE 3: Rivus with each compiler (faber-ts first, then nanus-*)
    // =============================================================================

    if (rivus) {
        console.log('\n--- Stage 3: Rivus (multi-compiler) ---\n');

        const compilers = ['faber-ts', 'nanus-ts', 'nanus-go', 'nanus-rs', 'nanus-py'] as const;

        for (const compiler of compilers) {
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
