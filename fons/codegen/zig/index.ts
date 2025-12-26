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
    GenusDeclaration,
    PactumDeclaration,
    FieldDeclaration,
    PactumMethod,
    TypeAliasDeclaration,
    IfStatement,
    WhileStatement,
    ForStatement,
    WithStatement,
    SwitchStatement,
    DiscretioDeclaration,
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
    ThisExpression,
    Identifier,
    Literal,
    Parameter,
    TypeAnnotation,
    TypeCastExpression,
    TypeCheckExpression,
    SpreadElement,
    LambdaExpression,
    FacBlockStatement,
    BreakStatement,
    ContinueStatement,
    PraefixumExpression,
    TypeParameterDeclaration,
} from '../../parser/ast';
import type { CodegenOptions } from '../types';
import type { SemanticType } from '../../semantic/types';

// Collection method registries
import { getListaMethod } from './norma/lista';
import { getTabulaMethod } from './norma/tabula';
import { getCopiaMethod } from './norma/copia';

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
 * | Latin     | TypeScript | Zig            |
 * |-----------|------------|----------------|
 * | textus    | string     | []const u8     |
 * | numerus   | number     | i64            |
 * | fractus   | number     | f64            |
 * | decimus   | number     | f128           |
 * | magnus    | bigint     | i128           |
 * | bivalens  | boolean    | bool           |
 * | nihil     | null       | void           |
 * | vacuum    | void       | void           |
 * | objectum  | object     | anytype        |
 * | ignotum   | unknown    | anytype        |
 * | numquam   | never      | noreturn       |
 *
 * CASE: Keys are lowercase. Lookup normalizes input to lowercase for
 *       case-insensitive matching.
 *
 * LIMITATION: Generic types (lista<T>, tabula<K,V>) require special handling
 *             since Zig uses comptime generics differently.
 */
const typeMap: Record<string, string> = {
    // Primitives
    textus: '[]const u8',
    numerus: 'i64',
    fractus: 'f64',
    decimus: 'f128',
    magnus: 'i128',
    bivalens: 'bool',
    nihil: 'void',
    vacuum: 'void',
    octeti: '[]u8',
    // Meta types
    objectum: 'anytype',
    ignotum: 'anytype',
    numquam: 'noreturn',
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

    // Track if program uses collections (need arena allocator)
    let usesCollections = false;

    /**
     * Check if a block contains non-fatal throw statements (iace).
     *
     * WHY: Functions containing `iace` need error union return types (!T).
     *      We recursively search the AST to find any `iace` usage.
     */
    function blockContainsIace(node: BlockStatement): boolean {
        return node.body.some(stmt => statementContainsIace(stmt));
    }

    function statementContainsIace(stmt: Statement): boolean {
        switch (stmt.type) {
            case 'ThrowStatement':
                return !stmt.fatal; // iace has fatal=false, mori has fatal=true

            case 'IfStatement':
                if (blockContainsIace(stmt.consequent)) {
                    return true;
                }
                if (stmt.alternate) {
                    if (stmt.alternate.type === 'BlockStatement') {
                        if (blockContainsIace(stmt.alternate)) {
                            return true;
                        }
                    } else if (stmt.alternate.type === 'IfStatement') {
                        if (statementContainsIace(stmt.alternate)) {
                            return true;
                        }
                    }
                }
                return false;

            case 'WhileStatement':
            case 'ForStatement':
                return blockContainsIace(stmt.body);

            case 'SwitchStatement':
                for (const c of stmt.cases) {
                    if (c.type === 'SwitchCase' && blockContainsIace(c.consequent)) {
                        return true;
                    }
                    if (c.type === 'VariantCase' && blockContainsIace(c.consequent)) {
                        return true;
                    }
                }
                if (stmt.defaultCase && blockContainsIace(stmt.defaultCase)) {
                    return true;
                }
                return false;

            case 'TryStatement':
                if (blockContainsIace(stmt.block)) {
                    return true;
                }
                if (stmt.handler && blockContainsIace(stmt.handler.body)) {
                    return true;
                }
                if (stmt.finalizer && blockContainsIace(stmt.finalizer)) {
                    return true;
                }
                return false;

            case 'FacBlockStatement':
                if (blockContainsIace(stmt.body)) {
                    return true;
                }
                if (stmt.catchClause && blockContainsIace(stmt.catchClause.body)) {
                    return true;
                }
                return false;

            case 'GuardStatement':
                return stmt.clauses.some(c => blockContainsIace(c.consequent));

            case 'WithStatement':
                return blockContainsIace(stmt.body);

            default:
                return false;
        }
    }

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

        // WHY: Check if collections are used to emit arena allocator preamble
        usesCollections = programUsesCollections(node);

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

            // WHY: Emit arena allocator preamble when collections are used
            // This provides a module-level 'alloc' for ArrayList/HashMap operations
            if (usesCollections) {
                lines.push(`${ind()}var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);`);
                lines.push(`${ind()}defer arena.deinit();`);
                lines.push(`${ind()}const alloc = arena.allocator();`);
                lines.push('');
            }

            lines.push(...runtime.map(genStatement));
            depth--;
            lines.push('}');
        }

        return lines.join('\n');
    }

    /**
     * Check if program uses collection types that need arena allocator.
     *
     * WHY: lista, tabula, copia operations in Zig require an allocator.
     *      We detect their use and emit arena preamble in main().
     */
    function programUsesCollections(node: Program): boolean {
        // WHY: Use replacer to handle BigInt values in AST
        const json = JSON.stringify(node, (_key, value) => (typeof value === 'bigint' ? value.toString() : value));
        // Check for collection type annotations or generic names
        return json.includes('"lista"') || json.includes('"tabula"') || json.includes('"copia"');
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

        // WHY: Structs and interfaces are always module-level in Zig
        if (node.type === 'GenusDeclaration' || node.type === 'PactumDeclaration') {
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

        // WHY: Binary/unary expressions with comptime operands are also comptime
        if (node.type === 'BinaryExpression') {
            return isComptimeValue(node.left) && isComptimeValue(node.right);
        }

        if (node.type === 'UnaryExpression') {
            return isComptimeValue(node.argument);
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
            case 'GenusDeclaration':
                return genGenusDeclaration(node);
            case 'PactumDeclaration':
                return genPactumDeclaration(node);
            case 'TypeAliasDeclaration':
                return genTypeAliasDeclaration(node);
            case 'DiscretioDeclaration':
                return genDiscretioDeclaration(node);
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
            case 'FacBlockStatement':
                return genFacBlockStatement(node);
            case 'BreakStatement':
                return genBreakStatement();
            case 'ContinueStatement':
                return genContinueStatement();
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

    // Track module-level constant names to use prefixed identifiers
    // WHY: Zig forbids shadowing - function params can't share names with module consts
    //      Using m_ prefix for module consts avoids collision with param names
    const moduleConstants = new Set<string>();

    /**
     * Generate variable declaration.
     *
     * TRANSFORMS:
     *   varia x: numerus = 5 -> var x: i64 = 5;
     *   fixum y: textus = "hello" -> const m_y: []const u8 = "hello"; (module-level)
     *   fixum { a, b } = obj -> const a = obj.a; const b = obj.b;
     *
     * TARGET: Zig requires explicit types for var (mutable) declarations.
     *         Const can infer but we add type for clarity.
     *         Zig doesn't have destructuring, so we expand to multiple statements.
     *
     * WHY: Module-level constants use m_ prefix to avoid shadowing conflicts
     *      with function parameters. Zig forbids a param named 'x' if there's a
     *      module const 'x', but m_x doesn't conflict.
     */
    function genVariableDeclaration(node: VariableDeclaration): string {
        const kind = node.kind === 'varia' ? 'var' : 'const';

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

        // Check if this is a module-level const (depth 0 means we're at module level)
        const isModuleLevel = depth === 0 && kind === 'const';
        const zigName = isModuleLevel ? `m_${name}` : name;

        // Track module constants for reference generation
        if (isModuleLevel) {
            moduleConstants.add(name);
        }

        // TARGET: Zig requires explicit types for var, we infer if not provided
        let typeAnno = '';

        if (node.typeAnnotation) {
            typeAnno = `: ${genType(node.typeAnnotation)}`;
        } else if (kind === 'var' && node.init) {
            typeAnno = `: ${inferZigType(node.init)}`;
        }

        const init = node.init ? ` = ${genExpression(node.init)}` : ' = undefined';

        return `${ind()}${kind} ${zigName}${typeAnno}${init};`;
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
                    case 'textus':
                        return `${nullable}[]const u8`;
                    case 'numerus':
                        return `${nullable}i64`;
                    case 'bivalens':
                        return `${nullable}bool`;
                    case 'nihil':
                        return 'void';
                    case 'vacuum':
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
        if (node.resolvedType?.kind === 'primitive' && node.resolvedType.name === 'textus') {
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
     *   textus -> []const u8
     *   numerus -> i64
     *   textus? -> ?[]const u8
     *   lista<numerus> -> []i64
     *   lista<textus> -> [][]const u8
     *   tabula<textus, numerus> -> std.StringHashMap(i64)
     *   unio<A, B> -> anytype (Zig doesn't have untagged unions)
     *
     * TARGET: Zig uses ? prefix for optional types, not | null suffix.
     *         Generic collections map to Zig slice/hashmap types.
     *
     * LIMITATION: Zig doesn't support untagged unions like TypeScript's A | B.
     *             Use `discretio` (tagged unions) for Zig target instead.
     *             For unio types, we emit anytype as a fallback.
     */
    function genType(node: TypeAnnotation): string {
        const name = node.name.toLowerCase();

        // Handle union types (unio<A, B>) - Zig doesn't have untagged unions
        // WHY: Zig's type system requires tagged unions (union(enum)) for sum types.
        //      Untagged unions like TS's `A | B` have no Zig equivalent.
        //      Use `discretio` instead for Zig target.
        if (node.union && node.union.length > 0) {
            return 'anytype';
        }

        // Handle generic types
        if (node.typeParameters && node.typeParameters.length > 0) {
            return genGenericType(name, node.typeParameters, node.nullable);
        }

        // Case-insensitive type lookup
        const base = typeMap[name] ?? node.name;

        if (node.nullable) {
            return `?${base}`;
        }

        return base;
    }

    /**
     * Generate generic type (lista<T>, tabula<K,V>, etc).
     *
     * TRANSFORMS:
     *   lista<numerus> -> []i64
     *   lista<textus> -> [][]const u8
     *   tabula<textus, numerus> -> std.StringHashMap(i64)
     *   copia<textus> -> std.StringHashMap(void)
     *
     * TARGET: Zig uses slices for arrays, std library hashmaps for maps/sets.
     *
     * LIMITATION: Zig's std collections require allocators which we don't
     *             manage here. For now, we emit the type signature only.
     */
    function genGenericType(name: string, params: import('../../parser/ast').TypeParameter[], nullable?: boolean): string {
        let result: string;

        switch (name) {
            case 'lista': {
                // lista<T> -> []T
                const elemType = params[0];
                const innerType = elemType && 'name' in elemType ? genType(elemType as TypeAnnotation) : 'anytype';
                result = `[]${innerType}`;
                break;
            }
            case 'tabula': {
                // tabula<K, V> -> depends on key type
                // For string keys: std.StringHashMap(V)
                // For other keys: std.AutoHashMap(K, V)
                const keyType = params[0];
                const valType = params[1];
                const keyZig = keyType && 'name' in keyType ? genType(keyType as TypeAnnotation) : 'anytype';
                const valZig = valType && 'name' in valType ? genType(valType as TypeAnnotation) : 'anytype';

                if (keyZig === '[]const u8') {
                    result = `std.StringHashMap(${valZig})`;
                } else {
                    result = `std.AutoHashMap(${keyZig}, ${valZig})`;
                }
                break;
            }
            case 'copia': {
                // copia<T> -> set, implemented as map with void values
                const elemType = params[0];
                const innerType = elemType && 'name' in elemType ? genType(elemType as TypeAnnotation) : 'anytype';

                if (innerType === '[]const u8') {
                    result = 'std.StringHashMap(void)';
                } else {
                    result = `std.AutoHashMap(${innerType}, void)`;
                }
                break;
            }
            case 'promissum': {
                // promissum<T> -> Zig doesn't have promises, use error union
                const innerType = params[0];
                const zigType = innerType && 'name' in innerType ? genType(innerType as TypeAnnotation) : 'anytype';
                result = `!${zigType}`;
                break;
            }
            default:
                // Unknown generic - pass through
                result = name;
        }

        if (nullable) {
            return `?${result}`;
        }

        return result;
    }

    /**
     * Generate function declaration.
     *
     * TRANSFORMS:
     *   functio salve(nomen: textus): nihil -> fn salve(nomen: []const u8) void
     *   futura functio f(): numerus -> fn f() !i64
     *   functio max(prae typus T, T a, T b) -> T -> fn max(comptime T: type, a: T, b: T) T
     *
     * TARGET: Zig uses fn not function. Async becomes error union (!T).
     *         Type parameters become comptime T: type parameters.
     *
     * EDGE: anytype is not valid as a return type in Zig. If the return type
     *       resolves to anytype (from objectum/ignotum), generate a compile error.
     */
    function genFunctionDeclaration(node: FunctionDeclaration): string {
        const name = node.name.name;

        // Generate type parameters as comptime T: type
        const typeParams = node.typeParams?.map(genTypeParameter) ?? [];

        // Generate regular parameters
        const regularParams = node.params.map(genParameter);

        // Combine: type params first, then regular params
        const allParams = [...typeParams, ...regularParams].join(', ');

        const returnType = node.returnType ? genType(node.returnType) : 'void';

        // EDGE: anytype is not valid as return type in Zig
        if (returnType === 'anytype') {
            const body = `{ @compileError("Function '${name}' returns objectum/ignotum which has no Zig equivalent - use a concrete type"); }`;
            return `${ind()}fn ${name}(${allParams}) void ${body}`;
        }

        // WHY: Functions containing `iace` need error union return type
        // TARGET: Async in Zig also uses error unions (!T)
        const hasIace = blockContainsIace(node.body);
        const needsErrorUnion = node.async || hasIace;
        const retType = needsErrorUnion ? `!${returnType}` : returnType;

        const body = genBlockStatement(node.body);

        return `${ind()}fn ${name}(${allParams}) ${retType} ${body}`;
    }

    /**
     * Generate type parameter declaration.
     *
     * TRANSFORMS:
     *   prae typus T -> comptime T: type
     *
     * TARGET: Zig uses comptime T: type for compile-time type parameters.
     */
    function genTypeParameter(node: TypeParameterDeclaration): string {
        return `comptime ${node.name.name}: type`;
    }

    /**
     * Generate function parameter.
     *
     * TRANSFORMS:
     *   nomen: textus -> nomen: []const u8
     *   de textus source -> source: []const u8 (borrowed, read-only)
     *   in lista<T> items -> items: *std.ArrayList(T) (mutable pointer)
     *
     * TARGET: Zig requires type after parameter name (name: type).
     *         Prepositions (de/in) modify the type for ownership semantics.
     */
    function genParameter(node: Parameter): string {
        const name = node.name.name;
        const preposition = node.preposition;
        const type = node.typeAnnotation ? genTypeWithPreposition(node.typeAnnotation, preposition) : 'anytype';

        return `${name}: ${type}`;
    }

    /**
     * Generate type with ownership preposition applied.
     *
     * TRANSFORMS:
     *   (none) textus -> []const u8 (owned, uses module allocator if needed)
     *   de textus -> []const u8 (borrowed slice, read-only)
     *   in textus -> *[]u8 (mutable pointer to slice)
     *
     *   (none) numerus -> i64 (owned value)
     *   de numerus -> *const i64 (borrowed, read-only pointer)
     *   in numerus -> *i64 (mutable pointer)
     *
     *   (none) lista<T> -> std.ArrayList(T) (owned)
     *   de lista<T> -> []const T (borrowed slice of items)
     *   in lista<T> -> *std.ArrayList(T) (mutable pointer)
     *
     * WHY: Latin prepositions encode ownership for systems targets:
     *      de = "from/concerning" = borrowed, read-only
     *      in = "into" = mutable borrow, will be modified
     */
    function genTypeWithPreposition(node: TypeAnnotation, preposition?: string): string {
        const baseType = genType(node);
        const typeName = node.name.toLowerCase();

        if (!preposition) {
            // No preposition = owned value (current behavior)
            return baseType;
        }

        if (preposition === 'de') {
            // Borrowed, read-only
            return genBorrowedType(typeName, baseType, node);
        }

        if (preposition === 'in') {
            // Mutable borrow
            return genMutableType(typeName, baseType, node);
        }

        // Unknown preposition, return base type
        return baseType;
    }

    /**
     * Generate borrowed (read-only) type for 'de' preposition.
     *
     * WHY: 'de' means "from/concerning" - the parameter is borrowed, not owned.
     *      For slices (textus, lista), we keep the slice type.
     *      For values (numerus, bivalens), we use *const T pointer.
     */
    function genBorrowedType(typeName: string, baseType: string, node: TypeAnnotation): string {
        // Strings are already slices - no change needed
        if (typeName === 'textus') {
            return baseType; // []const u8
        }

        // Lista becomes a const slice of the element type
        if (typeName === 'lista' && node.typeParameters && node.typeParameters.length > 0) {
            const elemType = node.typeParameters[0]!;
            if (elemType.type === 'TypeAnnotation') {
                const innerType = genType(elemType);
                return `[]const ${innerType}`;
            }
        }

        // Tabula/Copia - const pointer to the map
        if (typeName === 'tabula' || typeName === 'copia') {
            return `*const ${baseType}`;
        }

        // Primitives (numerus, bivalens, etc.) - const pointer
        if (typeMap[typeName]) {
            return `*const ${baseType}`;
        }

        // User-defined types - const pointer
        return `*const ${baseType}`;
    }

    /**
     * Generate mutable type for 'in' preposition.
     *
     * WHY: 'in' means "into" - the parameter will be modified.
     *      All types become mutable pointers.
     */
    function genMutableType(typeName: string, baseType: string, node: TypeAnnotation): string {
        // Strings with 'in' - mutable pointer to mutable slice
        if (typeName === 'textus') {
            return `*[]u8`;
        }

        // Lista with 'in' - mutable pointer to ArrayList (not slice)
        // WHY: To mutate a list, we need access to the ArrayList, not just its items
        if (typeName === 'lista' && node.typeParameters && node.typeParameters.length > 0) {
            const elemType = node.typeParameters[0]!;
            if (elemType.type === 'TypeAnnotation') {
                const innerType = genType(elemType);
                return `*std.ArrayList(${innerType})`;
            }
        }

        // Everything else - mutable pointer
        return `*${baseType}`;
    }

    // -------------------------------------------------------------------------
    // OOP: Genus (Struct) and Pactum (Interface)
    // -------------------------------------------------------------------------

    /**
     * Generate genus declaration as a Zig struct.
     *
     * TRANSFORMS:
     *   genus persona { textus nomen: "X" }
     *   -> const persona = struct {
     *          nomen: []const u8 = "X",
     *          const Self = @This();
     *          pub fn init(overrides: anytype) Self { ... }
     *      };
     *
     * TARGET: Zig uses structs with methods. The init() pattern replaces constructors.
     *         Self = @This() enables methods to reference their own type.
     */
    function genGenusDeclaration(node: GenusDeclaration): string {
        const name = node.name.name;
        const lines: string[] = [];

        lines.push(`${ind()}const ${name} = struct {`);
        depth++;

        // Fields with defaults
        for (const field of node.fields) {
            lines.push(genFieldDeclaration(field));
        }

        // Self type reference (needed for init, methods)
        // WHY: init() uses Self for return type and self variable
        if (node.fields.length > 0 || node.methods.length > 0 || node.constructor) {
            if (node.fields.length > 0) {
                lines.push('');
            }
            lines.push(`${ind()}const Self = @This();`);
        }

        // Auto-merge init function
        if (node.fields.length > 0) {
            lines.push('');
            lines.push(genStructInit(node));
        }

        // User's creo as private post-init method
        if (node.constructor) {
            lines.push('');
            lines.push(genCreoMethod(node.constructor));
        }

        // Methods
        if (node.methods.length > 0) {
            lines.push('');
            for (const method of node.methods) {
                lines.push(genStructMethod(method));
            }
        }

        depth--;
        lines.push(`${ind()}};`);

        return lines.join('\n');
    }

    /**
     * Generate field declaration within a struct.
     *
     * TRANSFORMS:
     *   textus nomen: "X" -> nomen: []const u8 = "X",
     *   numerus aetas -> aetas: i64 = undefined,
     */
    function genFieldDeclaration(node: FieldDeclaration): string {
        const name = node.name.name;
        const type = genType(node.fieldType);
        const init = node.init ? genExpression(node.init) : 'undefined';

        // TARGET: Zig struct fields use = for defaults, end with comma
        return `${ind()}${name}: ${type} = ${init},`;
    }

    /**
     * Generate init function for struct (auto-merge constructor).
     *
     * TRANSFORMS:
     *   genus persona { textus nomen: "X", numerus aetas: 0 }
     *   -> pub fn init(overrides: anytype) Self {
     *          var self = Self{
     *              .nomen = if (@hasField(@TypeOf(overrides), "nomen")) overrides.nomen else "X",
     *              .aetas = if (@hasField(@TypeOf(overrides), "aetas")) overrides.aetas else 0,
     *          };
     *          return self;
     *      }
     *
     * TARGET: Uses comptime @hasField to check if override was provided.
     */
    function genStructInit(node: GenusDeclaration): string {
        const lines: string[] = [];

        lines.push(`${ind()}pub fn init(overrides: anytype) Self {`);
        depth++;

        // WHY: Use 'var' if creo mutates self, otherwise 'const' to satisfy Zig linter
        const selfKind = node.constructor ? 'var' : 'const';
        lines.push(`${ind()}${selfKind} self = Self{`);
        depth++;

        for (const field of node.fields) {
            const name = field.name.name;
            const defaultVal = field.init ? genExpression(field.init) : 'undefined';

            lines.push(`${ind()}.${name} = if (@hasField(@TypeOf(overrides), "${name}")) overrides.${name} else ${defaultVal},`);
        }

        depth--;
        lines.push(`${ind()}};`);

        // Call creo if defined
        if (node.constructor) {
            lines.push(`${ind()}self.creo();`);
        }

        lines.push(`${ind()}return self;`);

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate user's creo as a method.
     *
     * TRANSFORMS:
     *   functio creo() { si ego.aetas < 0 { ego.aetas = 0 } }
     *   -> fn creo(self: *Self) void { if (self.aetas < 0) { self.aetas = 0; } }
     */
    function genCreoMethod(node: FunctionDeclaration): string {
        const lines: string[] = [];

        lines.push(`${ind()}fn creo(self: *Self) void {`);
        depth++;

        // Generate body, replacing ego with self
        for (const stmt of node.body.body) {
            const code = genStatement(stmt);
            lines.push(code.replace(/\bego\b/g, 'self'));
        }

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate method declaration within a struct.
     *
     * TRANSFORMS:
     *   functio saluta() -> textus { redde ego.nomen }
     *   -> pub fn saluta(self: *const Self) []const u8 { return self.nomen; }
     *
     * EDGE: anytype is not valid as a return type in Zig.
     */
    function genStructMethod(node: FunctionDeclaration): string {
        const name = node.name.name;
        const params = node.params.map(genParameter);
        const returnType = node.returnType ? genType(node.returnType) : 'void';

        // Add self parameter - use *Self for methods that might mutate
        const selfParam = 'self: *const Self';
        const allParams = [selfParam, ...params].join(', ');

        // EDGE: anytype is not valid as return type in Zig
        if (returnType === 'anytype') {
            return `${ind()}pub fn ${name}(${allParams}) void { @compileError("Method '${name}' returns objectum/ignotum which has no Zig equivalent - use a concrete type"); }`;
        }

        const lines: string[] = [];

        lines.push(`${ind()}pub fn ${name}(${allParams}) ${returnType} {`);
        depth++;

        // Generate body, replacing ego with self
        for (const stmt of node.body.body) {
            const code = genStatement(stmt);
            lines.push(code.replace(/\bego\b/g, 'self'));
        }

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate pactum declaration.
     *
     * TARGET: Zig doesn't have interfaces. We emit a comment documenting the contract
     *         and rely on duck typing / comptime checks.
     *
     * TRANSFORMS:
     *   pactum iterabilis { functio sequens() -> textus? }
     *   -> // pactum iterabilis: requires fn sequens() ?[]const u8
     */
    function genPactumDeclaration(node: PactumDeclaration): string {
        const name = node.name.name;
        const lines: string[] = [];

        lines.push(`${ind()}// pactum ${name}: interface contract (Zig uses duck typing)`);

        for (const method of node.methods) {
            const methodName = method.name.name;
            const params = method.params.map(genParameter).join(', ');
            const returnType = method.returnType ? genType(method.returnType) : 'void';

            lines.push(`${ind()}//   requires fn ${methodName}(${params}) ${returnType}`);
        }

        return lines.join('\n');
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
     *   ex 0..10 pro i { } -> var i: usize = 0; while (i < 10) : (i += 1) { }
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
            const cmp = range.inclusive ? '<=' : '<';

            if (range.step) {
                const step = genExpression(range.step);

                return `${ind()}var ${varName}: usize = ${start}; while (${varName} ${cmp} ${end}) : (${varName} += ${step}) ${body}`;
            }

            return `${ind()}var ${varName}: usize = ${start}; while (${varName} ${cmp} ${end}) : (${varName} += 1) ${body}`;
        }

        const iterable = genExpression(node.iterable);

        // Zig uses for (slice) |item| syntax
        return `${ind()}for (${iterable}) |${varName}| ${body}`;
    }

    /**
     * Generate with statement (mutation block).
     *
     * TRANSFORMS:
     *   in user { nomen = "Marcus" } -> user.nomen = "Marcus";
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
            } else {
                lines.push(genStatement(stmt));
            }
        }

        return lines.join('\n');
    }

    /**
     * Generate switch statement as if-else chain.
     *
     * TRANSFORMS (value matching):
     *   elige x { si 1 { a() } si 2 { b() } aliter { c() } }
     *   -> if (x == 1) { a(); } else if (x == 2) { b(); } else { c(); }
     *
     *   elige status { si "pending" { ... } aliter { ... } }
     *   -> if (std.mem.eql(u8, status, "pending")) { ... } else { ... }
     *
     * TRANSFORMS (variant matching):
     *   elige event { ex Click pro x, y { use(x, y) } ex Quit { exit() } }
     *   -> switch (event) { .click => |c| { use(c.x, c.y); }, .quit => { exit(); } }
     *
     * WHY: For value matching, use if-else chains instead of switch statements.
     *      For variant matching, Zig's switch on tagged unions is idiomatic.
     */
    function genSwitchStatement(node: SwitchStatement): string {
        const discriminant = genExpression(node.discriminant);

        // Check if we have any variant cases (pattern matching on discretio)
        const hasVariantCases = node.cases.some(c => c.type === 'VariantCase');

        if (hasVariantCases) {
            // Variant matching: use native switch on tagged union
            let result = `${ind()}switch (${discriminant}) {\n`;
            depth++;

            for (const caseNode of node.cases) {
                if (caseNode.type === 'VariantCase') {
                    const variantName = caseNode.variant.name.toLowerCase();

                    if (caseNode.bindings.length > 0) {
                        // Capture payload: .click => |c| { ... }
                        result += `${ind()}.${variantName} => |payload| {\n`;
                        depth++;

                        // Bind individual fields
                        for (const binding of caseNode.bindings) {
                            result += `${ind()}const ${binding.name} = payload.${binding.name};\n`;
                        }

                        for (const stmt of caseNode.consequent.body) {
                            result += genStatement(stmt) + '\n';
                        }

                        depth--;
                        result += `${ind()}},\n`;
                    } else {
                        // No payload: .quit => { ... }
                        result += `${ind()}.${variantName} => {\n`;
                        depth++;

                        for (const stmt of caseNode.consequent.body) {
                            result += genStatement(stmt) + '\n';
                        }

                        depth--;
                        result += `${ind()}},\n`;
                    }
                } else {
                    // Mixed value case in variant switch - not typical
                    result += `${ind()}// TODO: Mixed value case not supported in Zig variant switch\n`;
                }
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

        // Value matching: use if-else chain
        // Check if comparing strings (need std.mem.eql)
        const hasStringCase = node.cases.some(c => c.type === 'SwitchCase' && c.test.type === 'Literal' && typeof c.test.value === 'string');
        const isString = isStringType(node.discriminant) || hasStringCase;

        let result = '';
        let first = true;

        for (const caseNode of node.cases) {
            if (caseNode.type !== 'SwitchCase') {
                continue;
            }

            const test = genExpression(caseNode.test);
            const prefix = first ? '' : ' else ';
            first = false;

            // Use std.mem.eql for strings, == for everything else
            const condition = isString ? `std.mem.eql(u8, ${discriminant}, ${test})` : `(${discriminant} == ${test})`;

            result += `${ind()}${prefix}if (${condition}) {\n`;
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
     *   typus ID = textus -> const ID = []const u8;
     *
     * TARGET: Zig uses const for type aliases.
     */
    function genTypeAliasDeclaration(node: TypeAliasDeclaration): string {
        const name = node.name.name;
        const typeAnno = genType(node.typeAnnotation);

        return `${ind()}const ${name} = ${typeAnno};`;
    }

    /**
     * Generate discretio (tagged union) declaration.
     *
     * TRANSFORMS:
     *   discretio Event { Click { numerus x, numerus y }, Quit }
     *   -> const Event = union(enum) { click: struct { x: i64, y: i64 }, quit };
     *
     * TARGET: Zig has native tagged unions via union(enum).
     */
    function genDiscretioDeclaration(node: DiscretioDeclaration): string {
        const name = node.name.name;
        const lines: string[] = [];

        lines.push(`${ind()}const ${name} = union(enum) {`);
        depth++;

        for (const variant of node.variants) {
            const variantName = variant.name.name.toLowerCase();

            if (variant.fields.length === 0) {
                // Unit variant
                lines.push(`${ind()}${variantName},`);
            } else {
                // Variant with payload - use anonymous struct
                const fields = variant.fields
                    .map((field: (typeof variant.fields)[0]) => {
                        const fieldName = field.name.name;
                        const fieldType = genType(field.fieldType);
                        return `${fieldName}: ${fieldType}`;
                    })
                    .join(', ');

                lines.push(`${ind()}${variantName}: struct { ${fields} },`);
            }
        }

        depth--;
        lines.push(`${ind()}};`);

        return lines.join('\n');
    }

    function genReturnStatement(node: ReturnStatement): string {
        if (node.argument) {
            return `${ind()}return ${genExpression(node.argument)};`;
        }

        return `${ind()}return;`;
    }

    /**
     * Generate break statement.
     *
     * TRANSFORMS:
     *   rumpe -> break
     *
     * TARGET: Zig uses 'break' keyword, same as most languages.
     */
    function genBreakStatement(): string {
        return `${ind()}break;`;
    }

    /**
     * Generate continue statement.
     *
     * TRANSFORMS:
     *   perge -> continue
     *
     * TARGET: Zig uses 'continue' keyword, same as most languages.
     */
    function genContinueStatement(): string {
        return `${ind()}continue;`;
    }

    /**
     * Generate throw/panic statement.
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
    function genThrowStatement(node: ThrowStatement): string {
        // mori (fatal=true) -> @panic
        if (node.fatal) {
            return genPanicStatement(node.argument);
        }

        // iace (fatal=false) -> return error.X
        return genErrorReturn(node.argument);
    }

    /**
     * Generate @panic for fatal errors (mori).
     */
    function genPanicStatement(argument: Expression): string {
        // Handle string literals
        if (argument.type === 'Literal' && typeof argument.value === 'string') {
            return `${ind()}@panic("${argument.value}");`;
        }

        // Handle new Error("msg") - extract message
        if (argument.type === 'NewExpression' && argument.callee.name === 'Error' && argument.arguments.length > 0) {
            const firstArg = argument.arguments[0]!;
            const msg = firstArg.type !== 'SpreadElement' ? genExpression(firstArg) : genExpression(argument);
            return `${ind()}@panic(${msg});`;
        }

        // Fallback
        return `${ind()}@panic(${genExpression(argument)});`;
    }

    /**
     * Generate return error.X for recoverable errors (iace).
     *
     * WHY: Zig error names must be valid identifiers. We convert string messages
     *      to PascalCase error names. For complex expressions, use generic Error.
     */
    function genErrorReturn(argument: Expression): string {
        // Handle string literals -> convert to error name
        if (argument.type === 'Literal' && typeof argument.value === 'string') {
            const errorName = stringToErrorName(argument.value);
            return `${ind()}return error.${errorName};`;
        }

        // Handle new Error("msg") - extract message and convert
        if (argument.type === 'NewExpression' && argument.callee.name === 'Error' && argument.arguments.length > 0) {
            const firstArg = argument.arguments[0]!;
            if (firstArg.type !== 'SpreadElement' && firstArg.type === 'Literal' && typeof firstArg.value === 'string') {
                const errorName = stringToErrorName(firstArg.value);
                return `${ind()}return error.${errorName};`;
            }
        }

        // Handle identifier (already an error name)
        if (argument.type === 'Identifier') {
            const errorName = toPascalCase(argument.name);
            return `${ind()}return error.${errorName};`;
        }

        // Fallback: use generic error
        return `${ind()}return error.Error;`;
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

    /**
     * Generate scribe/vide/mone statement.
     *
     * TRANSFORMS:
     *   scribe "hello" -> std.debug.print("hello\n", .{});
     *   vide x         -> std.debug.print("[DEBUG] {any}\n", .{x});
     *   mone "oops"    -> std.debug.print("[WARN] oops\n", .{});
     *
     * TARGET: Zig's std.debug.print with level prefixes for debug/warn.
     */
    function genScribeStatement(node: ScribeStatement): string {
        const prefix = node.level === 'debug' ? '[DEBUG] ' : node.level === 'warn' ? '[WARN] ' : '';

        if (node.arguments.length === 0) {
            return `${ind()}std.debug.print("${prefix}\\n", .{});`;
        }

        // Build format string and args list
        const formatParts: string[] = [];
        const args: string[] = [];

        for (const arg of node.arguments) {
            formatParts.push(getFormatSpecifier(arg));
            args.push(genExpression(arg));
        }

        const format = prefix + formatParts.join(' ') + '\\n';

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

    /**
     * Generate fac block statement.
     *
     * TRANSFORMS:
     *   fac { riskyOperation() } -> { riskyOperation(); }
     *   fac { op() } cape err { handle(err) } -> // fac block with error capture
     *
     * TARGET: Zig uses error unions and catch syntax for error handling,
     *         not try-catch blocks. fac without cape is just a scope block.
     *         fac with cape requires different patterns (catch on expressions).
     */
    function genFacBlockStatement(node: FacBlockStatement): string {
        if (node.catchClause) {
            // With cape, emit as commented block since Zig catch works differently
            // Real implementation would need to use catch on error-returning expressions
            let result = `${ind()}// fac block with catch - Zig uses 'catch' on error union expressions\n`;
            result += genBlockStatementContent(node.body);

            result += `\n${ind()}// catch clause for: ${node.catchClause.param.name}`;
            // The catch body is not directly usable without error union context

            return result;
        }

        // Without cape, just emit the block
        return genBlockStatementContent(node.body);
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
        if (node.expression.type === 'CallExpression' || node.expression.type === 'AssignmentExpression') {
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
            case 'ThisExpression':
                // TARGET: ego (this) becomes self in Zig struct methods
                return 'self';
            case 'NewExpression':
                return genNewExpression(node);
            case 'ConditionalExpression':
                return `if (${genExpression(node.test)}) ${genExpression(node.consequent)} else ${genExpression(node.alternate)}`;
            case 'TypeCastExpression':
                return genTypeCastExpression(node);
            case 'TypeCheckExpression':
                return genTypeCheckExpression(node);
            case 'LambdaExpression':
                return genLambdaExpression(node);
            case 'PraefixumExpression':
                return genPraefixumExpression(node);
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
                // Use m_ prefix for module constants to match declaration
                if (moduleConstants.has(node.name)) {
                    return `m_${node.name}`;
                }
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
            // Escape special characters for Zig string literals
            const escaped = node.value.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n').replace(/\r/g, '\\r').replace(/\t/g, '\\t');
            return `"${escaped}"`;
        }

        if (typeof node.value === 'boolean') {
            return node.value ? 'true' : 'false';
        }

        // WHY: Zig comptime_int is arbitrary precision, no 'n' suffix needed
        if (typeof node.value === 'bigint') {
            // Strip 'n' suffix from raw (e.g., "0xFFn" -> "0xFF")
            return node.raw.replace(/n$/, '');
        }

        // WHY: Use raw to preserve original format (hex: 0xFF, decimal: 123)
        if (typeof node.value === 'number') {
            return node.raw;
        }

        return String(node.value);
    }

    /**
     * Generate array literal.
     *
     * TRANSFORMS:
     *   [1, 2, 3] -> .{ 1, 2, 3 }
     *   [sparge a, sparge b] -> a ++ b (comptime only)
     *
     * TARGET: Zig uses .{ } for array/tuple literals.
     *         Spread requires ++ concatenation (comptime only).
     *
     * LIMITATION: Zig array spread only works at comptime. Runtime spread
     *             would require allocators and explicit memory management.
     */
    function genArrayExpression(node: ArrayExpression): string {
        if (node.elements.length === 0) {
            return '.{}';
        }

        // Check if any elements are spread
        const hasSpread = node.elements.some(el => el.type === 'SpreadElement');

        if (hasSpread) {
            // WHY: Zig doesn't have spread syntax. For comptime arrays, we use ++
            // This is a simplification - runtime would need allocator
            const parts = node.elements.map(el => {
                if (el.type === 'SpreadElement') {
                    return genExpression(el.argument);
                }
                // Wrap non-spread elements in array literal
                return `.{ ${genExpression(el)} }`;
            });

            return parts.join(' ++ ');
        }

        const elements = node.elements.map(el => genExpression(el as Expression)).join(', ');

        return `.{ ${elements} }`;
    }

    /**
     * Generate object literal.
     *
     * TRANSFORMS:
     *   { nomen: "Marcus" } -> .{ .nomen = "Marcus" }
     *   { sparge defaults, x: 1 } -> (struct merge not directly supported)
     *
     * TARGET: Zig uses .{ .field = value } for struct literals.
     *
     * LIMITATION: Zig doesn't have object spread. We skip spread elements
     *             since they cannot be represented in Zig struct literals.
     *
     * EDGE: 'error' is a reserved keyword in Zig, rename to 'err'.
     */
    function genObjectExpression(node: ObjectExpression): string {
        if (node.properties.length === 0) {
            return '.{}';
        }

        // Filter out spread elements - they can't be represented in Zig
        const props = node.properties
            .filter(prop => prop.type !== 'SpreadElement')
            .map(prop => {
                let key = prop.key.type === 'Identifier' ? prop.key.name : String((prop.key as Literal).value);

                // EDGE: 'error' is reserved in Zig
                if (key === 'error') {
                    key = 'err';
                }

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
     *   "a" + "b" -> (a ++ b) for comptime strings
     *   a && b -> (a and b)
     *   a || b -> (a or b)
     *   s == "foo" -> std.mem.eql(u8, s, "foo")
     *   s != "foo" -> !std.mem.eql(u8, s, "foo")
     *
     * TARGET: Zig uses 'and'/'or' keywords not &&/|| operators.
     *         String concatenation requires ++ operator (comptime only).
     *         String comparison requires std.mem.eql, not ==.
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

        // Handle string comparison with == or ===
        // WHY: Zig cannot compare []const u8 with ==, must use std.mem.eql
        if ((node.operator === '==' || node.operator === '===') && (isStringType(node.left) || isStringType(node.right))) {
            return `std.mem.eql(u8, ${left}, ${right})`;
        }

        // Handle string comparison with != or !==
        if ((node.operator === '!=' || node.operator === '!==') && (isStringType(node.left) || isStringType(node.right))) {
            return `!std.mem.eql(u8, ${left}, ${right})`;
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
            case '??':
                // WHY: Zig's orelse works on optionals: a orelse b
                return 'orelse';
            case '===':
                return '==';
            case '!==':
                return '!=';
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

        // nihil: check if null (for optionals)
        if (node.operator === 'nihil') {
            return `(${arg} == null)`;
        }

        // nonnihil: check if not null (for optionals)
        if (node.operator === 'nonnihil') {
            return `(${arg} != null)`;
        }

        // negativum: check if less than zero
        if (node.operator === 'negativum') {
            return `(${arg} < 0)`;
        }

        // positivum: check if greater than zero
        if (node.operator === 'positivum') {
            return `(${arg} > 0)`;
        }

        return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
    }

    /**
     * Generate type cast expression.
     *
     * TRANSFORMS:
     *   x ut numerus -> @as(i64, x)
     *   data ut textus -> @as([]const u8, data)
     *
     * TARGET: Zig uses @as(T, x) builtin for type coercion.
     */
    function genTypeCastExpression(node: TypeCastExpression): string {
        const expr = genExpression(node.expression);
        const targetType = genType(node.targetType);

        return `@as(${targetType}, ${expr})`;
    }

    /**
     * Generate type check expression.
     *
     * TRANSFORMS:
     *   x est textus      -> @TypeOf(x) == []const u8  (compile-time)
     *   x est numerus     -> @TypeOf(x) == i64         (compile-time)
     *   x est persona     -> @TypeOf(x) == persona     (compile-time)
     *   x non est textus  -> @TypeOf(x) != []const u8
     *
     * TARGET: Zig is statically typed - all type checks are compile-time.
     *         @TypeOf gives the type at comptime.
     *
     * LIMITATION: Runtime type checks don't exist in Zig. This emits comptime
     *             comparisons that the compiler will constant-fold to true/false.
     *
     * NOTE: For null checks, use `nihil x` or `nonnihil x` unary operators.
     */
    function genTypeCheckExpression(node: TypeCheckExpression): string {
        const expr = genExpression(node.expression);
        const op = node.negated ? '!=' : '==';

        // Compile-time type comparison
        const targetType = genType(node.targetType);
        return `(@TypeOf(${expr}) ${op} ${targetType})`;
    }

    /**
     * Generate function call.
     *
     * TRANSFORMS:
     *   scribe("hello") -> std.debug.print("{s}\n", .{"hello"})
     *   scribe(42) -> std.debug.print("{d}\n", .{42})
     *   foo(x, y) -> foo(x, y)
     *   foo(sparge args) -> (spread not directly supported)
     *
     * TARGET: scribe() is Latin's print function, maps to std.debug.print().
     *         Zig print uses format strings and anonymous tuple syntax (.{...}).
     *
     * LIMITATION: Zig doesn't support spread in function calls. Would require
     *             comptime tuple unpacking or @call with .args tuple.
     */
    function genCallExpression(node: CallExpression): string {
        // Helper to generate argument, handling spread
        const genArg = (arg: Expression | SpreadElement): string => {
            if (arg.type === 'SpreadElement') {
                // WHY: Zig doesn't have spread in calls. This is a limitation.
                // Could potentially use @call(.auto, fn, args_tuple) but complex.
                return `@compileError("Call spread not supported in Zig target")`;
            }
            return genExpression(arg);
        };

        // TARGET: _scribe intrinsic maps to Zig's std.debug.print()
        if (node.callee.type === 'Identifier' && node.callee.name === '_scribe') {
            const args = node.arguments.map(genArg);
            const formatSpecs = node.arguments.map(arg => {
                if (arg.type === 'SpreadElement') {
                    return '{any}';
                }
                return getFormatSpecifier(arg);
            });
            const format = formatSpecs.join(' ') + '\\n';

            return `std.debug.print("${format}", .{${args.join(', ')}})`;
        }

        // Check for collection methods (method calls on lista/tabula/copia)
        // WHY: Latin collection methods map to Zig stdlib ArrayList/HashMap operations
        if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
            const methodName = (node.callee.property as Identifier).name;
            const obj = genExpression(node.callee.object);
            const args = node.arguments.map(genArg).join(', ');

            // Use semantic type info to dispatch to correct collection registry
            const objType = node.callee.object.resolvedType;
            const collectionName = objType?.kind === 'generic' ? objType.name : null;

            // Dispatch based on resolved type
            if (collectionName === 'tabula') {
                const method = getTabulaMethod(methodName);
                if (method) {
                    if (typeof method.zig === 'function') {
                        return method.zig(obj, args);
                    }
                    return `${obj}.${method.zig}(${args})`;
                }
            } else if (collectionName === 'copia') {
                const method = getCopiaMethod(methodName);
                if (method) {
                    if (typeof method.zig === 'function') {
                        return method.zig(obj, args);
                    }
                    return `${obj}.${method.zig}(${args})`;
                }
            } else if (collectionName === 'lista') {
                const method = getListaMethod(methodName);
                if (method) {
                    if (typeof method.zig === 'function') {
                        return method.zig(obj, args);
                    }
                    return `${obj}.${method.zig}(${args})`;
                }
            }

            // Fallback: no type info or unknown type - try lista (most common)
            const listaMethod = getListaMethod(methodName);
            if (listaMethod) {
                if (typeof listaMethod.zig === 'function') {
                    return listaMethod.zig(obj, args);
                }
                return `${obj}.${listaMethod.zig}(${args})`;
            }
        }

        const callee = genExpression(node.callee);
        const args = node.arguments.map(genArg).join(', ');

        // WHY: Optional call in Zig requires if-else pattern
        if (node.optional) {
            return `(if (${callee}) |_fn| _fn(${args}) else null)`;
        }

        // WHY: Non-null assertion unwraps optional function before calling
        if (node.nonNull) {
            return `${callee}.?(${args})`;
        }

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
                case 'textus':
                    return '{s}';
                case 'numerus':
                    return '{d}';
                case 'bivalens':
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
     *   obj.prop  -> obj.prop
     *   obj[key]  -> obj[key]
     *   obj?.prop -> if (obj) |o| o.prop else null (simplified)
     *   obj!.prop -> obj.?.prop (unwrap optional)
     */
    function genMemberExpression(node: MemberExpression): string {
        const obj = genExpression(node.object);
        const prop = node.computed ? `[${genExpression(node.property)}]` : `.${(node.property as Identifier).name}`;

        // WHY: Zig's optional unwrap uses .? syntax
        if (node.nonNull) {
            return `${obj}.?${prop}`;
        }

        // WHY: Zig's optional chaining requires if-else pattern
        //      This is a simplified version; full impl would need temp vars
        if (node.optional) {
            const propName = node.computed ? `[${genExpression(node.property)}]` : (node.property as Identifier).name;
            return `(if (${obj}) |_o| _o.${propName} else null)`;
        }

        return `${obj}${prop}`;
    }

    /**
     * Generate arrow function (emulated).
     *
     * TRANSFORMS:
     *   (x) => x + 1 -> @compileError("Arrow functions not supported in Zig")
     *
     * TARGET: Zig doesn't have arrow functions or lambdas as first-class values.
     *         Arrow functions don't have return type annotations in Faber, so
     *         they cannot be compiled to Zig. Use pro syntax with return type instead.
     *
     * LIMITATION: Arrow functions should be converted to lambdas with return types
     *             or named functions for Zig target.
     */
    function genArrowFunction(_node: ArrowFunctionExpression): string {
        return `@compileError("Arrow functions not supported in Zig - use 'pro x -> Type: expr' syntax")`;
    }

    /**
     * Generate lambda expression (pro syntax).
     *
     * TRANSFORMS:
     *   pro x -> numerus: x * 2 -> struct { fn call(x: anytype) i64 { return x * 2; } }.call
     *   pro x: x * 2 -> @compileError("Lambda requires return type for Zig")
     *
     * TARGET: Zig doesn't have lambdas/closures as first-class values.
     *         We emulate with anonymous struct containing a function.
     *         This ONLY works when a return type annotation is provided,
     *         because anytype can't be used as a return type in Zig.
     *
     * LIMITATION: Closures are not properly supported - captured variables
     *             would need to be passed explicitly via context struct.
     */
    function genLambdaExpression(node: LambdaExpression): string {
        // WHY: Zig doesn't support first-class functions with inferred return types.
        // With an explicit return type, we can generate valid Zig code.
        if (!node.returnType) {
            return `@compileError("Lambda requires return type annotation for Zig target: pro x -> Type: expr")`;
        }

        // Build parameter list - lambda params are just Identifiers
        const params = node.params.map(p => `${p.name}: anytype`).join(', ');
        const returnType = genType(node.returnType);

        if (node.body.type === 'BlockStatement') {
            const body = genBlockStatement(node.body);

            return `struct { fn call(${params}) ${returnType} ${body} }.call`;
        }

        // Expression body
        const body = genExpression(node.body as Expression);

        return `struct { fn call(${params}) ${returnType} { return ${body}; } }.call`;
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
     * Generate compile-time evaluation expression.
     *
     * TRANSFORMS:
     *   praefixum(256 * 4) -> comptime (256 * 4)
     *   praefixum { ... } -> comptime blk: { ... break :blk result; }
     *
     * TARGET: Zig's comptime keyword handles both forms:
     *         - Expression: comptime (expr)
     *         - Block: comptime blk: { ... break :blk result; }
     *
     * WHY: praefixum delegates compile-time evaluation to Zig's comptime.
     *      The Faber compiler doesn't interpret the code; Zig's compiler does.
     */
    function genPraefixumExpression(node: PraefixumExpression): string {
        if (node.body.type === 'BlockStatement') {
            // Block form: comptime blk: { ... }
            // Note: The block should end with a redde (return) which becomes break :blk value
            const body = genComptimeBlockStatement(node.body);
            return `comptime ${body}`;
        }

        // Expression form: comptime (expr)
        const expr = genExpression(node.body);
        return `comptime (${expr})`;
    }

    /**
     * Generate a comptime block statement with labeled break.
     *
     * WHY: Zig's comptime blocks that return values need labeled breaks.
     *      We transform redde statements into break :blk value statements.
     */
    function genComptimeBlockStatement(node: BlockStatement): string {
        if (node.body.length === 0) {
            return 'blk: {}';
        }

        depth++;
        const statements: string[] = [];

        for (const stmt of node.body) {
            if (stmt.type === 'ReturnStatement') {
                // Transform redde into break :blk
                const value = stmt.argument ? genExpression(stmt.argument) : 'void{}';
                statements.push(`${ind()}break :blk ${value};`);
            } else {
                statements.push(genStatement(stmt));
            }
        }

        depth--;

        return `blk: {\n${statements.join('\n')}\n${ind()}}`;
    }

    /**
     * Generate new expression.
     *
     * TRANSFORMS:
     *   novum Foo -> Foo.init(.{})
     *   novum Foo(x, y) -> Foo.init(x, y)
     *   novum Foo { a: 1 } -> Foo.init(.{ .a = 1 })
     *   novum Foo de props -> Foo.init(props)
     *
     * TARGET: Zig doesn't have 'new' keyword. Idiomatic pattern is Type.init().
     *         Property overrides are passed as an anonymous struct.
     *
     * WHY: When no overrides are provided, we pass .{} (empty struct) so the
     *      init() function's @hasField checks all return false, using defaults.
     */
    function genNewExpression(node: NewExpression): string {
        const callee = node.callee.name;

        // Handle { ... } or de expr overrides
        if (node.withExpression) {
            const overrides = genExpression(node.withExpression);
            return `${callee}.init(${overrides})`;
        }

        // Regular constructor call with arguments
        if (node.arguments.length > 0) {
            const args = node.arguments
                .filter((arg): arg is Expression => arg.type !== 'SpreadElement')
                .map(genExpression)
                .join(', ');
            return `${callee}.init(${args})`;
        }

        // No arguments and no overrides: pass empty struct for @hasField pattern
        return `${callee}.init(.{})`;
    }

    return genProgram(program);
}
