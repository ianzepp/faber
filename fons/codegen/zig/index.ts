/**
 * Zig Code Generator - Emit systems programming code
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into Zig source code. Unlike
 * the TypeScript generator which preserves JavaScript semantics, this generator
 * adapts Latin constructs to Zig's systems programming model.
 *
 * Key transformations:
 * - Top-level code is wrapped in pub fn main()
 * - Comptime (compile-time) values are hoisted to module scope
 * - Function declarations remain at module scope
 * - Async becomes error union types (!T)
 * - Exceptions map to error returns
 * - scribe() maps to std.debug.print()
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid Zig source code string
 * ERRORS: Throws on unknown AST node types (should never happen with valid AST)
 *
 * TARGET DIFFERENCES
 * ==================
 * Zig uses systems programming semantics:
 * - Compile-time vs runtime execution (comptime)
 * - Explicit types required for var declarations
 * - Error unions instead of exceptions (!T)
 * - No garbage collection (manual memory)
 * - Nullable via optional types (?T)
 * - No arrow functions (use struct { fn ... }.call pattern)
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid Zig
 * INV-2: All Latin type names are mapped to Zig equivalents
 * INV-3: Top-level declarations separate from runtime code
 * INV-4: Main function is only emitted if there are runtime statements
 */

import type { Program, Statement, Expression } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { ZigGenerator } from './generator';
import { genPreamble } from './preamble';

/**
 * Generate Zig source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> Zig source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent only, semicolons always used)
 * @returns Zig source code
 */
export function generateZig(program: Program, options: CodegenOptions = {}): string {
    const g = new ZigGenerator(options.indent ?? '    ');

    // WHY: Zig distinguishes compile-time (const, fn) from runtime (var, statements)
    const topLevel: Statement[] = [];
    const runtime: Statement[] = [];

    for (const stmt of program.body) {
        if (isTopLevelDeclaration(stmt, g)) {
            topLevel.push(stmt);
        } else {
            runtime.push(stmt);
        }
    }

    // First pass: generate top-level code (populates g.features for those statements)
    const topLevelCode = topLevel.map(stmt => g.genStatement(stmt));

    // Build output
    const lines: string[] = [];

    // WHY: Only emit main() if there's runtime code to execute
    if (runtime.length > 0) {
        lines.push('pub fn main() void {');
        g.depth++;

        // WHY: Always provide default allocator in main() so curatorStack['alloc'] is valid.
        // Arena allocator is cheap (just page_allocator wrapper) and Zig optimizes if unused.
        // Using _ = alloc silences "unused variable" warning when no collections are used.
        lines.push(`${g.ind()}var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);`);
        lines.push(`${g.ind()}defer arena.deinit();`);
        lines.push(`${g.ind()}const alloc = arena.allocator();`);
        lines.push(`${g.ind()}_ = alloc;`);
        lines.push('');

        const runtimeCode = runtime.map(stmt => g.genStatement(stmt));
        lines.push(...runtimeCode);
        g.depth--;
        lines.push('}');
    }

    // Prepend preamble and top-level code
    const preamble = genPreamble(g.features);
    const result = [preamble, ...topLevelCode];

    if (topLevelCode.length > 0 && lines.length > 0) {
        result.push('');
    }

    result.push(...lines);

    return result.join('\n');
}

/**
 * Determine if a statement belongs at module scope.
 *
 * WHY: Zig executes at compile-time (comptime) by default.
 *      Only runtime-dependent code goes in main().
 *
 * TARGET: Functions and imports are always top-level in Zig.
 *         Const declarations with comptime values are hoisted.
 */
function isTopLevelDeclaration(node: Statement, g: ZigGenerator): boolean {
    if (node.type === 'FunctioDeclaration') {
        return true;
    }

    if (node.type === 'ImportaDeclaration') {
        return true;
    }

    // WHY: Structs and interfaces are always module-level in Zig
    if (node.type === 'GenusDeclaration' || node.type === 'PactumDeclaration') {
        return true;
    }

    // WHY: Type aliases and discretio (tagged unions) are module-level
    if (node.type === 'TypeAliasDeclaration' || node.type === 'DiscretioDeclaration') {
        return true;
    }

    // WHY: Ordo (enum) declarations are module-level
    if (node.type === 'OrdoDeclaration') {
        return true;
    }

    // WHY: fixum with literal is comptime in Zig
    if (node.type === 'VariaDeclaration' && node.kind === 'fixum') {
        if (node.init && isComptimeValue(node.init)) {
            return true;
        }
    }

    return false;
}

/**
 * Determine if an expression can be evaluated at compile-time.
 *
 * WHY: Zig's comptime system allows literals and constants to be
 *      evaluated during compilation, not runtime.
 */
function isComptimeValue(node: Expression): boolean {
    if (node.type === 'Literal') {
        return true;
    }

    if (node.type === 'TemplateLiteral') {
        return true;
    }

    // WHY: verum, falsum, nihil are Latin keywords for literal values
    if (node.type === 'Identifier') {
        return ['verum', 'falsum', 'nihil'].includes(node.name);
    }

    // WHY: Binary/unary expressions with comptime operands are also comptime
    if (node.type === 'BinaryExpression') {
        return isComptimeValue(node.left) && isComptimeValue(node.right);
    }

    if (node.type === 'UnaryExpression') {
        return isComptimeValue(node.argument);
    }

    return false;
}
