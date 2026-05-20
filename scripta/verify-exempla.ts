#!/usr/bin/env bun
/**
 * Check examples/exempla with the active radix workspace compiler.
 *
 * This is a compiler check pass, not a target runtime verification harness.
 */

import { readdir } from 'fs/promises';
import { statSync } from 'fs';
import { join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const MANIFEST = join(ROOT, 'radix', 'Cargo.toml');
const EXEMPLA = join(ROOT, 'examples', 'exempla');

async function findFabFiles(dir: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        const fullPath = join(dir, entry);
        const info = statSync(fullPath);
        if (info.isDirectory()) {
            files.push(...await findFabFiles(fullPath));
        } else if (entry.endsWith('.fab')) {
            files.push(fullPath);
        }
    }

    return files.sort();
}

async function main() {
    const files = await findFabFiles(EXEMPLA);
    let failed = 0;

    for (const file of files) {
        const rel = relative(EXEMPLA, file);
        const result = await $`cargo run --quiet --manifest-path ${MANIFEST} -p radix -- check ${file}`.nothrow().quiet();

        if (result.exitCode === 0) {
            console.log(`PASS  ${rel}`);
            continue;
        }

        failed++;
        console.log(`FAIL  ${rel}`);
    }

    console.log(`\n${files.length - failed} passed, ${failed} failed`);
    process.exit(failed > 0 ? 1 : 0);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
