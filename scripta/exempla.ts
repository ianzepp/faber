#!/usr/bin/env bun
/**
 * Compile all .fab files in exempla/ to opus/exempla/
 *
 * Recursively scans exempla/ directory and mirrors structure in output.
 *
 * Usage:
 *   bun scripta/build-exempla.ts           # TypeScript output (default)
 *   bun scripta/build-exempla.ts -t zig    # Zig output
 *   bun scripta/build-exempla.ts -t all    # Both targets
 */

import { readdir, mkdir } from 'fs/promises';
import { statSync } from 'fs';
import { join, basename, relative, dirname } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const EXEMPLA = join(ROOT, 'exempla');
const OUTPUT = join(ROOT, 'opus', 'exempla');
const FABER = join(ROOT, 'opus', 'faber');

type Target = 'ts' | 'zig' | 'py';

/**
 * Recursively find all .fab files in a directory
 */
async function findFabFiles(dir: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        const fullPath = join(dir, entry);
        const stat = statSync(fullPath);

        if (stat.isDirectory()) {
            const subFiles = await findFabFiles(fullPath);
            files.push(...subFiles);
        }
        else if (entry.endsWith('.fab')) {
            files.push(fullPath);
        }
    }

    return files;
}

async function main() {
    const args = process.argv.slice(2);
    const targetArg = args.includes('-t') ? args[args.indexOf('-t') + 1] : 'ts';

    const targets: Target[] = targetArg === 'all' ? ['ts', 'zig', 'py'] : [targetArg as Target];

    // Always rebuild to avoid stale executable issues
    console.log('Building faber executable...');
    await $`${ROOT}/scripta/build`;

    // Find all .fab files recursively
    const files = await findFabFiles(EXEMPLA);

    console.log(`Found ${files.length} .fab files in exempla/`);

    let failed = 0;

    for (const file of files) {
        const relativePath = relative(EXEMPLA, file);
        const name = basename(file, '.fab');
        const subdir = dirname(relativePath);

        for (const target of targets) {
            const ext = target === 'ts' ? 'ts' : target === 'py' ? 'py' : 'zig';
            const outputDir = join(OUTPUT, target, subdir);
            const output = join(outputDir, `${name}.${ext}`);

            // Create output directory if needed
            await mkdir(outputDir, { recursive: true });

            try {
                const result = await $`${FABER} compile ${file} -t ${target}`.quiet();
                await Bun.write(output, result.stdout);
                console.log(`  ${relativePath} -> opus/exempla/${target}/${subdir}/${name}.${ext}`);
            }
            catch (err: any) {
                console.error(`  ${relativePath} [${target}] FAILED`);
                if (err.stderr) console.error(err.stderr.toString());
                failed++;
            }
        }
    }

    if (failed > 0) {
        console.log(`\n${failed} compilation(s) failed.`);
        process.exit(1);
    }

    // Lint TypeScript output
    if (targets.includes('ts')) {
        console.log('Linting TypeScript...');
        try {
            await $`npx eslint ${join(OUTPUT, 'ts')}/**/*.ts`.quiet();
            console.log('  eslint: OK');
        }
        catch (err: any) {
            console.error('  eslint: FAILED');
            console.error(err.stdout?.toString() || err.stderr?.toString());
            process.exit(1);
        }
    }

    // Compile Zig output
    if (targets.includes('zig')) {
        console.log('Compiling Zig...');
        const zigFiles = await findFabFiles(join(OUTPUT, 'zig'));
        const zigSources = zigFiles.filter(f => f.endsWith('.zig'));
        let zigFailed = 0;

        for (const file of zigSources) {
            const name = basename(file, '.zig');
            const output = join(dirname(file), name);

            try {
                await $`zig build-exe ${file} -femit-bin=${output}`.quiet();
                console.log(`  ${relative(OUTPUT, file)}: OK`);
            }
            catch (err: any) {
                console.error(`  ${relative(OUTPUT, file)}: FAILED`);
                const errText = err.stderr?.toString() || '';
                // Show first error only
                const firstError = errText.split('\n').slice(0, 3).join('\n');
                if (firstError) console.error(`    ${firstError}`);
                zigFailed++;
            }
        }

        if (zigFailed > 0) {
            console.log(`\n${zigFailed}/${zigSources.length} Zig file(s) failed to compile.`);
        }
    }

    // Verify Python output
    if (targets.includes('py')) {
        console.log('Verifying Python...');
        const pyDir = join(OUTPUT, 'py');

        // Find all .py files
        const findPyFiles = async (dir: string): Promise<string[]> => {
            const entries = await readdir(dir);
            const files: string[] = [];
            for (const entry of entries) {
                const fullPath = join(dir, entry);
                const stat = statSync(fullPath);
                if (stat.isDirectory()) {
                    files.push(...await findPyFiles(fullPath));
                }
                else if (entry.endsWith('.py')) {
                    files.push(fullPath);
                }
            }
            return files;
        };

        const pyFiles = await findPyFiles(pyDir);
        let pyFailed = 0;

        for (const file of pyFiles) {
            try {
                await $`python3 -m py_compile ${file}`.quiet();
            }
            catch (err: any) {
                console.error(`  ${relative(OUTPUT, file)}: FAILED`);
                const errText = err.stderr?.toString() || '';
                if (errText) console.error(`    ${errText.trim()}`);
                pyFailed++;
            }
        }

        if (pyFailed === 0) {
            console.log(`  py_compile: OK (${pyFiles.length} files)`);
        }
        else {
            console.log(`\n${pyFailed}/${pyFiles.length} Python file(s) failed syntax check.`);
            process.exit(1);
        }
    }

    console.log('Done.');
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
