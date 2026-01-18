#!/usr/bin/env bun
/**
 * Build artifex: Use rivus to compile itself into a new executable
 *
 * Uses the compiled rivus executable (opus/bin/rivus) to compile
 * the rivus source files in fons/rivus/, then builds opus/bin/artifex.
 *
 * This proves rivus can self-host and produces a working compiler.
 *
 * Usage:
 *   bun scripta/build-artifex.ts                # Build artifex executable
 *   bun scripta/build-artifex.ts --verify-diff  # Also compare output with faber
 *   bun scripta/build-artifex.ts --no-typecheck # Skip TypeScript type checking
 */

const SKIP_TYPECHECK = process.argv.includes('--no-typecheck');

import { Glob } from 'bun';
import { mkdir, symlink, unlink } from 'fs/promises';
import { dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');
const RIVUS_BIN = join(ROOT, 'opus', 'bin', 'rivus');
const ARTIFEX_DIR = join(ROOT, 'opus', 'artifex-ts', 'fons');
const REFERENCE_DIR = join(ROOT, 'opus', 'rivus-ts', 'fons');

interface CompileResult {
    file: string;
    success: boolean;
    error?: string;
}

async function compileFile(fabPath: string): Promise<CompileResult> {
    const relPath = relative(SOURCE, fabPath);
    const outPath = join(ARTIFEX_DIR, relPath.replace(/\.fab$/, '.ts'));

    try {
        const source = await Bun.file(fabPath).text();

        // rivus expects: first line = file path, rest = source
        const input = `${fabPath}\n${source}`;

        // Run rivus
        const proc = Bun.spawn([RIVUS_BIN], {
            stdin: 'pipe',
            stdout: 'pipe',
            stderr: 'pipe',
        });

        proc.stdin.write(input);
        proc.stdin.end();

        const exitCode = await proc.exited;
        const stderr = await new Response(proc.stderr).text();

        if (exitCode !== 0 || stderr.trim()) {
            throw new Error(stderr || `Exit code ${exitCode}`);
        }

        const output = await new Response(proc.stdout).text();

        await mkdir(dirname(outPath), { recursive: true });
        await Bun.write(outPath, output);

        return { file: relPath, success: true };
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        return { file: relPath, success: false, error: message };
    }
}

async function injectExternImpls(): Promise<void> {
    const modulusPath = join(ARTIFEX_DIR, 'semantic', 'modulus.ts');
    let modulusContent = await Bun.file(modulusPath).text();

    const externImpls = `
// FILE I/O IMPLEMENTATIONS (injected by build-artifex.ts)
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
        await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(ARTIFEX_DIR, 'cli.ts')}`.quiet();
        return true;
    }
    catch {
        return false;
    }
}

async function buildExecutable(): Promise<void> {
    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });
    const outExe = join(binDir, 'artifex-ts');
    await $`bun build ${join(ARTIFEX_DIR, 'cli.ts')} --compile --outfile=${outExe}`.quiet();
    await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();

    // Create backward-compat symlink: artifex -> artifex-ts
    const symlinkPath = join(binDir, 'artifex');
    try { await unlink(symlinkPath); } catch { /* ignore */ }
    await symlink('artifex-ts', symlinkPath);
}

async function compareFiles(file: string): Promise<{ match: boolean; diff?: string }> {
    const artifexPath = join(ARTIFEX_DIR, file.replace(/\.fab$/, '.ts'));
    const referencePath = join(REFERENCE_DIR, file.replace(/\.fab$/, '.ts'));

    try {
        const artifexContent = await Bun.file(artifexPath).text();
        const referenceContent = await Bun.file(referencePath).text();

        if (artifexContent === referenceContent) {
            return { match: true };
        }

        // Files differ - show diff
        const diffProc = Bun.spawn(['diff', '-u', referencePath, artifexPath], {
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

    console.log('Building artifex: rivus compiling itself\n');

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

    if (failed.length > 0) {
        console.log(`FAILED (${failed.length}/${results.length})\n`);
        for (const f of failed) {
            console.error(`  ${f.file}: ${f.error}`);
        }
        process.exit(1);
    }

    console.log(`OK (${succeeded} files, ${compileElapsed.toFixed(0)}ms)`);

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
            await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(ARTIFEX_DIR, 'cli.ts')}`;
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
        // Compare with faber-compiled output
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
    console.log(`\nArtifex built: ${succeeded} files compiled${verified} -> opus/bin/artifex-ts (${(elapsed / 1000).toFixed(1)}s)`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
