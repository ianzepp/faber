#!/usr/bin/env bun
/**
 * Full build pipeline:
 *
 *   Stage 1: nanus-ts, nanus-go, nanus-rs, nanus-py (bootstrap compilers)
 *   Stage 2: rivus via nanus-ts (must succeed)
 *   Stage 3: rivus via nanus-go, nanus-rs, nanus-py (optional, failures noted)
 *   Stage 4: exempla codegen via successful rivus compilers
 *   Stage 5: exempla verification (typecheck generated code)
 *   Stage 6: self-hosting (rivus compiles itself via verified compilers)
 *
 * Prework: wipes opus/* for clean builds.
 *
 * Usage:
 *   bun run build                    # full build (all compilers)
 *   bun run build -t ts              # single target (ts|go|rs|py)
 *   bun run build --verbose          # show subprocess output
 */

import { rm } from 'fs/promises';
import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const OPUS = join(ROOT, 'opus');

const VALID_TARGETS = ['ts', 'go', 'rs', 'py'] as const;
type Target = (typeof VALID_TARGETS)[number];

function parseArgs(): { verbose: boolean; target?: Target } {
    const args = process.argv.slice(2);
    const verbose = args.some(arg => arg === '-v' || arg === '--verbose');

    let target: Target | undefined;
    for (let i = 0; i < args.length; i++) {
        if (args[i] === '-t' || args[i] === '--target') {
            const value = args[i + 1];
            if (!value || value.startsWith('-')) {
                console.error('Error: -t/--target requires a value (ts|go|rs|py)');
                process.exit(1);
            }
            if (!VALID_TARGETS.includes(value as Target)) {
                console.error(`Error: invalid target '${value}'. Valid targets: ${VALID_TARGETS.join(', ')}`);
                process.exit(1);
            }
            target = value as Target;
            break;
        }
    }

    return { verbose, target };
}

interface StepResult {
    name: string;
    success: boolean;
    elapsed: number;
    error?: string;
    retryCommand?: string;
}

/**
 * Execute a build step with timing. Returns result instead of throwing.
 */
async function step(name: string, verbose: boolean, fn: () => Promise<void>, allowFailure = false, retryCommand?: string): Promise<StepResult> {
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

        return { name, success: true, elapsed, retryCommand };
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

        return { name, success: false, elapsed, error, retryCommand };
    }
}

async function main() {
    const { verbose, target } = parseArgs();
    const start = performance.now();
    const allResults: StepResult[] = [];
    let aborted = false;

    if (target) {
        console.log(`Build (target: ${target})\n`);
    } else {
        console.log('Build\n');
    }

    // =============================================================================
    // PREWORK: Clean opus directory
    // =============================================================================

    await step('clean opus/*', verbose, async () => {
        await rm(OPUS, { recursive: true, force: true });
    });

    // =============================================================================
    // STAGE 1: Bootstrap compilers (nanus-*)
    // =============================================================================

    console.log('\n--- Stage 1: Bootstrap compilers ---\n');

    const stage1Compilers = target ? [`nanus-${target}`] : ['nanus-ts', 'nanus-go', 'nanus-rs', 'nanus-py'];

    for (const compiler of stage1Compilers) {
        const result = await step(
            `build:${compiler}`,
            verbose,
            async () => {
                if (verbose) {
                    await $`bun run build:${compiler}`;
                } else {
                    await $`bun run build:${compiler}`.quiet();
                }
            },
            !target, // allow failure only when no target specified
            `bun run build:${compiler}`,
        );
        allResults.push(result);
        if (target && !result.success) {
            aborted = true;
        }
    }

    // =============================================================================
    // STAGE 2: Rivus via nanus compilers
    // =============================================================================

    if (!aborted) {
        if (target) {
            console.log(`\n--- Stage 2: Rivus (nanus-${target}) ---\n`);

            const result = await step(
                `build:rivus (nanus-${target})`,
                verbose,
                async () => {
                    if (verbose) {
                        await $`bun run build:rivus -- -c nanus-${target}`;
                    } else {
                        await $`bun run build:rivus -- -c nanus-${target}`.quiet();
                    }
                },
                false, // must succeed
                `bun run build:rivus -- -c nanus-${target}`,
            );
            allResults.push(result);
            if (!result.success) {
                aborted = true;
            }
        } else {
            console.log('\n--- Stage 2: Rivus (nanus-ts) ---\n');

            const stage2Result = await step('build:rivus (nanus-ts)', verbose, async () => {
                if (verbose) {
                    await $`bun run build:rivus -- -c nanus-ts`;
                } else {
                    await $`bun run build:rivus -- -c nanus-ts`.quiet();
                }
            });
            allResults.push(stage2Result);
        }
    }

    // =============================================================================
    // STAGE 3: Rivus via other nanus compilers (only when no target)
    // =============================================================================

    const stage3Results: StepResult[] = [];
    if (!aborted && !target) {
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
                `bun run build:rivus -- -c ${compiler}`,
            );
            stage3Results.push(result);
            allResults.push(result);
        }
    }

    // =============================================================================
    // STAGE 4: Exempla codegen
    // =============================================================================

    let successfulCompilers: string[] = [];
    if (!aborted) {
        if (target) {
            successfulCompilers = [`rivus-nanus-${target}`];
        } else {
            const stage2Success = allResults.find(r => r.name === 'build:rivus (nanus-ts)')?.success;
            if (stage2Success) successfulCompilers.push('rivus-nanus-ts');
            for (const result of stage3Results) {
                if (result.success) {
                    const compiler = result.name.replace('build:rivus (', '').replace(')', '');
                    successfulCompilers.push(`rivus-${compiler}`);
                }
            }
        }
    }

    const exemplaCodegen: StepResult[] = [];
    if (!aborted && successfulCompilers.length > 0) {
        console.log('\n--- Stage 4: Exempla (codegen) ---\n');

        for (const compiler of successfulCompilers) {
            const result = await step(
                `build:exempla (${compiler})`,
                verbose,
                async () => {
                    if (verbose) {
                        await $`bun run build:exempla -- -c ${compiler} --no-verify`;
                    } else {
                        await $`bun run build:exempla -- -c ${compiler} --no-verify`.quiet();
                    }
                },
                !target, // allow failure only when no target
                `bun run build:exempla -- -c ${compiler} --no-verify`,
            );
            exemplaCodegen.push(result);
            allResults.push(result);
            if (target && !result.success) {
                aborted = true;
                break;
            }
        }
    }

    // =============================================================================
    // STAGE 5: Exempla verification
    // =============================================================================

    let verifiedCompilers: string[] = [];
    if (!aborted) {
        const compilersToVerify = target
            ? successfulCompilers
            : successfulCompilers.filter((_, i) => exemplaCodegen[i]?.success);

        if (compilersToVerify.length > 0) {
            console.log('\n--- Stage 5: Exempla (verify) ---\n');

            for (const compiler of compilersToVerify) {
                const result = await step(
                    `verify:exempla (${compiler})`,
                    verbose,
                    async () => {
                        if (verbose) {
                            await $`bun run build:exempla -- -c ${compiler} --verify-only`;
                        } else {
                            await $`bun run build:exempla -- -c ${compiler} --verify-only`.quiet();
                        }
                    },
                    !target, // allow failure only when no target
                    `bun run build:exempla -- -c ${compiler} --verify-only`,
                );
                allResults.push(result);
                if (result.success) {
                    verifiedCompilers.push(compiler);
                } else if (target) {
                    aborted = true;
                    break;
                }
            }
        }
    }

    // =============================================================================
    // STAGE 6: Self-hosting (rivus compiles itself)
    // =============================================================================

    if (!aborted && verifiedCompilers.length > 0) {
        console.log('\n--- Stage 6: Self-hosting (rivus compiles rivus) ---\n');

        for (const compiler of verifiedCompilers) {
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
                !target, // allow failure only when no target
                `bun run build:rivus -- -c ${compiler}`,
            );
            allResults.push(result);
            if (target && !result.success) {
                aborted = true;
                break;
            }
        }
    }

    // =============================================================================
    // SUMMARY
    // =============================================================================

    const elapsed = performance.now() - start;
    const failedSteps = allResults.filter(r => !r.success);

    if (aborted) {
        console.log(`\nBuild aborted (${(elapsed / 1000).toFixed(1)}s)`);
    } else {
        console.log(`\nBuild complete (${(elapsed / 1000).toFixed(1)}s)`);
    }

    if (failedSteps.length > 0) {
        console.log(`\n${failedSteps.length} step(s) failed. To retry manually:\n`);
        for (const step of failedSteps) {
            if (step.retryCommand) {
                console.log(`  ${step.retryCommand}`);
            }
        }
        if (target) {
            process.exit(1);
        }
    }
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
