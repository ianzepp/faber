#!/usr/bin/env bun
/**
 * Compile fons/exempla/ using faber or rivus.
 *
 * Usage:
 *   bun run build:exempla                    # faber, TypeScript (default)
 *   bun run build:exempla -t zig             # faber, Zig
 *   bun run build:exempla -t all             # faber, all targets
 *   bun run build:exempla -c rivus           # rivus, TypeScript
 *   bun run build:exempla -c rivus -t all    # rivus, all targets
 */

import { mkdir, readdir, rm, stat } from 'fs/promises';
import { basename, dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const EXEMPLA_SOURCE = join(ROOT, 'fons', 'exempla');
const EXEMPLA_OUTPUT = join(ROOT, 'opus', 'exempla');

type Compiler = 'faber' | 'rivus';
type Target = 'ts' | 'zig' | 'py' | 'rs';

const VALID_COMPILERS = ['faber', 'rivus'] as const;
const VALID_TARGETS = ['ts', 'zig', 'py', 'rs', 'all'] as const;
const ALL_TARGETS: Target[] = ['ts', 'zig', 'py', 'rs'];
const TARGET_EXT: Record<Target, string> = { ts: 'ts', zig: 'zig', py: 'py', rs: 'rs' };

interface Args {
    compiler: Compiler;
    targets: Target[];
}

function parseArgs(): Args {
    const args = process.argv.slice(2);
    let compiler: Compiler = 'faber';
    let targets: Target[] = ['ts'];

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];

        if (arg === '-c' || arg === '--compiler') {
            const c = args[++i];
            if (!VALID_COMPILERS.includes(c as Compiler)) {
                console.error(`Unknown compiler '${c}'. Valid: ${VALID_COMPILERS.join(', ')}`);
                process.exit(1);
            }
            compiler = c as Compiler;
        }
        else if (arg === '-t' || arg === '--target') {
            const t = args[++i];
            if (!VALID_TARGETS.includes(t as typeof VALID_TARGETS[number])) {
                console.error(`Unknown target '${t}'. Valid: ${VALID_TARGETS.join(', ')}`);
                process.exit(1);
            }
            targets = t === 'all' ? ALL_TARGETS : [t as Target];
        }
    }

    return { compiler, targets };
}

async function findFiles(dir: string, ext: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        const fullPath = join(dir, entry);
        const s = await stat(fullPath);
        if (s.isDirectory()) {
            files.push(...await findFiles(fullPath, ext));
        }
        else if (entry.endsWith(ext)) {
            files.push(fullPath);
        }
    }

    return files;
}

async function compileExempla(compiler: Compiler, targets: Target[]): Promise<{ total: number; failed: number }> {
    const compilerBin = join(ROOT, 'opus', 'bin', compiler);
    const fabFiles = await findFiles(EXEMPLA_SOURCE, '.fab');

    // Clear output directories for each target to ensure fresh builds
    for (const target of targets) {
        const targetDir = join(EXEMPLA_OUTPUT, target);
        await rm(targetDir, { recursive: true, force: true });
    }

    let failed = 0;

    for (const fabPath of fabFiles) {
        const relPath = relative(EXEMPLA_SOURCE, fabPath);
        const name = basename(fabPath, '.fab');
        const subdir = dirname(relPath);

        for (const target of targets) {
            const ext = TARGET_EXT[target];
            const outDir = join(EXEMPLA_OUTPUT, target, subdir);
            const outPath = join(outDir, `${name}.${ext}`);

            try {
                await mkdir(outDir, { recursive: true });
                const result = await $`${compilerBin} compile ${fabPath} -t ${target}`.quiet();
                await Bun.write(outPath, result.stdout);
                console.log(`  ${relPath} -> ${target}/${subdir}/${name}.${ext}`);
            }
            catch (err: any) {
                console.error(`  ${relPath} [${target}] FAILED`);
                if (err.stderr) console.error(`    ${err.stderr.toString().trim()}`);
                failed++;
            }
        }
    }

    return { total: fabFiles.length * targets.length, failed };
}

async function verifyTypeScript(): Promise<{ total: number; failed: number }> {
    const tsDir = join(EXEMPLA_OUTPUT, 'ts');
    const files = await findFiles(tsDir, '.ts');
    let failed = 0;

    for (const file of files) {
        try {
            await $`bun build --no-bundle ${file}`.quiet();
        }
        catch {
            console.error(`  ${relative(EXEMPLA_OUTPUT, file)}: type error`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function verifyZig(): Promise<{ total: number; failed: number }> {
    const zigDir = join(EXEMPLA_OUTPUT, 'zig');
    const files = await findFiles(zigDir, '.zig');
    let failed = 0;

    for (const file of files) {
        const name = basename(file, '.zig');
        const output = join(dirname(file), name);

        try {
            await $`zig build-exe ${file} -femit-bin=${output}`.quiet();
        }
        catch (err: any) {
            console.error(`  ${relative(EXEMPLA_OUTPUT, file)}: compile error`);
            const errText = err.stderr?.toString() || '';
            const firstError = errText.split('\n').slice(0, 3).join('\n');
            if (firstError) console.error(`    ${firstError}`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function verifyPython(): Promise<{ total: number; failed: number }> {
    const pyDir = join(EXEMPLA_OUTPUT, 'py');
    const files = await findFiles(pyDir, '.py');
    let failed = 0;

    for (const file of files) {
        try {
            await $`python3 -m py_compile ${file}`.quiet();
        }
        catch (err: any) {
            console.error(`  ${relative(EXEMPLA_OUTPUT, file)}: syntax error`);
            const errText = err.stderr?.toString() || '';
            if (errText) console.error(`    ${errText.trim()}`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function verifyRust(): Promise<{ total: number; failed: number }> {
    const rsDir = join(EXEMPLA_OUTPUT, 'rs');
    const files = await findFiles(rsDir, '.rs');
    let failed = 0;

    for (const file of files) {
        try {
            await $`rustc --emit=metadata --edition=2021 -o /dev/null ${file}`.quiet();
        }
        catch (err: any) {
            console.error(`  ${relative(EXEMPLA_OUTPUT, file)}: compile error`);
            const errText = err.stderr?.toString() || '';
            const firstError = errText.split('\n').slice(0, 5).join('\n');
            if (firstError) console.error(`    ${firstError}`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function main() {
    const { compiler, targets } = parseArgs();
    const start = performance.now();

    console.log(`Compiling exempla with ${compiler} (targets: ${targets.join(', ')})\n`);

    const compile = await compileExempla(compiler, targets);
    if (compile.failed > 0) {
        console.log(`\n${compile.failed}/${compile.total} compilation(s) failed\n`);
    }

    console.log('\nVerifying output...');

    const verifiers: Record<Target, () => Promise<{ total: number; failed: number }>> = {
        ts: verifyTypeScript,
        zig: verifyZig,
        py: verifyPython,
        rs: verifyRust,
    };

    let verifyFailed = 0;
    for (const target of targets) {
        process.stdout.write(`  ${target}: `);
        const result = await verifiers[target]();
        if (result.failed === 0) {
            console.log(`OK (${result.total} files)`);
        }
        else {
            console.log(`${result.failed}/${result.total} failed`);
            verifyFailed += result.failed;
        }
    }

    const elapsed = performance.now() - start;
    console.log(`\nDone (${elapsed.toFixed(0)}ms)`);

    if (compile.failed > 0 || verifyFailed > 0) {
        process.exit(1);
    }
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
