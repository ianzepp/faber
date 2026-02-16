#!/usr/bin/env bun
/**
 * Primary build pipeline for Faber (radix-rs focused).
 *
 *   Stage 1: nanus-rs (bootstrap compiler)
 *   Stage 2: radix-rs (primary compiler)
 *   Stage 3: norma-rs (stdlib crate)
 *   Stage 4: exempla codegen (fab → rs via radix-rs)
 *   Stage 5: exempla verify (rustc compiles generated Rust)
 *   Stage 6: rivus codegen (fab → rs via radix-rs)
 *
 * Prework: wipes opus/* for clean builds.
 *
 * Usage:
 *   bun run build                    # full build
 *   bun run build --verbose          # show subprocess output
 *   bun run build --stage 2          # run up to stage N only
 */

import { mkdir, readdir, rm, stat, copyFile } from 'fs/promises';
import { basename, dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const OPUS = join(ROOT, 'opus');

const NANUS_RS_DIR = join(ROOT, 'fons', 'nanus-rs');
const RADIX_RS_DIR = join(ROOT, 'fons', 'radix-rs');
const NORMA_RS_DIR = join(ROOT, 'fons', 'norma-rs');
const EXEMPLA_DIR = join(ROOT, 'fons', 'exempla');
const RIVUS_DIR = join(ROOT, 'fons', 'rivus');

function parseArgs(): { verbose: boolean; maxStage: number } {
    const args = process.argv.slice(2);
    const verbose = args.some(arg => arg === '-v' || arg === '--verbose');

    let maxStage = 6;
    for (let i = 0; i < args.length; i++) {
        if (args[i] === '--stage' || args[i] === '-s') {
            const value = parseInt(args[i + 1], 10);
            if (isNaN(value) || value < 1 || value > 6) {
                console.error('Error: --stage requires a value between 1 and 6');
                process.exit(1);
            }
            maxStage = value;
            break;
        }
    }

    return { verbose, maxStage };
}

interface StepResult {
    name: string;
    success: boolean;
    elapsed: number;
    error?: string;
}

async function step(name: string, verbose: boolean, fn: () => Promise<void>): Promise<StepResult> {
    const start = performance.now();

    if (verbose) {
        console.log(`\n=== ${name} ===\n`);
    } else {
        process.stdout.write(`  ${name}... `);
    }

    try {
        await fn();
        const elapsed = performance.now() - start;

        if (verbose) {
            console.log(`\n=== ${name} OK (${elapsed.toFixed(0)}ms) ===`);
        } else {
            console.log(`OK (${elapsed.toFixed(0)}ms)`);
        }

        return { name, success: true, elapsed };
    } catch (err: any) {
        const elapsed = performance.now() - start;
        const error = err.stderr?.toString().trim() || err.message || 'unknown error';

        if (verbose) {
            console.log(`\n=== ${name} FAILED (${elapsed.toFixed(0)}ms) ===`);
            console.error(error);
        } else {
            console.log(`FAILED (${elapsed.toFixed(0)}ms)`);
        }

        return { name, success: false, elapsed, error };
    }
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

/**
 * Compile .fab files to .rs via radix-rs, writing output to outDir.
 */
async function compileToRust(
    radixBin: string,
    sourceDir: string,
    outDir: string,
    verbose: boolean,
): Promise<{ total: number; failed: number }> {
    const fabFiles = await findFiles(sourceDir, '.fab');

    await rm(outDir, { recursive: true, force: true });

    let failed = 0;

    for (const fabPath of fabFiles) {
        const relPath = relative(sourceDir, fabPath);
        const name = basename(fabPath, '.fab');
        const subdir = dirname(relPath);
        const destDir = join(outDir, subdir);
        const destPath = join(destDir, `${name}.rs`);

        try {
            await mkdir(destDir, { recursive: true });
            const result = await $`${radixBin} emit -t rust ${fabPath}`.quiet();
            await Bun.write(destPath, result.stdout);
            if (verbose) console.log(`    ${relPath} -> ${name}.rs`);
        } catch (err: any) {
            console.error(`    ${relPath} FAILED`);
            if (verbose && err.stderr) console.error(`      ${err.stderr.toString().trim()}`);
            failed++;
        }
    }

    return { total: fabFiles.length, failed };
}

/**
 * Verify .rs files compile with rustc.
 */
interface VerifyResult {
    total: number;
    failed: number;
    errors: Map<string, string[]>;
}

function extractRustcErrors(stderr: string): string[] {
    const codes: string[] = [];
    for (const line of stderr.split('\n')) {
        // Match "error[E0308]: mismatched types" or "warning: unnecessary parentheses ..."
        const codeMatch = line.match(/^(error\[E\d+\]:\s*.+)/);
        if (codeMatch) {
            codes.push(codeMatch[1]);
            continue;
        }
        const warnMatch = line.match(/^(warning:\s*.+)/);
        if (warnMatch) {
            codes.push(warnMatch[1]);
            continue;
        }
        // Match "error: free function without a body" but skip "aborting due to" noise
        const plainMatch = line.match(/^(error:\s*.+)/);
        if (plainMatch && !line.includes('aborting due to')) {
            codes.push(plainMatch[1]);
        }
    }
    return codes;
}

async function verifyRust(dir: string, verbose: boolean): Promise<VerifyResult> {
    const rsFiles = await findFiles(dir, '.rs');
    let failed = 0;
    const tmpDir = join(OPUS, '.rustc-verify');
    await mkdir(tmpDir, { recursive: true });
    const tmpOut = join(tmpDir, 'out.rmeta');
    const errors = new Map<string, string[]>();

    for (const file of rsFiles) {
        try {
            await $`rustc --emit=metadata --edition=2021 -o ${tmpOut} ${file}`.quiet();
        } catch (err: any) {
            const relPath = relative(dir, file);
            console.error(`    ${relPath}: compile error`);
            if (verbose) {
                const errText = err.stderr?.toString() || '';
                const firstLines = errText.split('\n').slice(0, 5).join('\n');
                if (firstLines) console.error(`      ${firstLines}`);
            }
            failed++;

            const errText = err.stderr?.toString() || '';
            for (const code of extractRustcErrors(errText)) {
                const files = errors.get(code) ?? [];
                if (!files.includes(relPath)) files.push(relPath);
                errors.set(code, files);
            }
        }
    }

    return { total: rsFiles.length, failed, errors };
}

function printErrorSummary(errors: Map<string, string[]>) {
    if (errors.size === 0) return;

    const sorted = [...errors.entries()].sort((a, b) => b[1].length - a[1].length);

    console.log('\n    Error summary:');
    for (const [code, files] of sorted) {
        console.log(`      ${String(files.length).padStart(3)}x  ${code}`);
    }
}

async function main() {
    const { verbose, maxStage } = parseArgs();
    const start = performance.now();
    const allResults: StepResult[] = [];

    console.log('Build (radix-rs)\n');

    // =========================================================================
    // PREWORK: Clean opus directory
    // =========================================================================

    await step('clean opus/*', verbose, async () => {
        await rm(OPUS, { recursive: true, force: true });
    });

    const binDir = join(OPUS, 'bin');
    await mkdir(binDir, { recursive: true });

    // =========================================================================
    // STAGE 1: nanus-rs (bootstrap compiler)
    // =========================================================================

    if (maxStage >= 1) {
        console.log('\n--- Stage 1: nanus-rs ---\n');

        const result = await step('cargo build nanus-rs', verbose, async () => {
            if (verbose) {
                await $`cargo build --release --manifest-path ${join(NANUS_RS_DIR, 'Cargo.toml')}`;
            } else {
                await $`cargo build --release --manifest-path ${join(NANUS_RS_DIR, 'Cargo.toml')}`.quiet();
            }
            await copyFile(join(NANUS_RS_DIR, 'target', 'release', 'nanus-rs'), join(binDir, 'nanus-rs'));
        });
        allResults.push(result);

        if (!result.success) {
            console.log(`\nBuild aborted at stage 1.`);
            process.exit(1);
        }
    }

    // =========================================================================
    // STAGE 2: radix-rs (primary compiler)
    // =========================================================================

    if (maxStage >= 2) {
        console.log('\n--- Stage 2: radix-rs ---\n');

        const result = await step('cargo build radix-rs', verbose, async () => {
            if (verbose) {
                await $`cargo build --release --manifest-path ${join(RADIX_RS_DIR, 'Cargo.toml')}`;
            } else {
                await $`cargo build --release --manifest-path ${join(RADIX_RS_DIR, 'Cargo.toml')}`.quiet();
            }
            await copyFile(join(RADIX_RS_DIR, 'target', 'release', 'radix'), join(binDir, 'radix-rs'));
        });
        allResults.push(result);

        if (!result.success) {
            console.log(`\nBuild aborted at stage 2.`);
            process.exit(1);
        }
    }

    // =========================================================================
    // STAGE 3: norma-rs (stdlib crate)
    // =========================================================================

    if (maxStage >= 3) {
        console.log('\n--- Stage 3: norma-rs ---\n');

        const result = await step('cargo build norma-rs', verbose, async () => {
            if (verbose) {
                await $`cargo build --release --manifest-path ${join(NORMA_RS_DIR, 'Cargo.toml')}`;
            } else {
                await $`cargo build --release --manifest-path ${join(NORMA_RS_DIR, 'Cargo.toml')}`.quiet();
            }
        });
        allResults.push(result);

        if (!result.success) {
            console.log(`\nBuild aborted at stage 3.`);
            process.exit(1);
        }
    }

    // =========================================================================
    // STAGE 4: exempla codegen (fab → rs via radix-rs)
    // =========================================================================

    const radixBin = join(binDir, 'radix-rs');
    const exemplaOutDir = join(OPUS, 'radix-rs', 'exempla', 'rs');

    if (maxStage >= 4) {
        console.log('\n--- Stage 4: exempla codegen (fab → rs) ---\n');

        const result = await step('compile exempla → rust', verbose, async () => {
            const { total, failed } = await compileToRust(radixBin, EXEMPLA_DIR, exemplaOutDir, verbose);
            console.log(`    ${total - failed}/${total} compiled`);
        });
        allResults.push(result);
    }

    // =========================================================================
    // STAGE 5: exempla verify (rustc compiles generated Rust)
    // =========================================================================

    if (maxStage >= 5) {
        console.log('\n--- Stage 5: exempla verify (rustc) ---\n');

        const result = await step('verify exempla .rs files', verbose, async () => {
            const { total, failed, errors } = await verifyRust(exemplaOutDir, verbose);
            console.log(`    ${total - failed}/${total} verified`);
            printErrorSummary(errors);
        });
        allResults.push(result);
    }

    // =========================================================================
    // STAGE 6: rivus codegen (fab → rs via radix-rs)
    // =========================================================================

    const rivusOutDir = join(OPUS, 'rivus', 'rs');

    if (maxStage >= 6) {
        console.log('\n--- Stage 6: rivus codegen (fab → rs) ---\n');

        const result = await step('compile rivus → rust', verbose, async () => {
            const { total, failed } = await compileToRust(radixBin, RIVUS_DIR, rivusOutDir, verbose);
            console.log(`    ${total - failed}/${total} compiled`);
        });
        allResults.push(result);
    }

    // =========================================================================
    // SUMMARY
    // =========================================================================

    const elapsed = performance.now() - start;
    const failedSteps = allResults.filter(r => !r.success);

    if (failedSteps.length > 0) {
        console.log(`\nBuild failed (${(elapsed / 1000).toFixed(1)}s) — ${failedSteps.length} step(s) failed:`);
        for (const s of failedSteps) {
            console.log(`  - ${s.name}`);
        }
        process.exit(1);
    } else {
        console.log(`\nBuild complete (${(elapsed / 1000).toFixed(1)}s)`);
    }
}

main().catch(err => {
    console.error(`\nFailed: ${err.message}`);
    process.exit(1);
});
