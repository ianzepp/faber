#!/usr/bin/env bun
/**
 * Compile all exempla/*.fab files to opus/exempla/
 *
 * Usage:
 *   bun scripta/build-exempla.ts           # TypeScript output (default)
 *   bun scripta/build-exempla.ts -t zig    # Zig output
 *   bun scripta/build-exempla.ts -t all    # Both targets
 */

import { readdir, mkdir } from 'fs/promises';
import { existsSync } from 'fs';
import { join, basename } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const EXEMPLA = join(ROOT, 'exempla');
const OUTPUT = join(ROOT, 'opus', 'exempla');
const FABER = join(ROOT, 'opus', 'faber');

type Target = 'ts' | 'zig';

async function main() {
    const args = process.argv.slice(2);
    const targetArg = args.includes('-t') ? args[args.indexOf('-t') + 1] : 'ts';

    const targets: Target[] = targetArg === 'all' ? ['ts', 'zig'] : [targetArg as Target];

    // Ensure faber executable exists
    if (!existsSync(FABER)) {
        console.log('Building faber executable...');
        await $`${ROOT}/scripta/build`;
    }

    // Create output directories
    for (const target of targets) {
        await mkdir(join(OUTPUT, target), { recursive: true });
    }

    // Find all .fab files
    const files = (await readdir(EXEMPLA)).filter(f => f.endsWith('.fab'));

    console.log(`Compiling ${files.length} files...`);

    let failed = 0;

    for (const file of files) {
        const name = basename(file, '.fab');
        const input = join(EXEMPLA, file);

        for (const target of targets) {
            const ext = target === 'ts' ? 'ts' : 'zig';
            const output = join(OUTPUT, target, `${name}.${ext}`);

            try {
                const result = await $`${FABER} compile ${input} -t ${target}`.quiet();
                await Bun.write(output, result.stdout);
                console.log(`  ${file} -> opus/exempla/${target}/${name}.${ext}`);
            }
            catch (err: any) {
                console.error(`  ${file} [${target}] FAILED`);
                if (err.stderr) console.error(err.stderr);
                failed++;
            }
        }
    }

    if (failed > 0) {
        console.log(`\n${failed} compilation(s) failed.`);
        process.exit(1);
    }

    console.log('Done.');
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
