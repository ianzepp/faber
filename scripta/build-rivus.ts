#!/usr/bin/env bun
/**
 * Build rivus (bootstrap compiler) from fons/rivus/ using faber or nanus.
 *
 * Uses the specified compiler to compile all .fab files in parallel.
 * Output is TypeScript (all compilers emit TS).
 *
 * Requires: bun run build:faber (or build:nanus-ts, build:nanus-go) first
 *
 * Usage:
 *   bun scripta/build-rivus.ts
 *   bun scripta/build-rivus.ts -c nanus-ts
 *   bun scripta/build-rivus.ts -c nanus-go
 *   bun scripta/build-rivus.ts -c faber --no-typecheck
 */

import { Glob } from 'bun';
import { mkdir, symlink, unlink } from 'fs/promises';
import { dirname, join, relative } from 'path';
import { $ } from 'bun';

type Compiler = 'faber' | 'nanus' | 'nanus-ts' | 'nanus-go';
const VALID_COMPILERS: Compiler[] = ['faber', 'nanus', 'nanus-ts', 'nanus-go'];

// Parse arguments
let compiler: Compiler = 'faber';
let skipTypecheck = false;

const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '-c' || arg === '--compiler') {
        const c = args[++i] as Compiler;
        if (!VALID_COMPILERS.includes(c)) {
            console.error(`Unknown compiler '${c}'. Valid: ${VALID_COMPILERS.join(', ')}`);
            process.exit(1);
        }
        compiler = c;
    } else if (arg === '--no-typecheck') {
        skipTypecheck = true;
    }
}

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');
const OUTPUT = join(ROOT, 'opus', 'rivus-ts', 'fons');
const COMPILER_BIN = join(ROOT, 'opus', 'bin', compiler);

// nanus-ts and nanus-go use stdin/stdout, faber/nanus use file args
const useStdinStdout = compiler === 'nanus-ts' || compiler === 'nanus-go';

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

        if (useStdinStdout) {
            // nanus-ts/nanus-go: cat file | compiler emit > output
            const result = await $`cat ${fabPath} | ${COMPILER_BIN} emit`.nothrow().quiet();
            if (result.exitCode !== 0) {
                const stderr = result.stderr.toString().trim();
                throw new Error(stderr || `Exit code ${result.exitCode}`);
            }
            await Bun.write(outPath, result.stdout);
        } else {
            // faber/nanus: compiler compile file -o output
            const result = await $`${COMPILER_BIN} compile ${fabPath} -o ${outPath}`.nothrow().quiet();
            if (result.exitCode !== 0) {
                const stderr = result.stderr.toString().trim();
                throw new Error(stderr || `Exit code ${result.exitCode}`);
            }
        }

        return { file: relPath, success: true };
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        return { file: relPath, success: false, error: message };
    }
}

async function typeCheck(): Promise<boolean> {
    const result = await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(OUTPUT, 'rivus.ts')}`.nothrow();
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
    // to opus/rivus-ts/norma/hal (not opus/rivus-ts/fons/norma/hal)
    const halDest = join(ROOT, 'opus', 'rivus-ts', 'norma', 'hal');
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
    const outExe = join(binDir, 'rivus-ts');
    await $`bun build ${join(OUTPUT, 'rivus.ts')} --compile --outfile=${outExe}`.quiet();
    await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();

    // Create backward-compat symlink: rivus -> rivus-ts
    const symlinkPath = join(binDir, 'rivus');
    try { await unlink(symlinkPath); } catch { /* ignore */ }
    await symlink('rivus-ts', symlinkPath);
}

async function main() {
    const start = performance.now();

    // Check compiler binary exists
    if (!await Bun.file(COMPILER_BIN).exists()) {
        console.error(`Error: ${compiler} binary not found at opus/bin/${compiler}`);
        console.error(`Run \`bun run build:${compiler}\` first.`);
        process.exit(1);
    }

    console.log(`Using compiler: ${compiler}`);

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
    if (!skipTypecheck) {
        console.log('Type checking...');
        const tcOk = await typeCheck();
        if (!tcOk) {
            console.error('TypeScript type check failed');
            process.exit(1);
        }
        console.log('Type check passed');
    }

    await injectExternImpls();

    console.log('Building rivus executable...');
    await buildExecutable();
    console.log('Built opus/bin/rivus-ts');
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
