#!/usr/bin/env bun
/**
 * Build rivus (bootstrap compiler) from fons/rivus/ using faber-ts or nanus.
 *
 * Uses the specified compiler to compile all .fab files in parallel.
 * Output target is derived from the compiler (e.g., nanus-go emits Go code).
 *
 * Requires: bun run build:faber-ts (or build:nanus-ts, build:nanus-go, build:nanus-rs, build:nanus-py) first
 *
 * Usage:
 *   bun scripta/build-rivus.ts                    # uses faber-ts -> TypeScript
 *   bun scripta/build-rivus.ts -c nanus-ts        # -> TypeScript
 *   bun scripta/build-rivus.ts -c nanus-go        # -> Go
 *   bun scripta/build-rivus.ts -c nanus-rs        # -> Rust
 *   bun scripta/build-rivus.ts -c nanus-py        # -> Python
 *   bun scripta/build-rivus.ts -c faber-ts --no-typecheck
 */

import { Glob } from 'bun';
import { mkdir, symlink, unlink } from 'fs/promises';
import { basename, dirname, join, relative } from 'path';
import { $ } from 'bun';

// =============================================================================
// CONSTANTS AND TYPES
// =============================================================================

type Compiler = 'faber-ts' | 'nanus-ts' | 'nanus-go' | 'nanus-rs' | 'nanus-py';
type Target = 'ts' | 'go' | 'rs' | 'py';

const VALID_COMPILERS: Compiler[] = ['faber-ts', 'nanus-ts', 'nanus-go', 'nanus-rs', 'nanus-py'];

const COMPILER_TARGET: Record<Compiler, Target> = {
    'faber-ts': 'ts',
    'nanus-ts': 'ts',
    'nanus-go': 'go',
    'nanus-rs': 'rs',
    'nanus-py': 'py',
};

// =============================================================================
// CONFIGURATION
// =============================================================================

let compiler: Compiler = 'faber-ts';
let skipTypecheck = false;

/**
 * Parse command line arguments
 */
function parseArgs() {
    const args = process.argv.slice(2);
    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        switch (arg) {
            case '-c':
            case '--compiler':
                const c = args[++i] as Compiler;
                if (!VALID_COMPILERS.includes(c)) {
                    console.error(`Unknown compiler '${c}'. Valid: ${VALID_COMPILERS.join(', ')}`);
                    process.exit(1);
                }
                compiler = c;
                break;
            case '--no-typecheck':
                skipTypecheck = true;
                break;
            default:
                console.error(`Unknown argument: ${arg}`);
                console.error('Usage: build-rivus.ts [-c compiler] [--no-typecheck]');
                process.exit(1);
        }
    }
}

parseArgs();

const target: Target = COMPILER_TARGET[compiler];

// =============================================================================
// PATH CONFIGURATION
// =============================================================================

const ROOT = join(import.meta.dir, '..');
const SOURCE = join(ROOT, 'fons', 'rivus');

const OUTPUT_PATH: Record<Compiler, string> = {
    'faber-ts': join(ROOT, 'opus', 'faber-ts', 'fons'),
    'nanus-ts': join(ROOT, 'opus', 'nanus-ts', 'fons'),
    'nanus-go': join(ROOT, 'opus', 'nanus-go', 'fons'),
    'nanus-rs': join(ROOT, 'opus', 'nanus-rs', 'src'),
    'nanus-py': join(ROOT, 'opus', 'nanus-py', 'fons'),
};

const FILE_EXT: Record<Target, string> = {
    'ts': '.ts',
    'go': '.go',
    'rs': '.rs',
    'py': '.py',
};

const OUTPUT = OUTPUT_PATH[compiler];
const COMPILER_BIN = join(ROOT, 'opus', 'bin', compiler);

interface CompileResult {
    file: string;
    success: boolean;
    error?: string;
}

/**
 * Compile a single .fab file using the configured compiler.
 * All compilers use stdin/stdout: cat file | compiler emit > output
 */
async function compileFile(fabPath: string): Promise<CompileResult> {
    const relPath = relative(SOURCE, fabPath);
    const outPath = join(OUTPUT, relPath.replace(/\.fab$/, FILE_EXT[target]));

    try {
        await mkdir(dirname(outPath), { recursive: true });

        // Go target needs package name (root files -> "main", subdirs -> dir name)
        const pkg = target === 'go'
            ? (dirname(relPath) === '.' ? 'main' : basename(dirname(relPath)))
            : null;

        // All compilers: cat file | compiler emit [flags] --stdin-filename file > output
        const result = pkg
            ? await $`cat ${fabPath} | ${COMPILER_BIN} emit -p ${pkg} --stdin-filename ${fabPath}`.nothrow().quiet()
            : await $`cat ${fabPath} | ${COMPILER_BIN} emit --stdin-filename ${fabPath}`.nothrow().quiet();

        if (result.exitCode !== 0) {
            throw new Error(result.stderr.toString().trim() || `Exit code ${result.exitCode}`);
        }

        await Bun.write(outPath, result.stdout);
        return { file: relPath, success: true };
    } catch (err) {
        return { file: relPath, success: false, error: err instanceof Error ? err.message : String(err) };
    }
}

async function typeCheck(): Promise<boolean> {
    // Type check from cli.ts entry point (the native shim)
    const result =
        await $`npx tsc --noEmit --skipLibCheck --target ES2022 --module ESNext --moduleResolution Bundler ${join(OUTPUT, 'cli.ts')}`.nothrow();
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
    // to opus/{compiler}/norma/hal (not opus/{compiler}/fons/norma/hal)
    const halDest = join(dirname(OUTPUT), 'norma', 'hal');
    await mkdir(halDest, { recursive: true });

    const glob = new Glob('*.ts');
    for await (const file of glob.scan({ cwd: halSource, absolute: false })) {
        if (file.endsWith('.test.ts')) { continue; }
        const src = join(halSource, file);
        const dest = join(halDest, file);
        await Bun.write(dest, await Bun.file(src).text());
    }
}

async function copyCliShim(): Promise<void> {
    const shimSource = join(ROOT, 'fons', 'rivus-cli', 'ts.ts');
    const shimDest = join(OUTPUT, 'cli.ts');
    await Bun.write(shimDest, await Bun.file(shimSource).text());
}

async function buildExecutableTs(): Promise<void> {
    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });

    const exeName = `rivus-${compiler}`;
    const outExe = join(binDir, exeName);
    // Use cli.ts (the native shim) as entry point instead of rivus.ts
    await $`bun build ${join(OUTPUT, 'cli.ts')} --compile --outfile=${outExe}`.quiet();
    await $`bash -c 'rm -f .*.bun-build 2>/dev/null || true'`.quiet();

    // faber-ts is the primary compiler, symlink rivus -> rivus-faber-ts
    if (compiler === 'faber-ts') {
        const symlinkPath = join(binDir, 'rivus');
        try { await unlink(symlinkPath); } catch { /* ignore */ }
        await symlink(exeName, symlinkPath);
    }
}

async function buildExecutablePy(): Promise<void> {
    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });

    const exeName = `rivus-${compiler}`;
    const outExe = join(binDir, exeName);
    const script = `#!/bin/bash
exec python3 "$(dirname "$0")/../${compiler}/fons/rivus.py" "$@"
`;
    await Bun.write(outExe, script);
    await $`chmod +x ${outExe}`.quiet();
}

async function buildExecutableGo(): Promise<void> {
    const binDir = join(ROOT, 'opus', 'bin');
    const moduleDir = dirname(OUTPUT);
    await mkdir(binDir, { recursive: true });

    // Initialize go.mod if not present
    const goMod = join(moduleDir, 'go.mod');
    if (!(await Bun.file(goMod).exists())) {
        await $`cd ${moduleDir} && go mod init rivus`.quiet();
    }

    const exeName = `rivus-${compiler}`;
    const outExe = join(binDir, exeName);
    const result = await $`cd ${moduleDir} && GOWORK=off go build -o ${outExe} ./fons/`.nothrow().quiet();
    if (result.exitCode !== 0) {
        console.error(result.stderr.toString());
        process.exit(1);
    }
}

async function buildExecutableRs(): Promise<void> {
    const binDir = join(ROOT, 'opus', 'bin');
    const moduleDir = dirname(OUTPUT);
    await mkdir(binDir, { recursive: true });

    if (compiler === 'nanus-rs') {
        const compilerExe = join(binDir, compiler);
        console.log("  Bundling Rust modules...");
        await $`${compilerExe} bundle ${OUTPUT} --entry rivus.rs`.quiet();

        // Generate a lib entrypoint containing only module declarations.
        // The generated CLI (src/rivus.rs) is currently TS-shaped and not Rust-native.
        const rivusRs = join(OUTPUT, 'rivus.rs');
        const libRs = join(OUTPUT, 'lib.rs');
        const rivusText = await Bun.file(rivusRs).text();
        const lines = rivusText.split('\n');
        const modLines: string[] = [];
        for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed.length === 0) break;
            if (trimmed.startsWith('pub mod ')) modLines.push(line);
        }
        await Bun.write(libRs, `${modLines.join('\n')}\n`);
    }

    // Create Cargo.toml if not present
    const cargoToml = join(moduleDir, 'Cargo.toml');
    if (compiler === 'nanus-rs') {
        const toml = `[package]
name = "rivus"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"

[features]
cli = []

[[bin]]
name = "rivus"
path = "src/rivus.rs"
required-features = ["cli"]
`;
        await Bun.write(cargoToml, toml);
    } else if (!(await Bun.file(cargoToml).exists())) {
        const toml = `[package]
name = "rivus"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "rivus"
path = "src/rivus.rs"
`;
        await Bun.write(cargoToml, toml);
    }

    if (compiler === 'nanus-rs') {
        const result = await $`cd ${moduleDir} && cargo check`.nothrow().quiet();
        if (result.exitCode !== 0) {
            console.error(result.stderr.toString());
            process.exit(1);
        }
        return;
    }

    const result = await $`cd ${moduleDir} && cargo build --release`.nothrow().quiet();
    if (result.exitCode !== 0) {
        console.error(result.stderr.toString());
        process.exit(1);
    }

    const exeName = `rivus-${compiler}`;
    const outExe = join(binDir, exeName);
    await $`cp ${join(moduleDir, 'target', 'release', 'rivus')} ${outExe}`.quiet();
}

/**
 * Main build function that orchestrates the rivus compilation process
 */
async function main() {
    const start = performance.now();

    // =============================================================================
    // VALIDATION
    // =============================================================================

    // Verify compiler binary exists
    if (!(await Bun.file(COMPILER_BIN).exists())) {
        console.error(`Error: ${compiler} binary not found at opus/bin/${compiler}`);
        console.error(`Run \`bun run build:${compiler}\` first.`);
        process.exit(1);
    }

    console.log(`Using compiler: ${compiler}, target: ${target}`);

    // =============================================================================
    // FILE DISCOVERY AND COMPILATION
    // =============================================================================

    // Find all .fab files in the source directory
    console.log('Discovering source files...');
    const glob = new Glob('**/*.fab');
    const files: string[] = [];
    for await (const file of glob.scan({ cwd: SOURCE, absolute: true })) {
        files.push(file);
    }
    console.log(`Found ${files.length} .fab files`);

    // Compile all files in parallel for better performance
    console.log('Compiling files...');
    const results = await Promise.all(files.map(f => compileFile(f)));

    // =============================================================================
    // RESULTS AND ERROR REPORTING
    // =============================================================================

    const elapsed = performance.now() - start;
    const succeeded = results.filter(r => r.success).length;
    const failedResults = results.filter(r => !r.success);

    // Report compilation failures
    if (failedResults.length > 0) {
        console.error('\nCompilation failures:');
        for (const f of failedResults) {
            console.error(`  ${f.file}: ${f.error}`);
        }
    }

    // Summary
    const relOut = relative(ROOT, OUTPUT);
    console.log(`\nCompiled ${succeeded}/${results.length} files to ${relOut}/ (${elapsed.toFixed(0)}ms)`);

    if (failedResults.length > 0) {
        process.exit(1);
    }

    // =============================================================================
    // TARGET-SPECIFIC POST-PROCESSING
    // =============================================================================

    if (target === 'ts') {
        console.log('\nTypeScript post-processing:');

        // Copy native CLI shim (provides I/O for the pure rivus library)
        await copyCliShim();
        console.log('  Copied CLI shim');

        // Type check the generated TypeScript
        if (!skipTypecheck) {
            console.log('  Type checking...');
            const tcOk = await typeCheck();
            if (!tcOk) {
                console.error('  TypeScript type check failed');
                process.exit(1);
            }
            console.log('  Type check passed');
        }

        // Inject runtime implementations for semantic analyzer (module resolution)
        await injectExternImpls();
        console.log('  Injected external implementations');

        // Build the final executable
        console.log('  Building rivus executable...');
        await buildExecutableTs();
        console.log(`  Built opus/bin/rivus-${compiler}`);
    } else if (target === 'go') {
        console.log('\nGo post-processing:');
        console.log('  Building rivus executable...');
        await buildExecutableGo();
        console.log(`  Built opus/bin/rivus-${compiler}`);
    } else if (target === 'rs') {
        console.log('\nRust post-processing:');
        console.log(compiler === 'nanus-rs' ? '  Checking rivus crate...' : '  Building rivus executable...');
        await buildExecutableRs();
        if (compiler === 'nanus-rs') {
            console.log('  Checked opus/nanus-rs (no binary built)');
        } else {
            console.log(`  Built opus/bin/rivus-${compiler}`);
        }
    } else if (target === 'py') {
        console.log('\nPython post-processing:');
        await buildExecutablePy();
        console.log(`  Built opus/bin/rivus-${compiler}`);
    }
}

main().catch(err => {
    if (err && typeof err === 'object' && 'stderr' in err) {
        console.error(err.stderr.toString());
    } else {
        console.error(err);
    }
    process.exit(1);
});
