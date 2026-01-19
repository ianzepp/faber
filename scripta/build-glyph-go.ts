#!/usr/bin/env bun
/**
 * Build standalone glyph-go executable.
 *
 * Steps:
 *   1. Compile Go glyph CLI to standalone binary
 */

import { mkdir } from 'fs/promises';
import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const GLYPH_GO = join(ROOT, 'fons', 'glyph-go');
const GO_CACHE = '/tmp/go-build';

async function main() {
    const start = performance.now();

    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });
    await mkdir(GO_CACHE, { recursive: true });

    const outExe = join(binDir, 'glyph-go');
    await $`cd ${GLYPH_GO} && go build -o ${outExe} .`
        .env({ ...process.env, GOCACHE: GO_CACHE });

    const elapsed = performance.now() - start;
    console.log(`Built opus/bin/glyph-go (${elapsed.toFixed(0)}ms)`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
