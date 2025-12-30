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
const FABER = join(ROOT, 'fons', 'cli.ts');

type Target = 'ts' | 'zig' | 'py' | 'rs';

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
        } else if (entry.endsWith('.fab')) {
            files.push(fullPath);
        }
    }

    return files;
}

async function main() {
    const args = process.argv.slice(2);
    const targetArg = args.includes('-t') ? args[args.indexOf('-t') + 1] : 'ts';

    const targets: Target[] = targetArg === 'all' ? ['ts', 'zig', 'py', 'rs'] : [targetArg as Target];

    // WHY: Use unbundled CLI directly — bundled executable can't load preamble .txt files

    // Find all .fab files recursively
    const files = await findFabFiles(EXEMPLA);

    console.log(`Found ${files.length} .fab files in exempla/`);

    let failed = 0;

    for (const file of files) {
        const relativePath = relative(EXEMPLA, file);
        const name = basename(file, '.fab');
        const subdir = dirname(relativePath);

        for (const target of targets) {
            const ext = { ts: 'ts', py: 'py', zig: 'zig', rs: 'rs' }[target];
            const outputDir = join(OUTPUT, target, subdir);
            const output = join(outputDir, `${name}.${ext}`);

            // Create output directory if needed
            await mkdir(outputDir, { recursive: true });

            try {
                const result = await $`bun ${FABER} compile ${file} -t ${target}`.quiet();
                await Bun.write(output, result.stdout);
                console.log(`  ${relativePath} -> opus/exempla/${target}/${subdir}/${name}.${ext}`);
            } catch (err: any) {
                console.error(`  ${relativePath} [${target}] FAILED`);
                if (err.stderr) console.error(err.stderr.toString());
                failed++;
            }
        }
    }

    if (failed > 0) {
        console.log(`\n${failed} parser/codegen failure(s) - continuing to verify generated files...`);
    }

    // Verify TypeScript output compiles
    if (targets.includes('ts')) {
        console.log('Verifying TypeScript...');
        const tsDir = join(OUTPUT, 'ts');

        // Find all .ts files
        const findTsFiles = async (dir: string): Promise<string[]> => {
            const entries = await readdir(dir);
            const files: string[] = [];
            for (const entry of entries) {
                const fullPath = join(dir, entry);
                const stat = statSync(fullPath);
                if (stat.isDirectory()) {
                    files.push(...(await findTsFiles(fullPath)));
                } else if (entry.endsWith('.ts')) {
                    files.push(fullPath);
                }
            }
            return files;
        };

        const tsFiles = await findTsFiles(tsDir);
        let tsFailed = 0;

        for (const file of tsFiles) {
            try {
                // WHY: --no-bundle just type-checks without bundling
                await $`bun build --no-bundle ${file}`.quiet();
            } catch (err: any) {
                console.error(`  ${relative(OUTPUT, file)}: FAILED`);
                const errText = err.stderr?.toString() || '';
                const firstError = errText.split('\n').slice(0, 3).join('\n');
                if (firstError) console.error(`    ${firstError}`);
                tsFailed++;
            }
        }

        if (tsFailed === 0) {
            console.log(`  bun build: OK (${tsFiles.length} files)`);
        } else {
            console.log(`\n${tsFailed}/${tsFiles.length} TypeScript file(s) failed to compile.`);
            process.exit(1);
        }
    }

    // Compile Zig output
    if (targets.includes('zig')) {
        console.log('Compiling Zig...');
        const findZigFiles = async (dir: string): Promise<string[]> => {
            const entries = await readdir(dir);
            const files: string[] = [];
            for (const entry of entries) {
                const fullPath = join(dir, entry);
                const stat = statSync(fullPath);
                if (stat.isDirectory()) {
                    files.push(...(await findZigFiles(fullPath)));
                } else if (entry.endsWith('.zig')) {
                    files.push(fullPath);
                }
            }
            return files;
        };
        const zigSources = await findZigFiles(join(OUTPUT, 'zig'));
        let zigFailed = 0;

        for (const file of zigSources) {
            const name = basename(file, '.zig');
            const output = join(dirname(file), name);

            try {
                await $`zig build-exe ${file} -femit-bin=${output}`.quiet();
                console.log(`  ${relative(OUTPUT, file)}: OK`);
            } catch (err: any) {
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
                    files.push(...(await findPyFiles(fullPath)));
                } else if (entry.endsWith('.py')) {
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
            } catch (err: any) {
                console.error(`  ${relative(OUTPUT, file)}: FAILED`);
                const errText = err.stderr?.toString() || '';
                if (errText) console.error(`    ${errText.trim()}`);
                pyFailed++;
            }
        }

        if (pyFailed === 0) {
            console.log(`  py_compile: OK (${pyFiles.length} files)`);
        } else {
            console.log(`\n${pyFailed}/${pyFiles.length} Python file(s) failed syntax check.`);
            process.exit(1);
        }
    }

    // Verify Rust output
    if (targets.includes('rs')) {
        console.log('Verifying Rust...');
        const rsDir = join(OUTPUT, 'rs');

        // Find all .rs files
        const findRsFiles = async (dir: string): Promise<string[]> => {
            const entries = await readdir(dir);
            const files: string[] = [];
            for (const entry of entries) {
                const fullPath = join(dir, entry);
                const stat = statSync(fullPath);
                if (stat.isDirectory()) {
                    files.push(...(await findRsFiles(fullPath)));
                } else if (entry.endsWith('.rs')) {
                    files.push(fullPath);
                }
            }
            return files;
        };

        const rsFiles = await findRsFiles(rsDir);
        let rsFailed = 0;

        for (const file of rsFiles) {
            try {
                // WHY: --emit=metadata only checks syntax, doesn't generate code
                // WHY: --edition=2021 matches Faber's target Rust edition
                await $`rustc --emit=metadata --edition=2021 -o /dev/null ${file}`.quiet();
                console.log(`  ${relative(OUTPUT, file)}: OK`);
            } catch (err: any) {
                console.error(`  ${relative(OUTPUT, file)}: FAILED`);
                const errText = err.stderr?.toString() || '';
                // Show first error only
                const firstError = errText.split('\n').slice(0, 5).join('\n');
                if (firstError) console.error(`    ${firstError}`);
                rsFailed++;
            }
        }

        if (rsFailed === 0) {
            console.log(`  rustc: OK (${rsFiles.length} files)`);
        } else {
            console.log(`\n${rsFailed}/${rsFiles.length} Rust file(s) failed syntax check.`);
        }
    }

    console.log('Done.');
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
