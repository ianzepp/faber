#!/usr/bin/env bun
/**
 * Build rivus (bootstrap compiler) from fons/rivus/ using faber.
 *
 * Compiles all .fab files in parallel without spawning processes.
 * Output is TypeScript (faber is TS-only; for other targets, use rivus).
 *
 * Usage:
 *   bun scripta/build-rivus.ts
 */

import { Glob } from 'bun';
import { mkdir } from 'fs/promises';
import { dirname, join, relative } from 'path';
import { tokenize } from '../fons/faber/tokenizer';
import { parse } from '../fons/faber/parser';
import { analyze } from '../fons/faber/semantic';
import { generate } from '../fons/faber/codegen';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');
const OUTPUT = join(ROOT, 'opus', 'rivus', 'fons', 'ts');

interface CompileResult {
    file: string;
    success: boolean;
    error?: string;
}

async function compileFile(fabPath: string): Promise<CompileResult> {
    const relPath = relative(SOURCE, fabPath);
    const outPath = join(OUTPUT, relPath.replace(/\.fab$/, '.ts'));

    try {
        const source = await Bun.file(fabPath).text();

        const { tokens, errors: tokenErrors } = tokenize(source);
        if (tokenErrors.length > 0) {
            const first = tokenErrors[0]!;
            throw new Error(`${first.position.line}:${first.position.column} ${first.text}`);
        }

        const { program, errors: parseErrors } = parse(tokens);
        if (parseErrors.length > 0) {
            const first = parseErrors[0]!;
            throw new Error(`${first.position.line}:${first.position.column} ${first.message}`);
        }

        if (!program) {
            throw new Error('Failed to parse program');
        }

        const { errors: semanticErrors } = analyze(program, { filePath: fabPath });
        if (semanticErrors.length > 0) {
            const first = semanticErrors[0]!;
            throw new Error(`${first.position.line}:${first.position.column} ${first.message}`);
        }

        const output = generate(program);

        await mkdir(dirname(outPath), { recursive: true });
        await Bun.write(outPath, output);

        return { file: relPath, success: true };
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        return { file: relPath, success: false, error: message };
    }
}

async function typeCheck(): Promise<boolean> {
    try {
        await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(OUTPUT, 'cli.ts')}`.quiet();
        return true;
    }
    catch {
        return false;
    }
}

async function main() {
    const start = performance.now();

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

    // Type check (TypeScript only)
    console.log('Type checking...');
    const tcOk = await typeCheck();
    if (!tcOk) {
        console.error('TypeScript type check failed');
        process.exit(1);
    }
    console.log('Type check passed');
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
