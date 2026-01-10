/**
 * Rust Preamble Generator
 *
 * Generates use statements based on features used.
 */

import type { RequiredFeatures } from '../../types';

/**
 * Generate preamble based on features used.
 *
 * @param features - Feature flags set during codegen traversal
 * @returns Preamble string (empty if no features need setup)
 */
export function genPreamble(features: RequiredFeatures): string {
    const uses: string[] = [];

    if (features.usesRegex) {
        uses.push('use regex::Regex;');
    }

    if (features.stdin) {
        uses.push('use std::io::Read;');
    }

    return uses.length > 0 ? uses.join('\n') + '\n' : '';
}
