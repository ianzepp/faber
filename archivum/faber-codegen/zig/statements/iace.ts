/**
 * Zig Code Generator - IaceStatement (throw/panic)
 *
 * TRANSFORMS:
 *   iace "message" -> return error.Message
 *   mori "message" -> @panic("message")
 *
 * TARGET: Zig distinguishes recoverable errors (error unions) from fatal panics.
 *   - iace (fatal=false) -> return error.X (recoverable, function needs !T return)
 *   - mori (fatal=true)  -> @panic (unrecoverable crash)
 *
 * WHY: iace converts error message to PascalCase error name for Zig's error set.
 *      Example: "invalid input" -> error.InvalidInput
 */

import type { IaceStatement, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genIaceStatement(node: IaceStatement, g: ZigGenerator): string {
    // mori (fatal=true) -> @panic
    if (node.fatal) {
        return genPanicStatement(node.argument, g);
    }

    // iace (fatal=false) -> return error.X
    return genErrorReturn(node.argument, g);
}

/**
 * Generate @panic for fatal errors (mori).
 */
function genPanicStatement(argument: Expression, g: ZigGenerator): string {
    // Handle string literals
    if (argument.type === 'Literal' && typeof argument.value === 'string') {
        return `${g.ind()}@panic("${argument.value}");`;
    }

    // Handle new Error("msg") - extract message
    if (argument.type === 'NovumExpression' && argument.callee.name === 'Error' && argument.arguments.length > 0) {
        const firstArg = argument.arguments[0]!;
        const msg = firstArg.type !== 'SpreadElement' ? g.genExpression(firstArg) : g.genExpression(argument);
        return `${g.ind()}@panic(${msg});`;
    }

    // Fallback
    return `${g.ind()}@panic(${g.genExpression(argument)});`;
}

/**
 * Generate return error.X for recoverable errors (iace).
 *
 * WHY: Zig error names must be valid identifiers. We convert string messages
 *      to PascalCase error names. For complex expressions, use generic Error.
 */
function genErrorReturn(argument: Expression, g: ZigGenerator): string {
    // Handle string literals -> convert to error name
    if (argument.type === 'Literal' && typeof argument.value === 'string') {
        const errorName = stringToErrorName(argument.value);
        return `${g.ind()}return error.${errorName};`;
    }

    // Handle new Error("msg") - extract message and convert
    if (argument.type === 'NovumExpression' && argument.callee.name === 'Error' && argument.arguments.length > 0) {
        const firstArg = argument.arguments[0]!;
        if (firstArg.type !== 'SpreadElement' && firstArg.type === 'Literal' && typeof firstArg.value === 'string') {
            const errorName = stringToErrorName(firstArg.value);
            return `${g.ind()}return error.${errorName};`;
        }
    }

    // Handle identifier (already an error name)
    if (argument.type === 'Identifier') {
        const errorName = toPascalCase(argument.name);
        return `${g.ind()}return error.${errorName};`;
    }

    // Fallback: use generic error
    return `${g.ind()}return error.Error;`;
}

/**
 * Convert an error message string to a valid Zig error name.
 *
 * WHY: Zig error names must be valid identifiers (PascalCase by convention).
 *      We strip non-alphanumeric characters and convert to PascalCase.
 *
 * Examples:
 *   "invalid input" -> InvalidInput
 *   "timeout" -> Timeout
 *   "404 not found" -> NotFound
 */
function stringToErrorName(message: string): string {
    // Remove non-alphanumeric, split on spaces/underscores/hyphens
    const words = message
        .replace(/[^a-zA-Z0-9\s_-]/g, '')
        .split(/[\s_-]+/)
        .filter(w => w.length > 0);

    if (words.length === 0) {
        return 'Error';
    }

    return words.map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase()).join('');
}

/**
 * Convert a string to PascalCase.
 */
function toPascalCase(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
}
