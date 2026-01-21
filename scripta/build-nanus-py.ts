#!/usr/bin/env bun
/**
 * Build standalone nanus-py executable wrapper.
 *
 * Steps:
 *   1. Create a shell script wrapper that invokes the Python module
 */

import { mkdir, writeFile, chmod } from 'fs/promises';
import { join } from 'path';

const ROOT = join(import.meta.dir, '..');
const NANUS_PY = join(ROOT, 'fons', 'nanus-py');

async function main() {
    const start = performance.now();

    const binDir = join(ROOT, 'opus', 'bin');
    await mkdir(binDir, { recursive: true });

    const outExe = join(binDir, 'nanus-py');
    const wrapper = `#!/bin/bash
exec python3 "${NANUS_PY}/__main__.py" "$@"
`;

    await writeFile(outExe, wrapper);
    await chmod(outExe, 0o755);

    const elapsed = performance.now() - start;
    console.log(`Built opus/bin/nanus-py (${elapsed.toFixed(0)}ms)`);
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
