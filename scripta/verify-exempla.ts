#!/usr/bin/env bun
/**
 * Verify all .fab files in exempla/ compile successfully for a target language.
 *
 * Outputs pass/fail status for each file. Validates output with target-specific
 * tools (eslint for TypeScript, python3 -m py_compile for Python, zig ast-check for Zig).
 *
 * Usage:
 *   bun scripta/verify-exempla.ts              # TypeScript (default)
 *   bun scripta/verify-exempla.ts -t py        # Python
 *   bun scripta/verify-exempla.ts -t zig       # Zig
 */

import { readdir } from 'fs/promises';
import { statSync } from 'fs';
import { join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const EXEMPLA = join(ROOT, 'fons', 'exempla');
const CLI = join(ROOT, 'opus', 'bin', 'rivus');

type Target = 'ts' | 'zig' | 'py';

async function findFabFiles(dir: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        const fullPath = join(dir, entry);
        const stat = statSync(fullPath);

        if (stat.isDirectory()) {
            files.push(...(await findFabFiles(fullPath)));
        } else if (entry.endsWith('.fab')) {
            files.push(fullPath);
        }
    }

    return files.sort();
}

async function validateOutput(code: string, target: Target): Promise<void> {
    switch (target) {
        case 'ts': {
            // WHY: Use eslint to match build:exempla behavior
            // Place temp file in opus/ to use the same eslint config as build output
            const tmpDir = join(ROOT, 'opus', 'exempla', 'ts');
            await $`mkdir -p ${tmpDir}`.quiet();
            const tmpFile = join(tmpDir, '.tmp-verify.ts');
            await Bun.write(tmpFile, code);
            await $`bun x eslint ${tmpFile}`.quiet();
            await $`rm -f ${tmpFile}`.quiet();
            break;
        }
        case 'py': {
            const tmpFile = join(ROOT, '.tmp-verify.py');
            await Bun.write(tmpFile, code);
            await $`python3 -m py_compile ${tmpFile}`.quiet();
            await $`rm -f ${tmpFile}`.quiet();
            break;
        }
        case 'zig': {
            // zig ast-check reads from stdin
            const proc = Bun.spawn(['zig', 'ast-check'], {
                stdin: 'pipe',
                stdout: 'pipe',
                stderr: 'pipe',
            });
            proc.stdin.write(code);
            proc.stdin.end();
            const exitCode = await proc.exited;
            if (exitCode !== 0) {
                const stderr = await new Response(proc.stderr).text();
                throw new Error(stderr);
            }
            break;
        }
    }
}

async function main() {
    const args = process.argv.slice(2);
    const target: Target = args.includes('-t') ? (args[args.indexOf('-t') + 1] as Target) : 'ts';

    const files = await findFabFiles(EXEMPLA);
    let passed = 0;
    let failed = 0;

    for (const file of files) {
        const relativePath = relative(EXEMPLA, file);

        try {
            // Step 1: Run codegen
            const result = await $`bun ${CLI} compile ${file} -t ${target}`.quiet();
            const code = result.stdout.toString();

            // Step 2: Validate output with target-specific tool
            await validateOutput(code, target);

            console.log(`PASS  ${relativePath}`);
            passed++;
        } catch {
            console.log(`FAIL  ${relativePath}`);
            failed++;
        }
    }

    console.log(`\n${passed} passed, ${failed} failed`);
    process.exit(failed > 0 ? 1 : 0);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
