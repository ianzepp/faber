/**
 * NANUS - Minimal Faber Compiler
 *
 * Error formatting and reporting.
 */

import type { Locus } from './ast';

export class CompileError extends Error {
    constructor(
        message: string,
        public locus: Locus,
        public filename: string = '<stdin>',
    ) {
        super(`${filename}:${locus.linea}:${locus.columna}: ${message}`);
        this.name = 'CompileError';
    }
}

export function formatError(err: unknown, source: string, filename: string): string {
    if (!(err instanceof Error)) {
        return String(err);
    }

    // Extract line:column from message if present
    const match = err.message.match(/^(\d+):(\d+): (.*)$/);
    if (!match) {
        return err.message;
    }

    const line = parseInt(match[1], 10);
    const col = parseInt(match[2], 10);
    const msg = match[3];

    const lines = source.split('\n');
    const srcLine = lines[line - 1] ?? '';

    const pointer = ' '.repeat(col - 1) + '^';

    return [
        `${filename}:${line}:${col}: error: ${msg}`,
        '',
        `  ${srcLine}`,
        `  ${pointer}`,
    ].join('\n');
}
