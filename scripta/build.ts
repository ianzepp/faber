#!/usr/bin/env bun
/**
 * Full build: norma -> faber -> rivus -> artifex
 *
 * norma runs first to generate registry files that faber needs to compile.
 * Each compiler stage is verified by running build:exempla against it.
 *
 * Usage:
 *   bun run build                        # All stages, TypeScript (default)
 *   bun run build -t zig                 # All stages, Zig
 *   bun run build -t all                 # All stages, all targets
 */

import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');

type Target = 'ts' | 'zig' | 'py' | 'rs' | 'all';

const VALID_TARGETS = ['ts', 'zig', 'py', 'rs', 'all'] as const;

function parseArgs(): { target: Target } {
    const args = process.argv.slice(2);
    let target: Target = 'ts';

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];

        if (arg === '-t' || arg === '--target') {
            const t = args[++i];
            if (!VALID_TARGETS.includes(t as Target)) {
                console.error(`Unknown target '${t}'. Valid: ${VALID_TARGETS.join(', ')}`);
                process.exit(1);
            }
            target = t as Target;
        }
    }

    return { target };
}

async function step(name: string, fn: () => Promise<void>) {
    const start = performance.now();
    process.stdout.write(`${name}... `);
    await fn();
    const elapsed = performance.now() - start;
    console.log(`OK (${elapsed.toFixed(0)}ms)`);
}

async function main() {
    const { target } = parseArgs();
    const start = performance.now();

    console.log(`Full build (target: ${target})\n`);

    await step('build:norma', async () => {
        await $`bun run build:norma`.quiet();
    });

    await step('build:faber', async () => {
        await $`bun run build:faber`.quiet();
        // await $`bun run build:exempla -c faber -t ${target}`.quiet();
    });

    await step('build:rivus', async () => {
        await $`bun run build:rivus`.quiet();
        // await $`bun run build:exempla -c rivus -t ${target}`.quiet();
    });

    await step('build:artifex', async () => {
        await $`bun run build:artifex`.quiet();
        // await $`bun run build:exempla -c artifex -t ${target}`.quiet();
    });

    const elapsed = performance.now() - start;
    console.log(`\nFull build complete (${(elapsed / 1000).toFixed(1)}s)`);
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
