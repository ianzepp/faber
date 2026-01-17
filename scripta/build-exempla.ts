#!/usr/bin/env bun
/**
 * Compile fons/exempla/ using faber, rivus, or artifex.
 *
 * Usage:
 *   bun run exempla                          # faber + rivus (default)
 *   bun run exempla -t zig                   # faber + rivus, Zig target
 *   bun run exempla --no-rivus               # faber only
 *   bun run exempla --artifex                # faber + rivus + artifex
 *   bun run exempla --no-faber --no-rivus --artifex  # artifex only
 */

import { mkdir, readdir, rm, stat } from 'fs/promises';
import { basename, dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const EXEMPLA_SOURCE = join(ROOT, 'fons', 'exempla');
const EXEMPLA_OUTPUT = join(ROOT, 'opus', 'exempla');

type Compiler = 'faber' | 'rivus' | 'artifex';
type Target = 'ts' | 'zig' | 'py' | 'rs' | 'go';

const VALID_TARGETS = ['ts', 'zig', 'py', 'rs', 'go', 'all'] as const;
const ALL_TARGETS: Target[] = ['ts', 'zig', 'py', 'rs', 'go'];
const TARGET_EXT: Record<Target, string> = { ts: 'ts', zig: 'zig', py: 'py', rs: 'rs', go: 'go' };

interface Args {
    compilers: Compiler[];
    targets: Target[];
}

function parseArgs(): Args {
    const args = process.argv.slice(2);
    let faber = true;
    let rivus = true;
    let artifex = false;
    let targets: Target[] = ['ts'];

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];

        if (arg === '-t' || arg === '--target') {
            const t = args[++i];
            if (!VALID_TARGETS.includes(t as typeof VALID_TARGETS[number])) {
                console.error(`Unknown target '${t}'. Valid: ${VALID_TARGETS.join(', ')}`);
                process.exit(1);
            }
            targets = t === 'all' ? ALL_TARGETS : [t as Target];
        } else if (arg === '--faber') {
            faber = true;
        } else if (arg === '--no-faber') {
            faber = false;
        } else if (arg === '--rivus') {
            rivus = true;
        } else if (arg === '--no-rivus') {
            rivus = false;
        } else if (arg === '--artifex') {
            artifex = true;
        } else if (arg === '--no-artifex') {
            artifex = false;
        }
    }

    const compilers: Compiler[] = [];
    if (faber) compilers.push('faber');
    if (rivus) compilers.push('rivus');
    if (artifex) compilers.push('artifex');

    return { compilers, targets };
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
    // faber/rivus use compiled binaries, artifex uses wrapper script
    const compilerBin = (compiler === 'faber' || compiler === 'rivus')
        ? join(ROOT, 'opus', 'bin', compiler)
        : join(ROOT, 'scripta', compiler);
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

                // All compilers use same CLI: compile <file> -t <target>
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

async function verifyGo(): Promise<{ total: number; failed: number }> {
    const goDir = join(EXEMPLA_OUTPUT, 'go');
    const files = await findFiles(goDir, '.go');
    let failed = 0;

    for (const file of files) {
        try {
            await $`go build -o /dev/null ${file}`.quiet();
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
    const { compilers, targets } = parseArgs();
    const start = performance.now();

    if (compilers.length === 0) {
        console.log('No compilers selected. Use --faber, --rivus, or --artifex.');
        process.exit(0);
    }

    console.log(`Compiling exempla (compilers: ${compilers.join(', ')}, targets: ${targets.join(', ')})\n`);

    let totalCompileFailed = 0;
    let totalCompileCount = 0;

    for (const compiler of compilers) {
        console.log(`\n[${compiler}]`);
        const compile = await compileExempla(compiler, targets);
        totalCompileCount += compile.total;
        totalCompileFailed += compile.failed;
        if (compile.failed > 0) {
            console.log(`  ${compile.failed}/${compile.total} compilation(s) failed`);
        }
    }

    console.log('\nVerifying output...');

    const verifiers: Record<Target, () => Promise<{ total: number; failed: number }>> = {
        ts: verifyTypeScript,
        zig: verifyZig,
        py: verifyPython,
        rs: verifyRust,
        go: verifyGo,
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

    if (totalCompileFailed > 0 || verifyFailed > 0) {
        process.exit(1);
    }
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
