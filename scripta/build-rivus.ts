#!/usr/bin/env bun
/**
 * Build rivus (bootstrap compiler) from fons/rivus/ using faber.
 *
 * Uses opus/bin/faber to compile all .fab files in parallel.
 * Output is TypeScript (faber is TS-only; for other targets, use rivus).
 *
 * Requires: bun run build --no-rivus (to build faber first)
 *
 * Usage:
 *   bun scripta/build-rivus.ts
 */

import { Glob } from 'bun';
import { mkdir } from 'fs/promises';
import { dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');
const OUTPUT = join(ROOT, 'opus', 'rivus', 'fons', 'ts');
const FABER_BIN = join(ROOT, 'opus', 'bin', 'faber');

interface CompileResult {
    file: string;
    success: boolean;
    error?: string;
}

async function compileFile(fabPath: string): Promise<CompileResult> {
    const relPath = relative(SOURCE, fabPath);
    const outPath = join(OUTPUT, relPath.replace(/\.fab$/, '.ts'));

    try {
        await mkdir(dirname(outPath), { recursive: true });

        const result = await $`${FABER_BIN} compile ${fabPath} -o ${outPath}`.nothrow().quiet();

        if (result.exitCode !== 0) {
            const stderr = result.stderr.toString().trim();
            throw new Error(stderr || `Exit code ${result.exitCode}`);
        }

        return { file: relPath, success: true };
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        return { file: relPath, success: false, error: message };
    }
}

async function typeCheck(): Promise<boolean> {
    const result = await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(OUTPUT, 'cli.ts')}`.nothrow();
    if (result.exitCode !== 0) {
        console.error(result.stdout.toString());
        return false;
    }
    return true;
}

async function injectExternImpls(): Promise<void> {
    const modulusPath = join(OUTPUT, 'semantic', 'modulus.ts');
    let modulusContent = await Bun.file(modulusPath).text();

    const externImpls = `
// FILE I/O IMPLEMENTATIONS (injected by build-rivus.ts)
import { readFileSync, existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
const _readFileSync = (via: string): string => readFileSync(via, 'utf-8');
const _existsSync = (via: string): boolean => existsSync(via);
const _dirname = (via: string): string => dirname(via);
const _resolve = (basis: string, relativum: string): string => resolve(basis, relativum);
`;

    modulusContent = modulusContent.replace(
        /declare function _readFileSync.*?;\ndeclare function _existsSync.*?;\ndeclare function _dirname.*?;\ndeclare function _resolve.*?;/s,
        externImpls.trim(),
    );

    await Bun.write(modulusPath, modulusContent);
}

async function copyHalImplementations(): Promise<void> {
    const halSource = join(ROOT, 'fons', 'norma', 'hal', 'codegen', 'ts');
    // WHY: Imports use ../../../norma/hal from cli/commands/, which resolves
    // to opus/rivus/fons/norma/hal (not opus/rivus/fons/ts/norma/hal)
    const halDest = join(ROOT, 'opus', 'rivus', 'fons', 'norma', 'hal');
    await mkdir(halDest, { recursive: true });

    const glob = new Glob('*.ts');
    for await (const file of glob.scan({ cwd: halSource, absolute: false })) {
        if (file.endsWith('.test.ts')) continue;
        const src = join(halSource, file);
        const dest = join(halDest, file);
        await Bun.write(dest, await Bun.file(src).text());
    }
}

async function buildExecutable(): Promise<void> {
    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });
    const outExe = join(binDir, 'rivus');
    await $`bun build ${join(OUTPUT, 'cli.ts')} --compile --outfile=${outExe}`.quiet();
    await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();
}

async function main() {
    const start = performance.now();

    // Check faber binary exists
    if (!await Bun.file(FABER_BIN).exists()) {
        console.error('Error: faber binary not found at opus/bin/faber');
        console.error('Run `bun run build --no-rivus` first to build faber.');
        process.exit(1);
    }

    // Find all .fab files
    const glob = new Glob('**/*.fab');
    const files: string[] = [];
    for await (const file of glob.scan({ cwd: SOURCE, absolute: true })) {
        files.push(file);
    }

    // Compile all in parallel
    const results = await Promise.all(files.map(f => compileFile(f)));

    const elapsed = performance.now() - start;
    const succeeded = results.filter(r => r.success).length;
    const failed = results.filter(r => !r.success);

    // Report failures
    for (const f of failed) {
        console.error(`${f.file}: ${f.error}`);
    }

    // Summary
    const relOut = relative(ROOT, OUTPUT);
    console.log(`Compiled ${succeeded}/${results.length} files to ${relOut}/ (${elapsed.toFixed(0)}ms)`);

    if (failed.length > 0) {
        process.exit(1);
    }

    // Copy HAL native implementations
    await copyHalImplementations();

    // Type check (TypeScript only)
    console.log('Type checking...');
    const tcOk = await typeCheck();
    if (!tcOk) {
        console.error('TypeScript type check failed');
        process.exit(1);
    }
    console.log('Type check passed');

    await injectExternImpls();

    console.log('Building rivus executable...');
    await buildExecutable();
    console.log('Built opus/bin/rivus');
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
