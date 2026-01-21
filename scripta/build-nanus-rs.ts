#!/usr/bin/env bun
/**
 * Build standalone nanus-rs executable.
 *
 * Steps:
 *   1. Compile Rust nanus CLI to standalone binary
 */

import { mkdir, copyFile } from 'fs/promises';
import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const NANUS_RS = join(ROOT, 'fons', 'nanus-rs');

async function main() {
    const start = performance.now();

    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });

    await $`cd ${NANUS_RS} && cargo build --release`;

    const builtExe = join(NANUS_RS, 'target', 'release', 'nanus-rs');
    const outExe = join(binDir, 'nanus-rs');
    await copyFile(builtExe, outExe);

    const elapsed = performance.now() - start;
    console.log(`Built opus/bin/nanus-rs (${elapsed.toFixed(0)}ms)`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
