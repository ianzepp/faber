#!/usr/bin/env bun
/**
 * Full build: norma -> nanus-ts -> nanus-go -> faber -> rivus -> artifex
 *
 * norma runs first to generate registry files that faber needs to compile.
 * nanus is the minimal compiler for bootstrapping.
 * Each compiler stage is verified by running build:exempla against it.
 *
 * Usage:
 *   bun run build                        # faber + rivus (default)
 *   bun run build -t zig                 # faber + rivus, Zig target
 *   bun run build --artifex              # faber + rivus + artifex
 *   bun run build --no-faber --rivus     # rivus only
 *   bun run build --verbose              # show subprocess output
 */

import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');

type Target = 'ts' | 'zig' | 'py' | 'rs' | 'all';

const VALID_TARGETS = ['ts', 'zig', 'py', 'rs', 'all'] as const;

interface BuildOptions {
    target: Target;
    faber: boolean;
    rivus: boolean;
    artifex: boolean;
    typecheck: boolean;
    verbose: boolean;
}

function parseArgs(): BuildOptions {
    const args = process.argv.slice(2);
    let target: Target = 'ts';
    let faber = true;
    let rivus = true;
    let artifex = false;
    let typecheck = true;
    let verbose = false;

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];

        if (arg === '-t' || arg === '--target') {
            const t = args[++i];
            if (!VALID_TARGETS.includes(t as Target)) {
                console.error(`Unknown target '${t}'. Valid: ${VALID_TARGETS.join(', ')}`);
                process.exit(1);
            }
            target = t as Target;
        } else if (arg === '--faber') {
            faber = true;
        } else if (arg === '--no-faber') {
            faber = false;
        } else if (arg === '--rivus') {
            rivus = true;
        } else if (arg === '--no-rivus') {
            rivus = false;
        } else if (arg === '--artifex') {
            artifex = true;
        } else if (arg === '--no-artifex') {
            artifex = false;
        } else if (arg === '--typecheck') {
            typecheck = true;
        } else if (arg === '--no-typecheck') {
            typecheck = false;
        } else if (arg === '-v' || arg === '--verbose') {
            verbose = true;
        }
    }

    return { target, faber, rivus, artifex, typecheck, verbose };
}

/**
 * Execute a build step with timing and optional verbose output
 */
async function step(name: string, verbose: boolean, fn: () => Promise<void>) {
    const start = performance.now();

    if (verbose) {
        console.log(`\n=== ${name} ===\n`);
    } else {
        process.stdout.write(`${name}... `);
    }

    await fn();

    const elapsed = performance.now() - start;
    if (verbose) {
        console.log(`\n=== ${name} OK (${elapsed.toFixed(0)}ms) ===`);
    } else {
        console.log(`OK (${elapsed.toFixed(0)}ms)`);
    }
}

async function main() {
    const { target, faber, rivus, artifex, typecheck, verbose } = parseArgs();
    const start = performance.now();
    const tcFlag = typecheck ? '' : '--no-typecheck';

    const stages = [faber && 'faber', rivus && 'rivus', artifex && 'artifex'].filter(Boolean);
    console.log(`Build (target: ${target}, stages: ${stages.join(', ') || 'none'})\n`);

    // Generate norma registry from .fab files (required by faber)
    await step('build:norma', verbose, async () => {
        if (verbose) {
            await $`bun run build:norma`;
        } else {
            await $`bun run build:norma`.quiet();
        }
    });

    // Build minimal TypeScript compiler (bootstrapping)
    await step('build:nanus-ts', verbose, async () => {
        if (verbose) {
            await $`bun run build:nanus-ts`;
        } else {
            await $`bun run build:nanus-ts`.quiet();
        }
    });

    // Build minimal Go compiler (bootstrapping)
    await step('build:nanus-go', verbose, async () => {
        if (verbose) {
            await $`bun run build:nanus-go`;
        } else {
            await $`bun run build:nanus-go`.quiet();
        }
    });

    if (faber) {
        await step('build:faber', verbose, async () => {
            if (verbose) {
                await $`bun run build:faber`;
            } else {
                await $`bun run build:faber`.quiet();
            }
        });
    }

    if (rivus) {
        await step('build:rivus', verbose, async () => {
            if (verbose) {
                if (tcFlag) {
                    await $`bun run build:rivus -- ${tcFlag}`;
                } else {
                    await $`bun run build:rivus`;
                }
            } else {
                if (tcFlag) {
                    await $`bun run build:rivus -- ${tcFlag}`.quiet();
                } else {
                    await $`bun run build:rivus`.quiet();
                }
            }
        });
    }

    if (artifex) {
        await step('build:artifex', verbose, async () => {
            if (verbose) {
                if (tcFlag) {
                    await $`bun run build:artifex -- ${tcFlag}`;
                } else {
                    await $`bun run build:artifex`;
                }
            } else {
                if (tcFlag) {
                    await $`bun run build:artifex -- ${tcFlag}`.quiet();
                } else {
                    await $`bun run build:artifex`.quiet();
                }
            }
        });
    }

    // =============================================================================
    // VERIFICATION: Run golden tests to ensure compilers work correctly
    // =============================================================================

    await step('golden:nanus-ts', verbose, async () => {
        if (verbose) {
            await $`bun run golden -c nanus-ts`;
        } else {
            await $`bun run golden -c nanus-ts`.quiet();
        }
    });

    await step('golden:nanus-go', verbose, async () => {
        if (verbose) {
            await $`bun run golden -c nanus-go`;
        } else {
            await $`bun run golden -c nanus-go`.quiet();
        }
    });

    if (faber) {
        await step('golden:faber', verbose, async () => {
            if (verbose) {
                await $`bun run golden -c faber`;
            } else {
                await $`bun run golden -c faber`.quiet();
            }
        });
    }

    const elapsed = performance.now() - start;
    console.log(`\nBuild complete (${(elapsed / 1000).toFixed(1)}s)`);
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
