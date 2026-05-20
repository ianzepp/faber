#!/usr/bin/env bun
/**
 * Compile examples/exempla with the active radix workspace compiler.
 *
 * Usage:
 *   bun scripta/build-exempla.ts
 *   bun scripta/build-exempla.ts -t go
 *   bun scripta/build-exempla.ts -t rust --no-clean
 */

import { mkdir, readdir, rm, stat } from 'fs/promises';
import { basename, dirname, join, relative } from 'path';
import { $ } from 'bun';

const ROOT = join(import.meta.dir, '..');
const MANIFEST = join(ROOT, 'radix', 'Cargo.toml');
const EXEMPLA = join(ROOT, 'examples', 'exempla');
const OUT_ROOT = join(ROOT, 'opus', 'radix', 'exempla');

type Target = 'rust' | 'ts' | 'go' | 'faber';
interface ShellErrorLike {
    stderr?: { toString(): string };
}

const TARGETS = ['rust', 'ts', 'go', 'faber'] as const;
const EXTENSION: Record<Target, string> = {
    rust: 'rs',
    ts: 'ts',
    go: 'go',
    faber: 'fab',
};

function parseArgs(): { target: Target; clean: boolean } {
    const args = process.argv.slice(2);
    let target: Target = 'rust';
    let clean = true;

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === '-t' || arg === '--target') {
            const value = args[++i] as Target | undefined;
            if (!value || !TARGETS.includes(value)) {
                console.error(`Unknown target '${value ?? ''}'. Expected one of: ${TARGETS.join(', ')}`);
                process.exit(1);
            }
            target = value;
            continue;
        }

        if (arg === '--no-clean') {
            clean = false;
            continue;
        }

        console.error(`Unknown argument: ${arg}`);
        process.exit(1);
    }

    return { target, clean };
}

async function findFabFiles(dir: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files: string[] = [];

    for (const entry of entries) {
        const fullPath = join(dir, entry);
        const info = await stat(fullPath);
        if (info.isDirectory()) {
            files.push(...(await findFabFiles(fullPath)));
        } else if (entry.endsWith('.fab')) {
            files.push(fullPath);
        }
    }

    return files.sort();
}

function stderrFromError(err: unknown): string {
    if (typeof err !== 'object' || err === null || !('stderr' in err)) {
        return '';
    }

    return ((err as ShellErrorLike).stderr?.toString() ?? '').trim();
}

async function main() {
    const { target, clean } = parseArgs();
    const outDir = join(OUT_ROOT, target);

    if (clean) {
        await rm(outDir, { recursive: true, force: true });
    }

    const files = await findFabFiles(EXEMPLA);
    let failed = 0;

    for (const file of files) {
        const rel = relative(EXEMPLA, file);
        const output = join(outDir, dirname(rel), `${basename(file, '.fab')}.${EXTENSION[target]}`);

        try {
            await mkdir(dirname(output), { recursive: true });
            const result = await $`cargo run --quiet --manifest-path ${MANIFEST} -p radix -- emit -t ${target} ${file}`.quiet();
            await Bun.write(output, result.stdout);
            console.log(`${rel} -> ${relative(ROOT, output)}`);
        } catch (err: unknown) {
            failed++;
            console.error(`${rel}: failed`);
            const stderr = stderrFromError(err);
            if (stderr) {
                console.error(stderr.split('\n').slice(0, 4).join('\n'));
            }
        }
    }

    if (failed > 0) {
        console.error(`\n${failed}/${files.length} examples failed`);
        process.exit(1);
    }
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
