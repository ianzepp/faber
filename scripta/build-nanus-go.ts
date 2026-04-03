#!/usr/bin/env bun
/**
 * Build standalone nanus-go executable.
 *
 * Steps:
 *   1. Compile Go nanus CLI to standalone binary
 */

import { mkdir } from 'fs/promises';
import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const NANUS_GO = join(ROOT, 'compilers', 'nanus-go');
const GO_CACHE = '/tmp/go-build';

async function main() {
    const start = performance.now();

    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });
    await mkdir(GO_CACHE, { recursive: true });

    const outExe = join(binDir, 'nanus-go');
    await $`cd ${NANUS_GO} && go build -o ${outExe} .`
        .env({ ...process.env, GOCACHE: GO_CACHE });

    const elapsed = performance.now() - start;
    console.log(`Built opus/bin/nanus-go (${elapsed.toFixed(0)}ms)`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
