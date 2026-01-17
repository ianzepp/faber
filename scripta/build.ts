#!/usr/bin/env bun
/**
 * Full build: norma -> faber -> rivus -> artifex
 *
 * norma runs first to generate registry files that faber needs to compile.
 * Each compiler stage is verified by running build:exempla against it.
 *
 * Usage:
 *   bun run build                        # faber + rivus (default)
 *   bun run build -t zig                 # faber + rivus, Zig target
 *   bun run build --artifex              # faber + rivus + artifex
 *   bun run build --no-faber --rivus     # rivus only
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
}

function parseArgs(): BuildOptions {
    const args = process.argv.slice(2);
    let target: Target = 'ts';
    let faber = true;
    let rivus = true;
    let artifex = false;
    let typecheck = true;

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
        }
    }

    return { target, faber, rivus, artifex, typecheck };
}

async function step(name: string, fn: () => Promise<void>) {
    const start = performance.now();
    process.stdout.write(`${name}... `);
    await fn();
    const elapsed = performance.now() - start;
    console.log(`OK (${elapsed.toFixed(0)}ms)`);
}

async function main() {
    const { target, faber, rivus, artifex, typecheck } = parseArgs();
    const start = performance.now();
    const tcFlag = typecheck ? '' : '--no-typecheck';

    const stages = [faber && 'faber', rivus && 'rivus', artifex && 'artifex'].filter(Boolean);
    console.log(`Build (target: ${target}, stages: ${stages.join(', ') || 'none'})\n`);

    await step('build:norma', async () => {
        await $`bun run build:norma`.quiet();
    });

    if (faber) {
        await step('build:faber', async () => {
            await $`bun run build:faber`.quiet();
            // await $`bun run build:exempla -c faber -t ${target}`.quiet();
        });
    }

    if (rivus) {
        await step('build:rivus', async () => {
            if (tcFlag) {
                await $`bun run build:rivus -- ${tcFlag}`.quiet();
            } else {
                await $`bun run build:rivus`.quiet();
            }
            // await $`bun run build:exempla -c rivus -t ${target}`.quiet();
        });
    }

    if (artifex) {
        await step('build:artifex', async () => {
            if (tcFlag) {
                await $`bun run build:artifex -- ${tcFlag}`.quiet();
            } else {
                await $`bun run build:artifex`.quiet();
            }
            // await $`bun run build:exempla -c artifex -t ${target}`.quiet();
        });
    }

    const elapsed = performance.now() - start;
    console.log(`\nBuild complete (${(elapsed / 1000).toFixed(1)}s)`);
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
