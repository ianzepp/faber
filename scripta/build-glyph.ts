#!/usr/bin/env bun
/**
 * Build glyph representation of rivus source.
 *
 * Compiles all .fab files in fons/rivus/ to Unicode glyph format.
 * Output: opus/glyph/
 *
 * Requires: bun run build (or at least rivus-ts)
 *
 * Usage:
 *   bun scripta/build-glyph.ts
 */

import { Glob } from 'bun';
import { mkdir } from 'fs/promises';
import { dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');
const OUTPUT = join(ROOT, 'opus', 'glyph');
const COMPILER = join(ROOT, 'opus', 'bin', 'rivus-ts');

interface CompileResult {
    file: string;
    success: boolean;
    error?: string;
}

async function compileFile(fabPath: string): Promise<CompileResult> {
    const relPath = relative(SOURCE, fabPath);
    const outPath = join(OUTPUT, relPath.replace(/\.fab$/, '.glyph'));

    try {
        await mkdir(dirname(outPath), { recursive: true });

        const result = await $`cat ${fabPath} | ${COMPILER} emit --target glyph --stdin-filename ${fabPath}`.nothrow().quiet();

        if (result.exitCode !== 0) {
            throw new Error(result.stderr.toString().trim() || `Exit code ${result.exitCode}`);
        }

        await Bun.write(outPath, result.stdout);
        return { file: relPath, success: true };
    } catch (err) {
        return { file: relPath, success: false, error: err instanceof Error ? err.message : String(err) };
    }
}

// Process files in batches to avoid spawning too many processes
async function processBatches<T, R>(
    items: T[],
    fn: (item: T) => Promise<R>,
    batchSize = 16
): Promise<R[]> {
    const results: R[] = [];
    for (let i = 0; i < items.length; i += batchSize) {
        const batch = items.slice(i, i + batchSize);
        const batchResults = await Promise.all(batch.map(fn));
        results.push(...batchResults);
    }
    return results;
}

async function main() {
    console.log(`Compiling to glyph: ${SOURCE} -> ${OUTPUT}\n`);

    // Discover all .fab files
    const glob = new Glob('**/*.fab');
    const files: string[] = [];
    for await (const file of glob.scan({ cwd: SOURCE, absolute: true })) {
        files.push(file);
    }

    console.log(`Found ${files.length} .fab files`);
    console.log('Compiling files...\n');

    // Compile in batches
    const start = performance.now();
    const results = await processBatches(files, compileFile, 16);
    const elapsed = Math.round(performance.now() - start);

    // Report failures
    const failures = results.filter(r => !r.success);
    if (failures.length > 0) {
        console.log('Compilation failures:');
        for (const f of failures) {
            console.log(`  ${f.file}: ${f.error}`);
        }
        console.log();
    }

    const succeeded = results.length - failures.length;
    console.log(`Compiled ${succeeded}/${results.length} files to ${OUTPUT} (${elapsed}ms)`);

    if (failures.length > 0) {
        process.exit(1);
    }
}

main();
