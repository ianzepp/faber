#!/usr/bin/env bun
/**
 * Compile and verify fons/exempla/ using various compilers.
 *
 * All compilers use stdin/stdout: cat file.fab | compiler emit -t target
 * Output goes to opus/<compiler>/exempla/<target>/
 *
 * Usage:
 *   bun run build:exempla -c nanus-ts                       # compile + verify
 *   bun run build:exempla -c rivus-nanus-ts                 # compile + verify
 *   bun run build:exempla -c rivus-nanus-ts,rivus-nanus-py  # multiple compilers
 *   bun run build:exempla -c nanus-ts -t zig                # specific target
 *   bun run build:exempla -c nanus-ts --no-verify           # compile only
 *   bun run build:exempla -c nanus-ts --verify-only         # verify only (no compile)
 */

import { mkdir, readdir, rm, stat } from 'fs/promises';
import { basename, dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const EXEMPLA_SOURCE = join(ROOT, 'fons', 'exempla');
const OPUS = join(ROOT, 'opus');

type Target = 'ts' | 'zig' | 'py' | 'rs' | 'go';

const VALID_TARGETS = ['ts', 'zig', 'py', 'rs', 'go', 'all'] as const;
const ALL_TARGETS: Target[] = ['ts', 'zig', 'py', 'rs', 'go'];
const TARGET_EXT: Record<Target, string> = { ts: 'ts', zig: 'zig', py: 'py', rs: 'rs', go: 'go' };

interface CompilerSpec {
    name: string;
    bin: string;
}

interface Args {
    compilers: CompilerSpec[];
    targets: Target[];
    compile: boolean;
    verify: boolean;
}

function parseArgs(): Args {
    const args = process.argv.slice(2);
    let compilerNames: string[] = [];
    let targets: Target[] = ['ts'];
    let compile = true;
    let verify = true;

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];

        if (arg === '-t' || arg === '--target') {
            const t = args[++i];
            if (!VALID_TARGETS.includes(t as (typeof VALID_TARGETS)[number])) {
                console.error(`Unknown target '${t}'. Valid: ${VALID_TARGETS.join(', ')}`);
                process.exit(1);
            }
            targets = t === 'all' ? ALL_TARGETS : [t as Target];
        } else if (arg === '-c' || arg === '--compiler') {
            const value = args[++i];
            if (!value) {
                console.error('Missing value for -c/--compiler');
                process.exit(1);
            }
            compilerNames = value.split(',').map(c => c.trim());
        } else if (arg === '--no-verify') {
            verify = false;
        } else if (arg === '--verify-only') {
            compile = false;
            verify = true;
        }
    }

    const compilers: CompilerSpec[] = compilerNames.map(name => ({
        name,
        bin: join(ROOT, 'opus', 'bin', name),
    }));

    return { compilers, targets, compile, verify };
}

async function findFiles(dir: string, ext: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        const fullPath = join(dir, entry);
        const s = await stat(fullPath);
        if (s.isDirectory()) {
            files.push(...(await findFiles(fullPath, ext)));
        } else if (entry.endsWith(ext)) {
            files.push(fullPath);
        }
    }

    return files;
}

async function compileExempla(compiler: CompilerSpec, targets: Target[]): Promise<{ total: number; failed: number }> {
    const fabFiles = await findFiles(EXEMPLA_SOURCE, '.fab');
    const outputBase = join(OPUS, compiler.name, 'exempla');

    // Clear output directories for each target to ensure fresh builds
    for (const target of targets) {
        const targetDir = join(outputBase, target);
        await rm(targetDir, { recursive: true, force: true });
    }

    let failed = 0;

    for (const fabPath of fabFiles) {
        const relPath = relative(EXEMPLA_SOURCE, fabPath);
        const name = basename(fabPath, '.fab');
        const subdir = dirname(relPath);

        for (const target of targets) {
            const ext = TARGET_EXT[target];
            const outDir = join(outputBase, target, subdir);
            const outPath = join(outDir, `${name}.${ext}`);

            try {
                await mkdir(outDir, { recursive: true });

                // All compilers use stdin: cat file | compiler emit -t target
                const result = await $`cat ${fabPath} | ${compiler.bin} emit -t ${target}`.quiet();

                await Bun.write(outPath, result.stdout);
                console.log(`  ${relPath} -> ${compiler.name}/${target}/${subdir}/${name}.${ext}`);
            } catch (err: any) {
                console.error(`  ${relPath} [${target}] FAILED`);
                if (err.stderr) console.error(`    ${err.stderr.toString().trim()}`);
                failed++;
            }
        }
    }

    return { total: fabFiles.length * targets.length, failed };
}

async function verifyTypeScript(outputBase: string): Promise<{ total: number; failed: number }> {
    const tsDir = join(outputBase, 'ts');
    const files = await findFiles(tsDir, '.ts');
    let failed = 0;

    for (const file of files) {
        try {
            await $`bun build --no-bundle ${file}`.quiet();
        } catch {
            console.error(`  ${relative(outputBase, file)}: type error`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function verifyZig(outputBase: string): Promise<{ total: number; failed: number }> {
    const zigDir = join(outputBase, 'zig');
    const files = await findFiles(zigDir, '.zig');
    let failed = 0;

    for (const file of files) {
        const name = basename(file, '.zig');
        const output = join(dirname(file), name);

        try {
            await $`zig build-exe ${file} -femit-bin=${output}`.quiet();
        } catch (err: any) {
            console.error(`  ${relative(outputBase, file)}: compile error`);
            const errText = err.stderr?.toString() || '';
            const firstError = errText.split('\n').slice(0, 3).join('\n');
            if (firstError) console.error(`    ${firstError}`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function verifyPython(outputBase: string): Promise<{ total: number; failed: number }> {
    const pyDir = join(outputBase, 'py');
    const files = await findFiles(pyDir, '.py');
    let failed = 0;

    for (const file of files) {
        try {
            await $`python3 -m py_compile ${file}`.quiet();
        } catch (err: any) {
            console.error(`  ${relative(outputBase, file)}: syntax error`);
            const errText = err.stderr?.toString() || '';
            if (errText) console.error(`    ${errText.trim()}`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function verifyRust(outputBase: string): Promise<{ total: number; failed: number }> {
    const rsDir = join(outputBase, 'rs');
    const files = await findFiles(rsDir, '.rs');
    let failed = 0;

    for (const file of files) {
        try {
            await $`rustc --emit=metadata --edition=2021 -o /dev/null ${file}`.quiet();
        } catch (err: any) {
            console.error(`  ${relative(outputBase, file)}: compile error`);
            const errText = err.stderr?.toString() || '';
            const firstError = errText.split('\n').slice(0, 5).join('\n');
            if (firstError) console.error(`    ${firstError}`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function verifyGo(outputBase: string): Promise<{ total: number; failed: number }> {
    const goDir = join(outputBase, 'go');
    const files = await findFiles(goDir, '.go');
    let failed = 0;

    for (const file of files) {
        try {
            await $`go build -o /dev/null ${file}`.quiet();
        } catch (err: any) {
            console.error(`  ${relative(outputBase, file)}: compile error`);
            const errText = err.stderr?.toString() || '';
            const firstError = errText.split('\n').slice(0, 5).join('\n');
            if (firstError) console.error(`    ${firstError}`);
            failed++;
        }
    }

    return { total: files.length, failed };
}

async function main() {
    const { compilers, targets, compile, verify } = parseArgs();
    const start = performance.now();

    if (compilers.length === 0) {
        console.log('No compilers selected. Use -c <compiler> (e.g., -c nanus-ts or -c rivus-nanus-ts).');
        process.exit(0);
    }

    const compilerNames = compilers.map(c => c.name).join(', ');
    let totalCompileFailed = 0;
    let totalCompileCount = 0;

    if (compile) {
        console.log(`Compiling exempla (compilers: ${compilerNames}, targets: ${targets.join(', ')})\n`);

        for (const compiler of compilers) {
            console.log(`\n[${compiler.name}]`);
            const result = await compileExempla(compiler, targets);
            totalCompileCount += result.total;
            totalCompileFailed += result.failed;
            if (result.failed > 0) {
                console.log(`  ${result.failed}/${result.total} compilation(s) failed`);
            }
        }
    }

    let verifyFailed = 0;

    if (verify) {
        console.log(`\nVerifying exempla (compilers: ${compilerNames}, targets: ${targets.join(', ')})\n`);

        const verifiers: Record<Target, (outputBase: string) => Promise<{ total: number; failed: number }>> = {
            ts: verifyTypeScript,
            zig: verifyZig,
            py: verifyPython,
            rs: verifyRust,
            go: verifyGo,
        };

        for (const compiler of compilers) {
            const outputBase = join(OPUS, compiler.name, 'exempla');
            for (const target of targets) {
                process.stdout.write(`  ${compiler.name}/${target}: `);
                const result = await verifiers[target](outputBase);
                if (result.failed === 0) {
                    console.log(`OK (${result.total} files)`);
                } else {
                    console.log(`${result.failed}/${result.total} failed`);
                    verifyFailed += result.failed;
                }
            }
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
