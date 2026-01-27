#!/usr/bin/env bun
/**
 * Build faber: Use rivus to compile itself into a new executable
 *
 * Uses the compiled rivus executable (opus/bin/rivus) to compile
 * the rivus source files in fons/rivus/, then builds opus/bin/faber-ts.
 *
 * This proves rivus can self-host and produces a working compiler.
 *
 * Usage:
 *   bun scripta/build-faber.ts                # Build faber executable
 *   bun scripta/build-faber.ts --verify-diff  # Also compare output with rivus-ts
 *   bun scripta/build-faber.ts --no-typecheck # Skip TypeScript type checking
 */

const SKIP_TYPECHECK = process.argv.includes('--no-typecheck');

import { Glob } from 'bun';
import { mkdir, symlink, unlink } from 'fs/promises';
import { dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');
const RIVUS_BIN = join(ROOT, 'opus', 'bin', 'rivus');
const FABER_DIR = join(ROOT, 'opus', 'faber-ts', 'fons');
const REFERENCE_DIR = join(ROOT, 'opus', 'rivus-ts', 'fons');

interface CompileResult {
    file: string;
    success: boolean;
    warnings?: string;
    error?: string;
}

async function compileFile(fabPath: string): Promise<CompileResult> {
    const relPath = relative(SOURCE, fabPath);
    const outPath = join(FABER_DIR, relPath.replace(/\.fab$/, '.ts'));

    try {
        await mkdir(dirname(outPath), { recursive: true });

        const proc = Bun.spawn([RIVUS_BIN, 'emit', '--input', fabPath, '-o', outPath, '--strip-tests'], {
            stdout: 'pipe',
            stderr: 'pipe',
        });

        const exitCode = await proc.exited;
        const stderr = await new Response(proc.stderr).text();

        if (exitCode !== 0) {
            return { file: relPath, success: false, error: stderr.trim() || `Exit code ${exitCode}` };
        }

        // Success - but may have warnings
        return { file: relPath, success: true, warnings: stderr.trim() || undefined };
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        return { file: relPath, success: false, error: message };
    }
}

async function injectExternImpls(): Promise<void> {
    const modulusPath = join(FABER_DIR, 'semantic', 'modulus.ts');
    let modulusContent = await Bun.file(modulusPath).text();

    const externImpls = `
// FILE I/O IMPLEMENTATIONS (injected by build-faber.ts)
import { readFileSync, existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
const _readFileSync = (via: string): string => readFileSync(via, 'utf-8');
const _existsSync = (via: string): boolean => existsSync(via);
const _dirname = (via: string): string => dirname(via);
const _resolve = (basis: string, relativum: string): string => resolve(basis, relativum);
`;

    modulusContent = modulusContent.replace(
        /declare function _readFileSync.*?;\ndeclare function _existsSync.*?;\ndeclare function _dirname.*?;\ndeclare function _resolve.*?;/s,
        externImpls.trim()
    );

    await Bun.write(modulusPath, modulusContent);
}

async function typeCheck(): Promise<boolean> {
    try {
        await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(FABER_DIR, 'cli.ts')}`.quiet();
        return true;
    }
    catch {
        return false;
    }
}

async function copyCliShim(): Promise<void> {
    const shimSource = join(ROOT, 'fons', 'rivus-cli', 'ts.ts');
    const shimDest = join(FABER_DIR, 'cli.ts');
    await Bun.write(shimDest, await Bun.file(shimSource).text());
}

async function copyNorma(): Promise<boolean> {
    const normaSource = join(ROOT, 'fons', 'norma-ts');

    try {
        const { readdirSync } = await import('node:fs');
        readdirSync(normaSource);
    } catch {
        return false;
    }

    const normaDest = join(dirname(FABER_DIR), 'norma');
    await mkdir(normaDest, { recursive: true });

    const glob = new Glob('**/*.ts');
    for await (const file of glob.scan({ cwd: normaSource, absolute: false })) {
        if (file.includes('.test.')) { continue; }
        const src = join(normaSource, file);
        const dest = join(normaDest, file);
        await mkdir(dirname(dest), { recursive: true });
        await Bun.write(dest, await Bun.file(src).text());
    }
    return true;
}

async function buildExecutable(): Promise<void> {
    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });
    const outExe = join(binDir, 'faber-ts');
    await $`bun build ${join(FABER_DIR, 'cli.ts')} --compile --outfile=${outExe}`.quiet();
    await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();

    // Create symlink: faber -> faber-ts
    const symlinkPath = join(binDir, 'faber');
    try { await unlink(symlinkPath); } catch { /* ignore */ }
    await symlink('faber-ts', symlinkPath);
}

async function compareFiles(file: string): Promise<{ match: boolean; diff?: string }> {
    const faberPath = join(FABER_DIR, file.replace(/\.fab$/, '.ts'));
    const referencePath = join(REFERENCE_DIR, file.replace(/\.fab$/, '.ts'));

    try {
        const faberContent = await Bun.file(faberPath).text();
        const referenceContent = await Bun.file(referencePath).text();

        if (faberContent === referenceContent) {
            return { match: true };
        }

        // Files differ - show diff
        const diffProc = Bun.spawn(['diff', '-u', referencePath, faberPath], {
            stdout: 'pipe',
        });
        const diff = await new Response(diffProc.stdout).text();

        return { match: false, diff };
    }
    catch (err) {
        return { match: false, diff: String(err) };
    }
}

async function main() {
    const start = performance.now();
    const verifyDiff = process.argv.includes('--verify-diff');

    // Check rivus binary exists
    if (!await Bun.file(RIVUS_BIN).exists()) {
        console.error('Error: rivus binary not found. Run `bun run build:rivus` first.');
        process.exit(1);
    }

    console.log('Building faber: rivus compiling itself\n');

    // Find all .fab files
    const glob = new Glob('**/*.fab');
    const files: string[] = [];
    for await (const file of glob.scan({ cwd: SOURCE, absolute: true })) {
        files.push(file);
    }

    // Compile all files
    process.stdout.write('Compiling... ');
    const compileStart = performance.now();
    const results = await Promise.all(files.map(f => compileFile(f)));
    const compileElapsed = performance.now() - compileStart;

    const succeeded = results.filter(r => r.success).length;
    const failed = results.filter(r => !r.success);
    const withWarnings = results.filter(r => r.success && r.warnings);

    if (failed.length > 0) {
        console.log(`FAILED (${failed.length}/${results.length})\n`);
        for (const f of failed) {
            console.error(`  ${f.file}:\n    ${f.error?.split('\n').join('\n    ')}`);
        }
        process.exit(1);
    }

    const warnSuffix = withWarnings.length > 0 ? `, ${withWarnings.length} with warnings` : '';
    console.log(`OK (${succeeded} files${warnSuffix}, ${compileElapsed.toFixed(0)}ms)`);

    // Copy CLI shim
    await copyCliShim();

    // Copy norma HAL implementations
    await copyNorma();

    // Inject extern implementations
    await injectExternImpls();

    // Type-check
    if (!SKIP_TYPECHECK) {
        process.stdout.write('Type-checking... ');
        const tcStart = performance.now();
        const tcOk = await typeCheck();
        const tcElapsed = performance.now() - tcStart;

        if (tcOk) {
            console.log(`OK (${tcElapsed.toFixed(0)}ms)`);
        }
        else {
            console.log('FAILED');
            await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(FABER_DIR, 'cli.ts')}`;
            process.exit(1);
        }
    }

    // Build executable
    process.stdout.write('Compiling executable... ');
    const buildStart = performance.now();
    await buildExecutable();
    const buildElapsed = performance.now() - buildStart;
    console.log(`OK (${buildElapsed.toFixed(0)}ms)`);

    if (verifyDiff) {
        // Compare with rivus-compiled output
        process.stdout.write('\nComparing with reference... ');
        const compareStart = performance.now();
        const relFiles = results.map(r => r.file);
        const comparisons = await Promise.all(relFiles.map(f => compareFiles(f)));
        const compareElapsed = performance.now() - compareStart;

        const matches = comparisons.filter(c => c.match).length;
        const diffs = comparisons.filter(c => !c.match);

        if (diffs.length > 0) {
            console.log(`FAILED (${diffs.length} differences)\n`);
            for (let i = 0; i < diffs.length; i++) {
                const file = relFiles[i];
                const diff = diffs.find((_, idx) => idx === i)?.diff;
                console.error(`\n${file}:\n${diff}`);
            }
            process.exit(1);
        }

        console.log(`OK (${matches} files match, ${compareElapsed.toFixed(0)}ms)`);
    }

    const elapsed = performance.now() - start;
    const verified = verifyDiff ? ', verified' : '';
    console.log(`\nFaber built: ${succeeded} files compiled${verified} -> opus/bin/faber-ts (${(elapsed / 1000).toFixed(1)}s)`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
