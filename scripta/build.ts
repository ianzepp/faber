#!/usr/bin/env bun
/**
 * Build the active radix workspace binary.
 *
 * This helper mirrors `bun run build:radix` and also places a convenience copy
 * at opus/bin/radix for scripts that want a stable repo-local executable path.
 */

import { copyFile, mkdir } from 'fs/promises';
import { join } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const RADIX_MANIFEST = join(ROOT, 'radix', 'Cargo.toml');
const RADIX_BIN = join(ROOT, 'radix', 'target', 'release', 'radix');
const OPUS_BIN = join(ROOT, 'opus', 'bin');

async function main() {
    await $`cargo build --release --manifest-path ${RADIX_MANIFEST} -p radix`;
    await mkdir(OPUS_BIN, { recursive: true });
    await copyFile(RADIX_BIN, join(OPUS_BIN, 'radix'));
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
