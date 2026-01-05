#!/usr/bin/env bun
/**
 * Full bootstrap build chain: faber -> rivus -> exempla
 *
 * 1. Build opus/bin/faber from fons/faber/cli.ts
 * 2. Use opus/bin/faber to compile fons/rivus/ -> opus/bin/rivus
 * 3. Use opus/bin/rivus to compile fons/exempla/ -> opus/exempla/
 *
 * This validates that the compiled faber can build rivus, and the compiled
 * rivus can build real programs.
 */

import { Glob } from 'bun';
import { mkdir, readdir, stat } from 'fs/promises';
import { basename, dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');

const FABER_BIN = join(ROOT, 'opus', 'bin', 'faber');
const RIVUS_BIN = join(ROOT, 'opus', 'bin', 'rivus');
const RIVUS_SOURCE = join(ROOT, 'fons', 'rivus');
const RIVUS_OUTPUT = join(ROOT, 'opus', 'rivus', 'fons', 'ts');
const EXEMPLA_SOURCE = join(ROOT, 'fons', 'exempla');
const EXEMPLA_OUTPUT = join(ROOT, 'opus', 'exempla');

async function step(name: string, fn: () => Promise<void>) {
    const start = performance.now();
    process.stdout.write(`${name}... `);
    await fn();
    const elapsed = performance.now() - start;
    console.log(`OK (${elapsed.toFixed(0)}ms)`);
}

async function buildFaber() {
    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });
    await $`bun build ${join(ROOT, 'fons', 'faber', 'cli.ts')} --compile --outfile=${FABER_BIN}`.quiet();
    await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();
}

async function compileRivusSource() {
    const glob = new Glob('**/*.fab');
    const files: string[] = [];
    for await (const file of glob.scan({ cwd: RIVUS_SOURCE, absolute: true })) {
        files.push(file);
    }

    let failed = 0;
    const results = await Promise.all(
        files.map(async fabPath => {
            const relPath = relative(RIVUS_SOURCE, fabPath);
            const outPath = join(RIVUS_OUTPUT, relPath.replace(/\.fab$/, '.ts'));

            try {
                await mkdir(dirname(outPath), { recursive: true });
                const result = await $`${FABER_BIN} compile ${fabPath} -t ts`.quiet();
                await Bun.write(outPath, result.stdout);
                return { file: relPath, success: true };
            }
            catch (err: any) {
                const msg = err.stderr?.toString().trim() || String(err);
                console.error(`\n  ${relPath}: ${msg}`);
                return { file: relPath, success: false };
            }
        })
    );

    failed = results.filter(r => !r.success).length;
    if (failed > 0) {
        throw new Error(`${failed}/${files.length} files failed to compile`);
    }

    return files.length;
}

async function typeCheckRivus() {
    await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(RIVUS_OUTPUT, 'cli.ts')}`.quiet();
}

async function buildRivusBinary() {
    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });
    await $`bun build ${join(RIVUS_OUTPUT, 'cli.ts')} --compile --outfile=${RIVUS_BIN}`.quiet();
    await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();
}

async function findFabFiles(dir: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        const fullPath = join(dir, entry);
        const s = await stat(fullPath);
        if (s.isDirectory()) {
            files.push(...await findFabFiles(fullPath));
        }
        else if (entry.endsWith('.fab')) {
            files.push(fullPath);
        }
    }

    return files;
}

async function compileExempla() {
    const files = await findFabFiles(EXEMPLA_SOURCE);
    let failed = 0;

    // WHY: rivus CLI reads from stdin, not file arguments
    const results = await Promise.all(
        files.map(async fabPath => {
            const relPath = relative(EXEMPLA_SOURCE, fabPath);
            const name = basename(fabPath, '.fab');
            const subdir = dirname(relPath);
            const outDir = join(EXEMPLA_OUTPUT, 'ts', subdir);
            const outPath = join(outDir, `${name}.ts`);

            try {
                await mkdir(outDir, { recursive: true });
                const source = await Bun.file(fabPath).text();
                const proc = Bun.spawn([RIVUS_BIN], {
                    stdin: new Response(source).body,
                    stdout: 'pipe',
                    stderr: 'pipe',
                });
                const [stdout, stderr] = await Promise.all([
                    new Response(proc.stdout).text(),
                    new Response(proc.stderr).text(),
                ]);
                const exitCode = await proc.exited;

                if (exitCode !== 0 || stderr.trim()) {
                    throw new Error(stderr.trim() || `exit code ${exitCode}`);
                }

                await Bun.write(outPath, stdout);
                return { file: relPath, success: true };
            }
            catch (err: any) {
                const msg = err.message || String(err);
                console.error(`\n  ${relPath}: ${msg}`);
                return { file: relPath, success: false };
            }
        })
    );

    failed = results.filter(r => !r.success).length;
    if (failed > 0) {
        throw new Error(`${failed}/${files.length} exempla failed to compile`);
    }

    return files.length;
}

async function verifyExempla() {
    const findTsFiles = async (dir: string): Promise<string[]> => {
        const entries = await readdir(dir);
        const files: string[] = [];
        for (const entry of entries) {
            const fullPath = join(dir, entry);
            const s = await stat(fullPath);
            if (s.isDirectory()) {
                files.push(...await findTsFiles(fullPath));
            }
            else if (entry.endsWith('.ts')) {
                files.push(fullPath);
            }
        }
        return files;
    };

    const tsDir = join(EXEMPLA_OUTPUT, 'ts');
    const tsFiles = await findTsFiles(tsDir);
    let failed = 0;

    for (const file of tsFiles) {
        try {
            await $`bun build --no-bundle ${file}`.quiet();
        }
        catch {
            console.error(`\n  ${relative(EXEMPLA_OUTPUT, file)}: type error`);
            failed++;
        }
    }

    if (failed > 0) {
        throw new Error(`${failed}/${tsFiles.length} TypeScript files failed verification`);
    }

    return tsFiles.length;
}

async function main() {
    const totalStart = performance.now();
    let rivusFileCount = 0;
    let exemplaFileCount = 0;

    console.log('Building bootstrap chain: faber -> rivus -> exempla\n');

    await step('Building opus/bin/faber', buildFaber);

    await step('Compiling fons/rivus/ with faber', async () => {
        rivusFileCount = await compileRivusSource();
    });

    await step('Type-checking rivus output', typeCheckRivus);

    await step('Building opus/bin/rivus', buildRivusBinary);

    await step('Compiling fons/exempla/ with rivus', async () => {
        exemplaFileCount = await compileExempla();
    });

    await step('Verifying exempla TypeScript output', verifyExempla);

    const totalElapsed = performance.now() - totalStart;
    console.log(`\nBootstrap complete: ${rivusFileCount} rivus + ${exemplaFileCount} exempla files (${(totalElapsed / 1000).toFixed(1)}s)`);
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
