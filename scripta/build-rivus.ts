#!/usr/bin/env bun
/**
 * Build rivus (bootstrap compiler) from fons/rivus/ using faber internals.
 *
 * Compiles all .fab files in parallel without spawning processes.
 *
 * Usage:
 *   bun scripta/build-rivus.ts           # TypeScript (default)
 *   bun scripta/build-rivus.ts -t zig    # Zig output
 *   bun scripta/build-rivus.ts -t py     # Python output
 */

import { Glob } from 'bun';
import { mkdir } from 'fs/promises';
import { dirname, join, relative } from 'path';
import { tokenize } from '../fons/faber/tokenizer';
import { parse } from '../fons/faber/parser';
import { analyze } from '../fons/faber/semantic';
import { generate, type CodegenTarget } from '../fons/faber/codegen';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');

const VALID_TARGETS = ['ts', 'zig', 'py', 'rs', 'cpp', 'fab'] as const;
const EXT: Record<CodegenTarget, string> = {
    ts: '.ts',
    zig: '.zig',
    py: '.py',
    rs: '.rs',
    cpp: '.cpp',
    fab: '.fab',
};

interface CompileResult {
    file: string;
    success: boolean;
    error?: string;
}

async function compileFile(fabPath: string, target: CodegenTarget, outputDir: string): Promise<CompileResult> {
    const relPath = relative(SOURCE, fabPath);
    const outPath = join(outputDir, relPath.replace(/\.fab$/, EXT[target]));

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

        const output = generate(program, { target });

        await mkdir(dirname(outPath), { recursive: true });
        await Bun.write(outPath, output);

        return { file: relPath, success: true };
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        return { file: relPath, success: false, error: message };
    }
}

async function typeCheck(outputDir: string): Promise<boolean> {
    try {
        await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(outputDir, 'cli.ts')}`.quiet();
        return true;
    }
    catch {
        return false;
    }
}

function parseArgs(): { target: CodegenTarget } {
    const args = process.argv.slice(2);
    let target: CodegenTarget = 'ts';

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === '-t' || arg === '--target') {
            const t = args[++i];
            if (!t || !VALID_TARGETS.includes(t as typeof VALID_TARGETS[number])) {
                console.error(`Unknown target '${t}'. Valid: ${VALID_TARGETS.join(', ')}`);
                process.exit(1);
            }
            target = t as CodegenTarget;
        }
    }

    return { target };
}

async function main() {
    const { target } = parseArgs();
    const outputDir = join(ROOT, 'opus', 'rivus', 'fons', target);
    const start = performance.now();

    // Find all .fab files
    const glob = new Glob('**/*.fab');
    const files: string[] = [];
    for await (const file of glob.scan({ cwd: SOURCE, absolute: true })) {
        files.push(file);
    }

    // Compile all in parallel
    const results = await Promise.all(files.map(f => compileFile(f, target, outputDir)));

    const elapsed = performance.now() - start;
    const succeeded = results.filter(r => r.success).length;
    const failed = results.filter(r => !r.success);

    // Report failures
    for (const f of failed) {
        console.error(`${f.file}: ${f.error}`);
    }

    // Summary
    const relOut = relative(ROOT, outputDir);
    console.log(`Compiled ${succeeded}/${results.length} files to ${relOut}/ (${target}, ${elapsed.toFixed(0)}ms)`);

    if (failed.length > 0) {
        process.exit(1);
    }

    // Inject extern implementations for module resolution (TypeScript only)
    if (target === 'ts') {
        const modulusPath = join(outputDir, 'semantic', 'modulus.ts');
        let modulusContent = await Bun.file(modulusPath).text();

        // Replace declare statements with actual implementations
        const externImpls = `
// FILE I/O IMPLEMENTATIONS (injected by build-rivus.ts)
import { readFileSync, existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
const _readFileSync = (via: string): string => readFileSync(via, 'utf-8');
const _existsSync = (via: string): boolean => existsSync(via);
const _dirname = (via: string): string => dirname(via);
const _resolve = (basis: string, relativum: string): string => resolve(basis, relativum);
`;

        // Remove declare statements and add implementations after imports
        modulusContent = modulusContent.replace(
            /declare function _readFileSync.*?;\ndeclare function _existsSync.*?;\ndeclare function _dirname.*?;\ndeclare function _resolve.*?;/s,
            externImpls.trim()
        );

        await Bun.write(modulusPath, modulusContent);
    }

    // Type-check and compile (TypeScript only)
    if (target === 'ts') {
        process.stdout.write('Type-checking... ');
        const tcStart = performance.now();
        const tcOk = await typeCheck(outputDir);
        const tcElapsed = performance.now() - tcStart;

        if (tcOk) {
            console.log(`OK (${tcElapsed.toFixed(0)}ms)`);
        }
        else {
            console.log('FAILED');
            await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(outputDir, 'cli.ts')}`;
            process.exit(1);
        }

        // Build standalone executable
        process.stdout.write('Compiling executable... ');
        const buildStart = performance.now();
        const binDir = join(ROOT, 'opus', 'bin');
        await mkdir(binDir, { recursive: true });
        const outExe = join(binDir, 'rivus');
        await $`bun build ${join(outputDir, 'cli.ts')} --compile --outfile=${outExe}`.quiet();
        await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();
        const buildElapsed = performance.now() - buildStart;
        console.log(`OK (${buildElapsed.toFixed(0)}ms)`);
    }
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
