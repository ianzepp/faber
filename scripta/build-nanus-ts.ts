#!/usr/bin/env bun
/**
 * Build standalone nanus executable.
 *
 * Steps:
 *   1. Typecheck fons/nanus-ts/ with semi-strict tsconfig
 *   2. Compile nanus CLI to standalone binary
 */

import { mkdir } from 'fs/promises';
import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');

async function main() {
    const start = performance.now();

    // Typecheck fons/nanus-ts/ before compiling
    await $`tsc -p tsconfig.nanus.json`.cwd(ROOT);

    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });
    const outExe = join(binDir, 'nanus-ts');
    await $`bun build ${join(ROOT, 'fons', 'nanus-ts', 'nanus.ts')} --compile --outfile=${outExe}`.quiet();
    await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();

    const elapsed = performance.now() - start;
    console.log(`Built opus/bin/nanus-ts (${elapsed.toFixed(0)}ms)`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
