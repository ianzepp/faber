#!/usr/bin/env bun
/**
 * Artifex test: Use rivus to compile itself
 *
 * Uses the compiled rivus executable (opus/bin/rivus) to compile
 * the rivus source files in fons/rivus/ and compare against
 * the faber-compiled output.
 *
 * This proves rivus can self-host.
 *
 * Usage:
 *   bun scripta/build-artifex.ts
 */

import { Glob } from 'bun';
import { mkdir } from 'fs/promises';
import { dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');
const RIVUS_BIN = join(ROOT, 'opus', 'bin', 'rivus');
const ARTIFEX_DIR = join(ROOT, 'opus', 'artifex', 'fons', 'ts');
const REFERENCE_DIR = join(ROOT, 'opus', 'rivus', 'fons', 'ts');

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

    // Check rivus binary exists
    if (!await Bun.file(RIVUS_BIN).exists()) {
        console.error('Error: rivus binary not found. Run `bun run build:rivus` first.');
        process.exit(1);
    }

    console.log('Artifex test: compiling rivus with rivus\n');

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

    console.log(`OK (${succeeded} files, ${compileElapsed.toFixed(0)}ms)\n`);

    // Compare with faber-compiled output
    process.stdout.write('Comparing with reference... ');
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

    const elapsed = performance.now() - start;
    console.log(`\nArtifex test passed! rivus successfully compiled itself (${(elapsed / 1000).toFixed(1)}s)`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
