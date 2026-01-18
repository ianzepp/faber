/**
 * NANUS - Minimal Faber Compiler
 *
 * A compact TypeScript implementation of the Faber language compiler,
 * supporting only the subset of features needed to compile rivus
 * (the self-hosting Faber compiler).
 *
 * Design goals:
 * - Small: ~3,500 lines vs faber's ~27,000
 * - Clean: Single file per phase, no over-abstraction
 * - Extensible: Add features by extending switch statements
 *
 * Phases:
 * 1. Lexer (lexer.ts): source → tokens
 * 2. Parser (parser.ts): tokens → AST
 * 3. Emitter (emitter.ts): AST → TypeScript
 *
 * Note: No type checker. Rivus source is assumed valid (checked by faber).
 * Nanus is for fast iteration during bootstrap development.
 */

export * from './ast';
export { lex, prepare } from './lexer';
export { parse, Parser } from './parser';
export { emit } from './emitter';
export { CompileError, formatError } from './errors';

import { lex, prepare } from './lexer';
import { parse } from './parser';
import { emit } from './emitter';
import { formatError } from './errors';

export interface CompileOptions {
    filename?: string;
}

export interface CompileResult {
    success: boolean;
    output?: string;
    error?: string;
}

/**
 * Compile Faber source to TypeScript.
 */
export function compile(source: string, options: CompileOptions = {}): CompileResult {
    const filename = options.filename ?? '<stdin>';

    try {
        const tokens = prepare(lex(source, filename));
        const ast = parse(tokens);
        const output = emit(ast);

        return { success: true, output };
    } catch (err) {
        return {
            success: false,
            error: formatError(err, source, filename),
        };
    }
}
