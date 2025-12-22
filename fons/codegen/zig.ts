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
 * The generator uses the same recursive descent pattern as the TypeScript
 * generator but with Zig-specific semantics for types, error handling, and
 * memory management.
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

import type {
    Program,
    Statement,
    Expression,
    ImportDeclaration,
    VariableDeclaration,
    FunctionDeclaration,
    TypeAliasDeclaration,
    IfStatement,
    WhileStatement,
    ForStatement,
    WithStatement,
    SwitchStatement,
    GuardStatement,
    AssertStatement,
    ReturnStatement,
    BlockStatement,
    ThrowStatement,
    ScribeStatement,
    TryStatement,
    ExpressionStatement,
    ArrayExpression,
    ObjectExpression,
    RangeExpression,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrowFunctionExpression,
    AssignmentExpression,
    NewExpression,
    Identifier,
    Literal,
    Parameter,
    TypeAnnotation,
} from '../parser/ast';
import type { CodegenOptions } from './types';
import type { SemanticType } from '../semantic/types';

// =============================================================================
// TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to Zig types.
 *
 * WHY: Zig uses explicit, low-level types instead of JavaScript's dynamic types.
 *      Strings are slices of bytes, numbers are sized integers, void replaces null.
 *
 * TARGET MAPPING:
 * | Latin     | TypeScript | Zig        |
 * |-----------|------------|------------|
 * | textus    | string     | []const u8 |
 * | numerus   | number     | i64        |
 * | bivalens  | boolean    | bool       |
 * | nihil     | null       | void       |
 *
 * CASE: Keys are lowercase. Lookup normalizes input to lowercase for
 *       case-insensitive matching.
 *
 * LIMITATION: Complex types (lista, tabula, etc.) not yet supported in Zig target.
 */
const typeMap: Record<string, string> = {
    textus: '[]const u8',
    numerus: 'i64',
    bivalens: 'bool',
    nihil: 'void',
    vacuum: 'void',
};

// =============================================================================
// MAIN GENERATOR
// =============================================================================

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
    // WHY: 4 spaces is Zig convention
    const indent = options.indent ?? '    ';

    // Track indentation depth for nested blocks
    let depth = 0;

    /**
     * Generate indentation for current depth.
     * WHY: Centralized indentation logic ensures consistent formatting.
     */
    function ind(): string {
        return indent.repeat(depth);
    }

    // ---------------------------------------------------------------------------
    // Top-level
    // ---------------------------------------------------------------------------

    /**
     * Generate top-level program structure.
     *
     * WHY: Zig requires separation between compile-time and runtime code.
     *      Functions and comptime constants go at module scope.
     *      Runtime statements are wrapped in pub fn main().
     */
    function genProgram(node: Program): string {
        const lines: string[] = [];

        // WHY: Many features need std (print, assert, string comparison)
        const needsStd = programNeedsStd(node);

        if (needsStd) {
            lines.push('const std = @import("std");');
            lines.push('');
        }

        // WHY: Zig distinguishes compile-time (const, fn) from runtime (var, statements)
        const topLevel: Statement[] = [];
        const runtime: Statement[] = [];

        for (const stmt of node.body) {
            if (isTopLevelDeclaration(stmt)) {
                topLevel.push(stmt);
            } else {
                runtime.push(stmt);
            }
        }

        lines.push(...topLevel.map(genStatement));

        // WHY: Only emit main() if there's runtime code to execute
        if (runtime.length > 0) {
            if (topLevel.length > 0) {
                lines.push('');
            }

            lines.push('pub fn main() void {');
            depth++;
            lines.push(...runtime.map(genStatement));
            depth--;
            lines.push('}');
        }

        return lines.join('\n');
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
    function isTopLevelDeclaration(node: Statement): boolean {
        if (node.type === 'FunctionDeclaration') {
            return true;
        }

        if (node.type === 'ImportDeclaration') {
            return true;
        }

        // WHY: fixum with literal is comptime in Zig
        if (node.type === 'VariableDeclaration' && node.kind === 'fixum') {
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

        return false;
    }

    /**
     * Check if program needs std library import.
     *
     * WHY: scribe() maps to std.debug.print(), adfirma uses std.debug.assert(),
     *      string switches use std.mem.eql(). Quick check via JSON serialization.
     */
    function programNeedsStd(node: Program): boolean {
        // Always import std for now - it's used for print, assert, string comparison
        return true;
    }

    // ---------------------------------------------------------------------------
    // Statements
    // ---------------------------------------------------------------------------

    /**
     * Generate code for any statement type.
     * WHY: Exhaustive switch ensures all statement types are handled.
     */
    function genStatement(node: Statement): string {
        switch (node.type) {
            case 'ImportDeclaration':
                return genImportDeclaration(node);
            case 'VariableDeclaration':
                return genVariableDeclaration(node);
            case 'FunctionDeclaration':
                return genFunctionDeclaration(node);
            case 'TypeAliasDeclaration':
                return genTypeAliasDeclaration(node);
            case 'IfStatement':
                return genIfStatement(node);
            case 'WhileStatement':
                return genWhileStatement(node);
            case 'ForStatement':
                return genForStatement(node);
            case 'WithStatement':
                return genWithStatement(node);
            case 'SwitchStatement':
                return genSwitchStatement(node);
            case 'GuardStatement':
                return genGuardStatement(node);
            case 'AssertStatement':
                return genAssertStatement(node);
            case 'ReturnStatement':
                return genReturnStatement(node);
            case 'ThrowStatement':
                return genThrowStatement(node);
            case 'ScribeStatement':
                return genScribeStatement(node);
            case 'TryStatement':
                return genTryStatement(node);
            case 'BlockStatement':
                return genBlockStatementContent(node);
            case 'ExpressionStatement':
                return genExpressionStatement(node);
            default:
                throw new Error(`Unknown statement type: ${(node as any).type}`);
        }
    }

    /**
     * Generate import declaration.
     *
     * TRANSFORMS:
     *   ex norma importa * -> const norma = @import("norma");
     *   ex norma importa scribe, lege -> const _norma = @import("norma");
     *                                     const scribe = _norma.scribe;
     *                                     const lege = _norma.lege;
     *
     * TARGET: Zig uses @import() builtin. Specific imports create const bindings.
     */
    function genImportDeclaration(node: ImportDeclaration): string {
        const source = node.source;

        if (node.wildcard) {
            return `${ind()}const ${source} = @import("${source}");`;
        }

        // WHY: Import module once with underscore prefix, then bind specific names
        const lines: string[] = [];
        const modVar = `_${source}`;

        lines.push(`${ind()}const ${modVar} = @import("${source}");`);
        for (const spec of node.specifiers) {
            lines.push(`${ind()}const ${spec.name} = ${modVar}.${spec.name};`);
        }

        return lines.join('\n');
    }

    /**
     * Generate variable declaration.
     *
     * TRANSFORMS:
     *   esto x: Numerus = 5 -> var x: i64 = 5;
     *   fixum y: Textus = "hello" -> const y: []const u8 = "hello";
     *   fixum { a, b } = obj -> const a = obj.a; const b = obj.b;
     *
     * TARGET: Zig requires explicit types for var (mutable) declarations.
     *         Const can infer but we add type for clarity.
     *         Zig doesn't have destructuring, so we expand to multiple statements.
     */
    function genVariableDeclaration(node: VariableDeclaration): string {
        const kind = node.kind === 'esto' ? 'var' : 'const';

        // Handle object pattern destructuring
        if (node.name.type === 'ObjectPattern') {
            const initExpr = node.init ? genExpression(node.init) : 'undefined';
            const lines: string[] = [];

            // Create a temp var to hold the object, then destructure
            const tempVar = `_tmp`;

            lines.push(`${ind()}const ${tempVar} = ${initExpr};`);

            for (const prop of node.name.properties) {
                const key = prop.key.name;
                const localName = prop.value.name;

                lines.push(`${ind()}${kind} ${localName} = ${tempVar}.${key};`);
            }

            return lines.join('\n');
        }

        const name = node.name.name;

        // TARGET: Zig requires explicit types for var, we infer if not provided
        let typeAnno = '';

        if (node.typeAnnotation) {
            typeAnno = `: ${genType(node.typeAnnotation)}`;
        }
        else if (kind === 'var' && node.init) {
            typeAnno = `: ${inferZigType(node.init)}`;
        }

        const init = node.init ? ` = ${genExpression(node.init)}` : ' = undefined';

        return `${ind()}${kind} ${name}${typeAnno}${init};`;
    }

    /**
     * Infer Zig type from expression using resolved semantic type when available.
     *
     * WHY: Zig var declarations require types. We use the resolved type from
     *      semantic analysis when available, falling back to literal inference.
     */
    function inferZigType(node: Expression): string {
        // Use resolved type from semantic analysis if available
        if (node.resolvedType) {
            return semanticTypeToZig(node.resolvedType);
        }

        // Fallback: infer from literals
        if (node.type === 'Literal') {
            if (typeof node.value === 'number') {
                return Number.isInteger(node.value) ? 'i64' : 'f64';
            }

            if (typeof node.value === 'string') {
                return '[]const u8';
            }

            if (typeof node.value === 'boolean') {
                return 'bool';
            }
        }

        if (node.type === 'Identifier') {
            if (node.name === 'verum' || node.name === 'falsum') {
                return 'bool';
            }

            if (node.name === 'nihil') {
                return '?void';
            }
        }

        return 'anytype';
    }

    /**
     * Convert a semantic type to Zig type string.
     */
    function semanticTypeToZig(type: SemanticType): string {
        const nullable = type.nullable ? '?' : '';

        switch (type.kind) {
            case 'primitive':
                switch (type.name) {
                    case 'Textus':
                        return `${nullable}[]const u8`;
                    case 'Numerus':
                        return `${nullable}i64`;
                    case 'Bivalens':
                        return `${nullable}bool`;
                    case 'Nihil':
                        return 'void';
                    case 'Vacuum':
                        return 'void';
                }

                break;
            case 'generic':
                // Generic types not fully supported in Zig target yet
                return 'anytype';
            case 'function':
                return 'anytype';
            case 'union':
                return 'anytype';
            case 'unknown':
                return 'anytype';
            case 'user':
                return type.name;
        }

        return 'anytype';
    }

    /**
     * Check if an expression has a string type.
     */
    function isStringType(node: Expression): boolean {
        if (node.resolvedType?.kind === 'primitive' && node.resolvedType.name === 'Textus') {
            return true;
        }

        if (node.type === 'Literal' && typeof node.value === 'string') {
            return true;
        }

        if (node.type === 'TemplateLiteral') {
            return true;
        }

        return false;
    }

    /**
     * Generate type annotation from Latin type.
     *
     * TRANSFORMS:
     *   Textus -> []const u8
     *   Numerus -> i64
     *   Textus? -> ?[]const u8
     *
     * TARGET: Zig uses ? prefix for optional types, not | null suffix.
     */
    function genType(node: TypeAnnotation): string {
        // Case-insensitive type lookup
        const base = typeMap[node.name.toLowerCase()] ?? node.name;

        if (node.nullable) {
            return `?${base}`;
        }

        return base;
    }

    /**
     * Generate function declaration.
     *
     * TRANSFORMS:
     *   functio salve(nomen: Textus): Nihil -> fn salve(nomen: []const u8) void
     *   futura functio f(): Numerus -> fn f() !i64
     *
     * TARGET: Zig uses fn not function. Async becomes error union (!T).
     */
    function genFunctionDeclaration(node: FunctionDeclaration): string {
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');
        const returnType = node.returnType ? genType(node.returnType) : 'void';

        // TARGET: Async in Zig uses error unions (!T) not async/await
        const retType = node.async ? `!${returnType}` : returnType;

        const body = genBlockStatement(node.body);

        return `${ind()}fn ${name}(${params}) ${retType} ${body}`;
    }

    /**
     * Generate function parameter.
     *
     * TRANSFORMS:
     *   nomen: Textus -> nomen: []const u8
     *
     * TARGET: Zig requires type after parameter name (name: type).
     */
    function genParameter(node: Parameter): string {
        const name = node.name.name;
        const type = node.typeAnnotation ? genType(node.typeAnnotation) : 'anytype';

        return `${name}: ${type}`;
    }

    function genIfStatement(node: IfStatement): string {
        let result = '';

        // Zig doesn't have try/catch like JS, we'll use error handling differently
        // For now, ignore catchClause in Zig output
        result += `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`;

        if (node.alternate) {
            if (node.alternate.type === 'IfStatement') {
                result += ` else ${genIfStatement(node.alternate).trim()}`;
            } else {
                result += ` else ${genBlockStatement(node.alternate)}`;
            }
        }

        return result;
    }

    function genWhileStatement(node: WhileStatement): string {
        const test = genExpression(node.test);
        const body = genBlockStatement(node.body);

        return `${ind()}while (${test}) ${body}`;
    }

    /**
     * Generate for statement.
     *
     * TRANSFORMS:
     *   ex 0..10 pro i { } -> var i: usize = 0; while (i <= 10) : (i += 1) { }
     *   ex items pro item { } -> for (items) |item| { }
     *
     * TARGET: Zig uses for (slice) |item| for iteration over slices.
     *         For ranges, we use while loops since Zig doesn't have range syntax.
     */
    function genForStatement(node: ForStatement): string {
        const varName = node.variable.name;
        const body = genBlockStatement(node.body);

        // Handle range expressions with while loop
        if (node.iterable.type === 'RangeExpression') {
            const range = node.iterable;
            const start = genExpression(range.start);
            const end = genExpression(range.end);

            if (range.step) {
                const step = genExpression(range.step);

                return `${ind()}var ${varName}: usize = ${start}; while (${varName} <= ${end}) : (${varName} += ${step}) ${body}`;
            }

            return `${ind()}var ${varName}: usize = ${start}; while (${varName} <= ${end}) : (${varName} += 1) ${body}`;
        }

        const iterable = genExpression(node.iterable);

        // Zig uses for (slice) |item| syntax
        return `${ind()}for (${iterable}) |${varName}| ${body}`;
    }

    /**
     * Generate with statement.
     *
     * TRANSFORMS:
     *   cum user { nomen = "Marcus" } -> user.nomen = "Marcus";
     *
     * TARGET: Zig doesn't have with-blocks, we expand to member assignments.
     */
    function genWithStatement(node: WithStatement): string {
        const context = genExpression(node.object);
        const lines: string[] = [];

        for (const stmt of node.body.body) {
            if (
                stmt.type === 'ExpressionStatement' &&
                stmt.expression.type === 'AssignmentExpression' &&
                stmt.expression.left.type === 'Identifier'
            ) {
                const prop = stmt.expression.left.name;
                const value = genExpression(stmt.expression.right);
                const op = stmt.expression.operator;

                lines.push(`${ind()}${context}.${prop} ${op} ${value};`);
            }
            else {
                lines.push(genStatement(stmt));
            }
        }

        return lines.join('\n');
    }

    /**
     * Generate switch statement.
     *
     * TRANSFORMS:
     *   elige x { si 1 { a() } si 2 { b() } aliter { c() } }
     *   -> switch (x) { 1 => a(), 2 => b(), else => c() }
     *
     * TARGET: Zig uses switch (x) { value => expr, ... } syntax.
     *         However, Zig cannot switch on strings, so we convert to if-else chain.
     */
    function genSwitchStatement(node: SwitchStatement): string {
        const discriminant = genExpression(node.discriminant);

        // Check if discriminant is a string type - need if-else chain instead
        if (isStringType(node.discriminant)) {
            return genStringSwitchStatement(node, discriminant);
        }

        let result = `${ind()}switch (${discriminant}) {\n`;

        depth++;

        for (const caseNode of node.cases) {
            const test = genExpression(caseNode.test);

            result += `${ind()}${test} => {\n`;
            depth++;

            for (const stmt of caseNode.consequent.body) {
                result += genStatement(stmt) + '\n';
            }

            depth--;
            result += `${ind()}},\n`;
        }

        if (node.defaultCase) {
            result += `${ind()}else => {\n`;
            depth++;

            for (const stmt of node.defaultCase.body) {
                result += genStatement(stmt) + '\n';
            }

            depth--;
            result += `${ind()}},\n`;
        }

        depth--;
        result += `${ind()}}`;

        return result;
    }

    /**
     * Generate string switch as if-else chain with std.mem.eql.
     *
     * TRANSFORMS:
     *   elige status { si "pending" { ... } si "active" { ... } aliter { ... } }
     *   -> if (std.mem.eql(u8, status, "pending")) { ... }
     *      else if (std.mem.eql(u8, status, "active")) { ... }
     *      else { ... }
     *
     * TARGET: Zig cannot switch on []const u8, must use std.mem.eql for comparison.
     */
    function genStringSwitchStatement(node: SwitchStatement, discriminant: string): string {
        let result = '';
        let first = true;

        for (const caseNode of node.cases) {
            const test = genExpression(caseNode.test);
            const prefix = first ? '' : ' else ';
            first = false;

            result += `${ind()}${prefix}if (std.mem.eql(u8, ${discriminant}, ${test})) {\n`;
            depth++;

            for (const stmt of caseNode.consequent.body) {
                result += genStatement(stmt) + '\n';
            }

            depth--;
            result += `${ind()}}`;
        }

        if (node.defaultCase) {
            result += ` else {\n`;
            depth++;

            for (const stmt of node.defaultCase.body) {
                result += genStatement(stmt) + '\n';
            }

            depth--;
            result += `${ind()}}`;
        }

        return result;
    }

    /**
     * Generate guard statement.
     *
     * TRANSFORMS:
     *   custodi { si x == nihil { redde } }
     *   -> if (x == null) { return; }
     *
     * TARGET: Guards are just sequential if statements in Zig too.
     */
    function genGuardStatement(node: GuardStatement): string {
        const lines: string[] = [];

        for (const clause of node.clauses) {
            const test = genExpression(clause.test);
            const body = genBlockStatement(clause.consequent);

            lines.push(`${ind()}if (${test}) ${body}`);
        }

        return lines.join('\n');
    }

    /**
     * Generate assert statement.
     *
     * TRANSFORMS:
     *   adfirma x > 0 -> std.debug.assert(x > 0)
     *   adfirma x > 0, "msg" -> if (!(x > 0)) @panic("msg")
     *
     * TARGET: Zig has std.debug.assert() for assertions.
     *         For custom messages, we use @panic.
     */
    function genAssertStatement(node: AssertStatement): string {
        const test = genExpression(node.test);

        if (node.message) {
            const message = genExpression(node.message);

            return `${ind()}if (!(${test})) @panic(${message});`;
        }

        return `${ind()}std.debug.assert(${test});`;
    }

    /**
     * Generate type alias declaration.
     *
     * TRANSFORMS:
     *   typus ID = Textus -> const ID = []const u8;
     *
     * TARGET: Zig uses const for type aliases.
     */
    function genTypeAliasDeclaration(node: TypeAliasDeclaration): string {
        const name = node.name.name;
        const typeAnno = genType(node.typeAnnotation);

        return `${ind()}const ${name} = ${typeAnno};`;
    }

    function genReturnStatement(node: ReturnStatement): string {
        if (node.argument) {
            return `${ind()}return ${genExpression(node.argument)};`;
        }

        return `${ind()}return;`;
    }

    /**
     * Generate throw statement.
     *
     * TRANSFORMS:
     *   iace "message" -> @panic("message")
     *   iace novum Error("msg") -> @panic("msg")
     *
     * TARGET: Zig doesn't have exceptions. For string errors, use @panic.
     *         For proper error handling, would need error union returns.
     */
    function genThrowStatement(node: ThrowStatement): string {
        // Handle string literals - use @panic
        if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
            return `${ind()}@panic("${node.argument.value}");`;
        }

        // Handle new Error("msg") - extract message and use @panic
        if (node.argument.type === 'NewExpression' &&
            node.argument.callee.name === 'Error' &&
            node.argument.arguments.length > 0) {
            const msg = genExpression(node.argument.arguments[0]);
            return `${ind()}@panic(${msg});`;
        }

        // Fallback - use @panic with expression
        return `${ind()}@panic(${genExpression(node.argument)});`;
    }

    /**
     * Generate scribe statement.
     *
     * TRANSFORMS:
     *   scribe "hello" -> std.debug.print("hello\n", .{});
     *   scribe x       -> std.debug.print("{any}\n", .{x});
     *   scribe a, b    -> std.debug.print("{any} {any}\n", .{a, b});
     *
     * TARGET: Zig's std.debug.print uses format specifiers.
     */
    function genScribeStatement(node: ScribeStatement): string {
        if (node.arguments.length === 0) {
            return `${ind()}std.debug.print("\\n", .{});`;
        }

        // Build format string and args list
        const formatParts: string[] = [];
        const args: string[] = [];

        for (const arg of node.arguments) {
            formatParts.push(getFormatSpecifier(arg));
            args.push(genExpression(arg));
        }

        const format = formatParts.join(' ') + '\\n';

        return `${ind()}std.debug.print("${format}", .{ ${args.join(', ')} });`;
    }

    function genTryStatement(node: TryStatement): string {
        // Zig handles errors differently — this is a simplified mapping
        // Real Zig would use catch |err| { } syntax on expressions
        let result = `${ind()}// try block\n`;

        result += genBlockStatementContent(node.block);

        if (node.handler) {
            result += `\n${ind()}// catch handling would use: catch |${node.handler.param.name}| { ... }`;
        }

        return result;
    }

    function genBlockStatement(node: BlockStatement): string {
        if (node.body.length === 0) {
            return '{}';
        }

        depth++;
        const body = node.body.map(genStatement).join('\n');

        depth--;

        return `{\n${body}\n${ind()}}`;
    }

    function genBlockStatementContent(node: BlockStatement): string {
        return node.body.map(genStatement).join('\n');
    }

    /**
     * Generate expression statement.
     *
     * TARGET: Zig requires expressions to be used or discarded with _ = prefix.
     *         Calls and assignments don't need discard prefix.
     */
    function genExpressionStatement(node: ExpressionStatement): string {
        const expr = genExpression(node.expression);

        // TARGET: Zig assignments and calls are statements, not expressions
        if (
            node.expression.type === 'CallExpression' ||
            node.expression.type === 'AssignmentExpression'
        ) {
            return `${ind()}${expr};`;
        }

        // WHY: Zig requires explicit discard with _ = for unused expression results
        return `${ind()}_ = ${expr};`;
    }

    // ---------------------------------------------------------------------------
    // Expressions
    // ---------------------------------------------------------------------------

    /**
     * Generate code for any expression type.
     * WHY: Exhaustive switch ensures all expression types are handled.
     */
    function genExpression(node: Expression): string {
        switch (node.type) {
            case 'Identifier':
                return genIdentifier(node);
            case 'Literal':
                return genLiteral(node);
            case 'TemplateLiteral':
                // Zig doesn't have template literals, convert to string
                return `"${node.raw.replace(/`/g, '')}"`;
            case 'ArrayExpression':
                return genArrayExpression(node);
            case 'ObjectExpression':
                return genObjectExpression(node);
            case 'RangeExpression':
                return genRangeExpression(node);
            case 'BinaryExpression':
                return genBinaryExpression(node);
            case 'UnaryExpression':
                return genUnaryExpression(node);
            case 'CallExpression':
                return genCallExpression(node);
            case 'MemberExpression':
                return genMemberExpression(node);
            case 'ArrowFunctionExpression':
                return genArrowFunction(node);
            case 'AssignmentExpression':
                return genAssignmentExpression(node);
            case 'AwaitExpression':
                // Zig async is different, use try for error handling
                return `try ${genExpression(node.argument)}`;
            case 'NewExpression':
                return genNewExpression(node);
            case 'ConditionalExpression':
                return `if (${genExpression(node.test)}) ${genExpression(node.consequent)} else ${genExpression(node.alternate)}`;
            default:
                throw new Error(`Unknown expression type: ${(node as any).type}`);
        }
    }

    /**
     * Generate identifier.
     *
     * TRANSFORMS:
     *   verum -> true
     *   falsum -> false
     *   nihil -> null
     *
     * WHY: Latin uses Latin words for boolean/null literals.
     */
    function genIdentifier(node: Identifier): string {
        switch (node.name) {
            case 'verum':
                return 'true';
            case 'falsum':
                return 'false';
            case 'nihil':
                return 'null';
            default:
                return node.name;
        }
    }

    /**
     * Generate literal value.
     *
     * TRANSFORMS:
     *   "hello" -> "hello"
     *   42 -> 42
     *   verum -> true
     *   nihil -> null
     *
     * TARGET: Zig string literals use double quotes, no escaping needed for simple strings.
     */
    function genLiteral(node: Literal): string {
        if (node.value === null) {
            return 'null';
        }

        if (typeof node.value === 'string') {
            return `"${node.value}"`;
        }

        if (typeof node.value === 'boolean') {
            return node.value ? 'true' : 'false';
        }

        return String(node.value);
    }

    /**
     * Generate array literal.
     *
     * TRANSFORMS:
     *   [1, 2, 3] -> .{ 1, 2, 3 }
     *
     * TARGET: Zig uses .{ } for array/tuple literals.
     */
    function genArrayExpression(node: ArrayExpression): string {
        if (node.elements.length === 0) {
            return '.{}';
        }

        const elements = node.elements.map(genExpression).join(', ');

        return `.{ ${elements} }`;
    }

    /**
     * Generate object literal.
     *
     * TRANSFORMS:
     *   { nomen: "Marcus" } -> .{ .nomen = "Marcus" }
     *
     * TARGET: Zig uses .{ .field = value } for struct literals.
     */
    function genObjectExpression(node: ObjectExpression): string {
        if (node.properties.length === 0) {
            return '.{}';
        }

        const props = node.properties.map(prop => {
            const key = prop.key.type === 'Identifier'
                ? prop.key.name
                : String((prop.key as Literal).value);
            const value = genExpression(prop.value);

            return `.${key} = ${value}`;
        });

        return `.{ ${props.join(', ')} }`;
    }

    /**
     * Generate range expression.
     *
     * TRANSFORMS:
     *   0..10 -> (range not directly expressible in Zig)
     *
     * TARGET: Zig doesn't have range literals. This is only used when a range
     *         appears outside a for loop. We generate a comment indicating
     *         the limitation.
     */
    function genRangeExpression(node: RangeExpression): string {
        const start = genExpression(node.start);
        const end = genExpression(node.end);

        // Zig doesn't have standalone range expressions
        // This would need to be converted to a slice or iterator
        return `@compileError("Range expressions must be used in for loops")`;
    }

    /**
     * Generate binary expression.
     *
     * TRANSFORMS:
     *   x + y -> (x + y) for numbers
     *   "a" + "b" -> std.mem.concat(allocator, "a", "b") for strings
     *   a && b -> (a and b)
     *   a || b -> (a or b)
     *
     * TARGET: Zig uses 'and'/'or' keywords not &&/|| operators.
     *         String concatenation requires std.mem.concat or array operations.
     */
    function genBinaryExpression(node: BinaryExpression): string {
        const left = genExpression(node.left);
        const right = genExpression(node.right);

        // Handle string concatenation with + operator
        if (node.operator === '+' && (isStringType(node.left) || isStringType(node.right))) {
            // For compile-time known strings, use ++ operator
            // Note: This only works for comptime-known strings in Zig
            // Runtime string concatenation would require an allocator
            return `(${left} ++ ${right})`;
        }

        const op = mapOperator(node.operator);

        return `(${left} ${op} ${right})`;
    }

    /**
     * Map JavaScript operators to Zig equivalents.
     *
     * TARGET: Zig uses keyword operators for boolean logic.
     */
    function mapOperator(op: string): string {
        switch (op) {
            case '&&':
                return 'and';
            case '||':
                return 'or';
            default:
                return op;
        }
    }

    /**
     * Generate unary expression.
     *
     * TRANSFORMS:
     *   !x -> !x (prefix)
     *   x++ -> x++ (postfix)
     *   nulla x -> x.len == 0
     *   nonnulla x -> x.len > 0
     *
     * TARGET: Zig is statically typed, so we emit .len checks for slices.
     *         For optionals, would need != null pattern.
     */
    function genUnaryExpression(node: UnaryExpression): string {
        const arg = genExpression(node.argument);

        // nulla: check if empty (for slices/arrays)
        if (node.operator === 'nulla') {
            return `(${arg}.len == 0)`;
        }

        // nonnulla: check if non-empty (for slices/arrays)
        if (node.operator === 'nonnulla') {
            return `(${arg}.len > 0)`;
        }

        return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
    }

    /**
     * Generate function call.
     *
     * TRANSFORMS:
     *   scribe("hello") -> std.debug.print("{s}\n", .{"hello"})
     *   scribe(42) -> std.debug.print("{d}\n", .{42})
     *   foo(x, y) -> foo(x, y)
     *
     * TARGET: scribe() is Latin's print function, maps to std.debug.print().
     *         Zig print uses format strings and anonymous tuple syntax (.{...}).
     */
    function genCallExpression(node: CallExpression): string {
        // TARGET: _scribe intrinsic maps to Zig's std.debug.print()
        if (node.callee.type === 'Identifier' && node.callee.name === '_scribe') {
            const args = node.arguments.map(genExpression);
            const formatSpecs = node.arguments.map(arg => getFormatSpecifier(arg));
            const format = formatSpecs.join(' ') + '\\n';

            return `std.debug.print("${format}", .{${args.join(', ')}})`;
        }

        const callee = genExpression(node.callee);
        const args = node.arguments.map(genExpression).join(', ');

        return `${callee}(${args})`;
    }

    /**
     * Get Zig format specifier for an expression based on its resolved type.
     *
     * TARGET: Zig uses {s} for strings, {d} for integers, {any} for unknown.
     */
    function getFormatSpecifier(expr: Expression): string {
        // Use resolved type if available
        if (expr.resolvedType?.kind === 'primitive') {
            switch (expr.resolvedType.name) {
                case 'Textus':
                    return '{s}';
                case 'Numerus':
                    return '{d}';
                case 'Bivalens':
                    return '{}';
                default:
                    return '{any}';
            }
        }

        // Fallback: infer from literal/identifier
        if (expr.type === 'Literal') {
            if (typeof expr.value === 'string') {
                return '{s}';
            }

            if (typeof expr.value === 'number') {
                return '{d}';
            }

            if (typeof expr.value === 'boolean') {
                return '{}';
            }
        }

        if (expr.type === 'Identifier') {
            if (expr.name === 'verum' || expr.name === 'falsum') {
                return '{}';
            }
        }

        if (expr.type === 'TemplateLiteral') {
            return '{s}';
        }

        return '{any}';
    }

    /**
     * Generate member access.
     *
     * TRANSFORMS:
     *   obj.prop -> obj.prop
     *   obj[key] -> obj[key]
     */
    function genMemberExpression(node: MemberExpression): string {
        const obj = genExpression(node.object);

        if (node.computed) {
            return `${obj}[${genExpression(node.property)}]`;
        }

        return `${obj}.${node.property.name}`;
    }

    /**
     * Generate arrow function (emulated).
     *
     * TRANSFORMS:
     *   (x) => x + 1 -> struct { fn call(x: anytype) anytype { return x + 1; } }.call
     *
     * TARGET: Zig doesn't have arrow functions. We emulate with anonymous struct
     *         containing a function. This is a simplification for basic cases.
     *
     * LIMITATION: This pattern doesn't capture closures properly. Full implementation
     *             would require passing captured variables explicitly.
     */
    function genArrowFunction(node: ArrowFunctionExpression): string {
        const params = node.params.map(genParameter).join(', ');

        if (node.body.type === 'BlockStatement') {
            const body = genBlockStatement(node.body);

            return `struct { fn call(${params}) anytype ${body} }.call`;
        }

        const body = genExpression(node.body as Expression);

        return `struct { fn call(${params}) anytype { return ${body}; } }.call`;
    }

    /**
     * Generate assignment expression.
     *
     * TRANSFORMS:
     *   x = 5 -> x = 5
     *   x += 1 -> x += 1
     */
    function genAssignmentExpression(node: AssignmentExpression): string {
        const left = node.left.type === 'Identifier' ? node.left.name : genExpression(node.left);

        return `${left} ${node.operator} ${genExpression(node.right)}`;
    }

    /**
     * Generate new expression.
     *
     * TRANSFORMS:
     *   new Foo(x, y) -> Foo.init(x, y)
     *
     * TARGET: Zig doesn't have 'new' keyword. Idiomatic pattern is Type.init().
     */
    function genNewExpression(node: NewExpression): string {
        const callee = node.callee.name;
        const args = node.arguments.map(genExpression).join(', ');

        return `${callee}.init(${args})`;
    }

    return genProgram(program);
}
