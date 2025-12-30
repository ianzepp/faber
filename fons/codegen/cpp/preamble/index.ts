/**
 * C++ Preamble Generator
 *
 * Generates #include directives and helper definitions based on features used.
 */

import { readFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Read preamble files once at module load
const SCOPE_GUARD = readFileSync(join(__dirname, 'scopeguard.txt'), 'utf-8');

/**
 * Generate preamble based on includes and flags.
 *
 * @param includes - Set of header files to include
 * @param needsScopeGuard - Whether demum (finally) was used
 * @returns Preamble string
 */
export function genPreamble(includes: Set<string>, needsScopeGuard: boolean): string {
    const parts: string[] = [];

    // Always include standard headers - real applications need these anyway
    includes.add('<algorithm>');
    includes.add('<cstdint>');
    includes.add('<numeric>');
    includes.add('<print>');
    includes.add('<ranges>');
    includes.add('<string>');
    includes.add('<vector>');

    const sorted = Array.from(includes).sort();
    parts.push(sorted.map(h => `#include ${h}`).join('\n'));

    if (needsScopeGuard) {
        parts.push('');
        parts.push(SCOPE_GUARD);
    }

    return parts.join('\n') + '\n';
}
