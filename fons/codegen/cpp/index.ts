/**
 * C++23 Code Generator - Emit modern C++ source code
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into C++23 source code.
 * C++23 provides modern features that map well to Latin constructs:
 * - std::print for output (replaces iostream verbosity)
 * - std::expected<T,E> for error handling (aligns with Rust's Result)
 * - Concepts for interfaces (cleaner than abstract classes)
 * - Range-based for with views
 *
 * Key transformations:
 * - varia -> auto (mutable by default in C++)
 * - fixum -> const auto
 * - functio -> function definition
 * - genus -> struct (public by default, simpler than class)
 * - pactum -> concept (C++20)
 * - si/aliter -> if/else
 * - elige -> switch
 * - ex...pro -> range-based for
 * - scribe -> std::print
 * - ego -> this (pointer, use this-> for access)
 * - novum -> std::make_unique<T>(...) for heap allocation
 * - iace -> throw (using exceptions, not std::expected for simplicity)
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid C++23 source code string
 * ERRORS: Throws on unsupported AST node types
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid C++23
 * INV-2: All Latin type names are mapped to STL equivalents
 * INV-3: 4-space indentation (common C++ convention)
 */

import type {
    Program,
    Statement,
    Expression,
    ImportDeclaration,
    VariableDeclaration,
    FunctionDeclaration,
    GenusDeclaration,
    FieldDeclaration,
    PactumDeclaration,
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
} from '../../parser/ast';
import type { CodegenOptions } from '../types';

// =============================================================================
// TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to C++23 types.
 *
 * TARGET MAPPING:
 * | Latin      | C++23                  |
 * |------------|------------------------|
 * | textus     | std::string            |
 * | numerus    | int64_t                |
 * | fractus    | double                 |
 * | bivalens   | bool                   |
 * | nihil      | std::nullptr_t         |
 * | vacuum     | void                   |
 * | lista      | std::vector            |
 * | tabula     | std::unordered_map     |
 * | copia      | std::unordered_set     |
 * | promissum  | std::future            |
 * | erratum    | std::runtime_error     |
 */
const typeMap: Record<string, string> = {
    textus: 'std::string',
    numerus: 'int64_t',
    fractus: 'double',
    decimus: 'double',
    bivalens: 'bool',
    nihil: 'void', // WHY: nihil as return type means "returns nothing"
    vacuum: 'void',
    octeti: 'std::vector<uint8_t>',
    lista: 'std::vector',
    tabula: 'std::unordered_map',
    copia: 'std::unordered_set',
    promissum: 'std::future',
    erratum: 'std::runtime_error',
    objectum: 'std::any',
};

// =============================================================================
// MAIN GENERATOR
// =============================================================================

/**
 * Generate C++23 source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> C++23 source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration
 * @returns C++23 source code
 */
export function generateCpp(program: Program, options: CodegenOptions = {}): string {
    // WHY: 4 spaces is common C++ convention
    const indent = options.indent ?? '    ';

    // Track indentation depth
    let depth = 0;

    // Track which headers are needed
    const includes = new Set<string>();

    /**
     * Generate indentation for current depth.
     */
    function ind(): string {
        return indent.repeat(depth);
    }

    // -------------------------------------------------------------------------
    // Top-level
    // -------------------------------------------------------------------------

    /**
     * Generate top-level program structure.
     *
     * C++ requires:
     * - #include directives at top
     * - Function declarations before use (or prototypes)
     * - int main() entry point for runtime code
     */
    function genProgram(node: Program): string {
        const lines: string[] = [];

        // Separate declarations from runtime statements
        const declarations: Statement[] = [];
        const runtime: Statement[] = [];

        for (const stmt of node.body) {
            if (isDeclaration(stmt)) {
                declarations.push(stmt);
            } else {
                runtime.push(stmt);
            }
        }

        // Generate declarations first (functions, classes, etc.)
        for (const stmt of declarations) {
            lines.push(genStatement(stmt));
        }

        // Wrap runtime code in main()
        if (runtime.length > 0) {
            if (declarations.length > 0) {
                lines.push('');
            }

            lines.push('int main() {');
            depth++;

            for (const stmt of runtime) {
                lines.push(genStatement(stmt));
            }

            lines.push(`${ind()}return 0;`);
            depth--;
            lines.push('}');
        }

        // Build includes based on what was used
        const includeLines = genIncludes();

        if (includeLines) {
            return includeLines + '\n' + lines.join('\n');
        }

        return lines.join('\n');
    }

    /**
     * Check if a statement is a declaration (vs runtime code).
     */
    function isDeclaration(node: Statement): boolean {
        switch (node.type) {
            case 'FunctionDeclaration':
            case 'GenusDeclaration':
            case 'PactumDeclaration':
            case 'TypeAliasDeclaration':
            case 'EnumDeclaration':
                return true;
            // Top-level const could be declaration, but we'll put in main for simplicity
            default:
                return false;
        }
    }

    /**
     * Generate #include directives based on features used.
     */
    function genIncludes(): string {
        // Always include these for basic functionality
        includes.add('<print>');
        includes.add('<string>');
        includes.add('<cstdint>');

        const sorted = Array.from(includes).sort();

        return sorted.map(h => `#include ${h}`).join('\n');
    }

    // -------------------------------------------------------------------------
    // Statements
    // -------------------------------------------------------------------------

    /**
     * Generate code for any statement type.
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
     *   ex iostream importa * -> #include <iostream>
     *   ex "mylib" importa foo -> #include "mylib" (handled in includes)
     *
     * WHY: C++ uses #include, not import (until C++20 modules are widespread).
     *      For now, we just track that an import was requested.
     */
    function genImportDeclaration(node: ImportDeclaration): string {
        // Add to includes set - will be rendered at top
        const source = node.source;

        // Check if it's a standard library or local
        if (source.startsWith('<') || !source.includes('/')) {
            includes.add(`<${source}>`);
        } else {
            includes.add(`"${source}"`);
        }

        return ''; // Actual include rendered at top
    }

    /**
     * Generate variable declaration.
     *
     * TRANSFORMS:
     *   varia x = 5 -> auto x = 5;
     *   fixum y = "hi" -> const auto y = "hi";
     *   varia numerus x = 5 -> int64_t x = 5;
     *   fixum textus y = "hi" -> const std::string y = "hi";
     */
    function genVariableDeclaration(node: VariableDeclaration): string {
        const isConst = node.kind === 'fixum';

        // Handle destructuring (not directly supported in C++, expand it)
        if (node.name.type === 'ObjectPattern') {
            return genDestructuringDeclaration(node);
        }

        const name = node.name.name;
        const constPrefix = isConst ? 'const ' : '';

        let typeSpec: string;

        if (node.typeAnnotation) {
            typeSpec = genType(node.typeAnnotation);
        } else {
            typeSpec = 'auto';
        }

        const init = node.init ? ` = ${genExpression(node.init)}` : '';

        return `${ind()}${constPrefix}${typeSpec} ${name}${init};`;
    }

    /**
     * Generate destructuring as multiple declarations.
     *
     * TRANSFORMS:
     *   fixum { a, b } = obj -> const auto& a = obj.a; const auto& b = obj.b;
     */
    function genDestructuringDeclaration(node: VariableDeclaration): string {
        if (node.name.type !== 'ObjectPattern') {
            throw new Error('Expected ObjectPattern');
        }

        const lines: string[] = [];
        const isConst = node.kind === 'fixum';
        const constPrefix = isConst ? 'const ' : '';
        const initExpr = node.init ? genExpression(node.init) : 'undefined';

        // Create temp variable
        lines.push(`${ind()}${constPrefix}auto& _tmp = ${initExpr};`);

        // Extract each property
        for (const prop of node.name.properties) {
            const key = prop.key.name;
            const localName = prop.value.name;

            lines.push(`${ind()}${constPrefix}auto& ${localName} = _tmp.${key};`);
        }

        return lines.join('\n');
    }

    /**
     * Generate function declaration.
     *
     * TRANSFORMS:
     *   functio salve(textus nomen) -> nihil { }
     *   -> void salve(std::string nomen) { }
     */
    function genFunctionDeclaration(node: FunctionDeclaration): string {
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');
        const returnType = node.returnType ? genType(node.returnType) : 'void';

        const body = genBlockStatement(node.body);

        return `${ind()}${returnType} ${name}(${params}) ${body}`;
    }

    /**
     * Generate function parameter.
     */
    function genParameter(node: Parameter): string {
        const name = node.name.name;

        if (node.typeAnnotation) {
            const type = genType(node.typeAnnotation);

            // Pass strings and vectors by const reference
            if (type === 'std::string' || type.startsWith('std::vector')) {
                return `const ${type}& ${name}`;
            }

            return `${type} ${name}`;
        }

        // No type annotation - use auto (requires C++20 abbreviated function template)
        return `auto ${name}`;
    }

    /**
     * Generate type annotation.
     *
     * TRANSFORMS:
     *   textus -> std::string
     *   numerus -> int64_t
     *   lista<textus> -> std::vector<std::string>
     *   textus? -> std::optional<std::string>
     */
    function genType(node: TypeAnnotation): string {
        const base = typeMap[node.name.toLowerCase()] ?? node.name;

        // Handle generic type parameters
        let result = base;

        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters
                .map(p => {
                    if (p.type === 'TypeAnnotation') {
                        return genType(p);
                    }

                    // Literal type parameter (e.g., numeric size)
                    return String(p.value);
                })
                .join(', ');

            result = `${base}<${params}>`;
        }

        // Handle nullable -> std::optional
        if (node.nullable) {
            includes.add('<optional>');

            return `std::optional<${result}>`;
        }

        // Add includes based on type
        if (base === 'std::string') {
            includes.add('<string>');
        }

        if (base === 'std::vector') {
            includes.add('<vector>');
        }

        if (base === 'std::unordered_map') {
            includes.add('<unordered_map>');
        }

        if (base === 'std::unordered_set') {
            includes.add('<unordered_set>');
        }

        return result;
    }

    // -------------------------------------------------------------------------
    // OOP: Genus (Struct) and Pactum (Concept)
    // -------------------------------------------------------------------------

    /**
     * Generate genus declaration as C++ struct.
     *
     * TRANSFORMS:
     *   genus Persona { textus nomen: "anon" }
     *   -> struct Persona {
     *          std::string nomen = "anon";
     *      };
     */
    function genGenusDeclaration(node: GenusDeclaration): string {
        const name = node.name.name;
        const lines: string[] = [];

        lines.push(`${ind()}struct ${name} {`);
        depth++;

        // Fields
        for (const field of node.fields) {
            lines.push(genFieldDeclaration(field));
        }

        // Constructor if there's a creo
        if (node.constructor) {
            lines.push('');
            lines.push(genCreoConstructor(node));
        }

        // Methods
        for (const method of node.methods) {
            lines.push('');
            lines.push(genMethodDeclaration(method));
        }

        depth--;
        lines.push(`${ind()}};`);

        return lines.join('\n');
    }

    /**
     * Generate field declaration.
     */
    function genFieldDeclaration(node: FieldDeclaration): string {
        const name = node.name.name;
        const type = genType(node.fieldType);
        const init = node.init ? ` = ${genExpression(node.init)}` : '';

        return `${ind()}${type} ${name}${init};`;
    }

    /**
     * Generate creo as constructor.
     */
    function genCreoConstructor(node: GenusDeclaration): string {
        const name = node.name.name;
        const lines: string[] = [];

        lines.push(`${ind()}${name}() {`);
        depth++;

        if (node.constructor) {
            for (const stmt of node.constructor.body.body) {
                const code = genStatement(stmt);
                // Replace ego. with this->
                lines.push(code.replace(/\bego\./g, 'this->'));
            }
        }

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate method declaration.
     */
    function genMethodDeclaration(node: FunctionDeclaration): string {
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');
        const returnType = node.returnType ? genType(node.returnType) : 'void';

        const lines: string[] = [];

        lines.push(`${ind()}${returnType} ${name}(${params}) {`);
        depth++;

        for (const stmt of node.body.body) {
            const code = genStatement(stmt);
            // Replace ego. with this->
            lines.push(code.replace(/\bego\./g, 'this->'));
        }

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate pactum as C++20 concept.
     *
     * TRANSFORMS:
     *   pactum Iterabilis { functio sequens() -> textus? }
     *   -> template<typename T>
     *      concept Iterabilis = requires(T t) {
     *          { t.sequens() } -> std::same_as<std::optional<std::string>>;
     *      };
     */
    function genPactumDeclaration(node: PactumDeclaration): string {
        const name = node.name.name;
        const lines: string[] = [];

        includes.add('<concepts>');

        lines.push(`${ind()}template<typename T>`);
        lines.push(`${ind()}concept ${name} = requires(T t) {`);
        depth++;

        for (const method of node.methods) {
            const methodName = method.name.name;
            const returnType = method.returnType ? genType(method.returnType) : 'void';

            // Generate parameter types for the requires expression
            const paramTypes = method.params.map(p => {
                if (p.typeAnnotation) {
                    return genType(p.typeAnnotation);
                }

                return 'auto';
            });

            if (paramTypes.length === 0) {
                lines.push(`${ind()}{ t.${methodName}() } -> std::same_as<${returnType}>;`);
            } else {
                lines.push(`${ind()}{ t.${methodName}(std::declval<${paramTypes.join('>(), std::declval<')}>()) } -> std::same_as<${returnType}>;`);
            }
        }

        depth--;
        lines.push(`${ind()}};`);

        return lines.join('\n');
    }

    /**
     * Generate type alias.
     *
     * TRANSFORMS:
     *   typus ID = textus -> using ID = std::string;
     */
    function genTypeAliasDeclaration(node: TypeAliasDeclaration): string {
        const name = node.name.name;
        const type = genType(node.typeAnnotation);

        return `${ind()}using ${name} = ${type};`;
    }

    // -------------------------------------------------------------------------
    // Control Flow
    // -------------------------------------------------------------------------

    function genIfStatement(node: IfStatement): string {
        let result = `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`;

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
     *   ex items pro item { } -> for (auto& item : items) { }
     *   ex 0..10 pro i { } -> for (int64_t i = 0; i < 10; ++i) { }
     */
    function genForStatement(node: ForStatement): string {
        const varName = node.variable.name;
        const body = genBlockStatement(node.body);

        // Handle range expressions
        if (node.iterable.type === 'RangeExpression') {
            const range = node.iterable;
            const start = genExpression(range.start);
            const end = genExpression(range.end);
            const step = range.step ? genExpression(range.step) : '1';

            return `${ind()}for (int64_t ${varName} = ${start}; ${varName} < ${end}; ${varName} += ${step}) ${body}`;
        }

        const iterable = genExpression(node.iterable);

        // Range-based for
        return `${ind()}for (auto& ${varName} : ${iterable}) ${body}`;
    }

    /**
     * Generate with statement (mutation block).
     *
     * TRANSFORMS:
     *   in user { nomen = "Marcus" } -> { user.nomen = "Marcus"; }
     */
    function genWithStatement(node: WithStatement): string {
        const context = genExpression(node.object);
        const lines: string[] = [];

        lines.push(`${ind()}{`);
        depth++;

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

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate switch statement.
     */
    function genSwitchStatement(node: SwitchStatement): string {
        const discriminant = genExpression(node.discriminant);
        const lines: string[] = [];

        lines.push(`${ind()}switch (${discriminant}) {`);

        for (const caseNode of node.cases) {
            const test = genExpression(caseNode.test);

            lines.push(`${ind()}case ${test}: {`);
            depth++;

            for (const stmt of caseNode.consequent.body) {
                lines.push(genStatement(stmt));
            }

            lines.push(`${ind()}break;`);
            depth--;
            lines.push(`${ind()}}`);
        }

        if (node.defaultCase) {
            lines.push(`${ind()}default: {`);
            depth++;

            for (const stmt of node.defaultCase.body) {
                lines.push(genStatement(stmt));
            }

            lines.push(`${ind()}break;`);
            depth--;
            lines.push(`${ind()}}`);
        }

        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    function genGuardStatement(node: GuardStatement): string {
        const lines: string[] = [];

        for (const clause of node.clauses) {
            const test = genExpression(clause.test);
            const body = genBlockStatement(clause.consequent);

            lines.push(`${ind()}if (${test}) ${body}`);
        }

        return lines.join('\n');
    }

    function genAssertStatement(node: AssertStatement): string {
        includes.add('<cassert>');

        const test = genExpression(node.test);

        if (node.message) {
            // C++ assert doesn't support messages easily, use comma operator trick
            const msg = genExpression(node.message);

            return `${ind()}assert((${test}) && ${msg});`;
        }

        return `${ind()}assert(${test});`;
    }

    function genReturnStatement(node: ReturnStatement): string {
        if (node.argument) {
            return `${ind()}return ${genExpression(node.argument)};`;
        }

        return `${ind()}return;`;
    }

    function genThrowStatement(node: ThrowStatement): string {
        includes.add('<stdexcept>');

        // Handle string literals -> runtime_error
        if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
            return `${ind()}throw std::runtime_error("${node.argument.value}");`;
        }

        // Handle new Error("msg")
        if (node.argument.type === 'NewExpression' && node.argument.callee.name === 'Error') {
            const firstArg = node.argument.arguments[0];
            const msg = firstArg ? genExpression(firstArg) : '""';

            return `${ind()}throw std::runtime_error(${msg});`;
        }

        return `${ind()}throw ${genExpression(node.argument)};`;
    }

    /**
     * Generate scribe statement.
     *
     * TRANSFORMS:
     *   scribe "hello" -> std::print("hello\n");
     *   scribe x, y -> std::print("{} {}\n", x, y);
     */
    function genScribeStatement(node: ScribeStatement): string {
        includes.add('<print>');

        const prefix = node.level === 'debug' ? '[DEBUG] ' : node.level === 'warn' ? '[WARN] ' : '';

        if (node.arguments.length === 0) {
            return `${ind()}std::print("${prefix}\\n");`;
        }

        // Build format string with {} placeholders
        const formatParts = node.arguments.map(() => '{}');
        const format = prefix + formatParts.join(' ') + '\\n';
        const args = node.arguments.map(genExpression).join(', ');

        return `${ind()}std::print("${format}", ${args});`;
    }

    function genTryStatement(node: TryStatement): string {
        const lines: string[] = [];

        lines.push(`${ind()}try ${genBlockStatement(node.block)}`);

        if (node.handler) {
            const param = node.handler.param.name;

            lines.push(`${ind()}catch (const std::exception& ${param}) ${genBlockStatement(node.handler.body)}`);
        }

        if (node.finalizer) {
            // C++ doesn't have finally, but we can use RAII or just add a comment
            lines.push(`${ind()}// finally block (use RAII for cleanup)`);
            lines.push(genBlockStatementContent(node.finalizer));
        }

        return lines.join('\n');
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

    function genExpressionStatement(node: ExpressionStatement): string {
        return `${ind()}${genExpression(node.expression)};`;
    }

    // -------------------------------------------------------------------------
    // Expressions
    // -------------------------------------------------------------------------

    function genExpression(node: Expression): string {
        switch (node.type) {
            case 'Identifier':
                return genIdentifier(node);
            case 'Literal':
                return genLiteral(node);
            case 'TemplateLiteral':
                return genTemplateLiteral(node);
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
                // C++ doesn't have await, would need coroutines
                return genExpression(node.argument);
            case 'ThisExpression':
                return 'this';
            case 'NewExpression':
                return genNewExpression(node);
            case 'ConditionalExpression':
                return `(${genExpression(node.test)} ? ${genExpression(node.consequent)} : ${genExpression(node.alternate)})`;
            default:
                throw new Error(`Unknown expression type: ${(node as any).type}`);
        }
    }

    function genIdentifier(node: Identifier): string {
        switch (node.name) {
            case 'verum':
                return 'true';
            case 'falsum':
                return 'false';
            case 'nihil':
                return 'nullptr';
            case 'ego':
                return 'this';
            default:
                return node.name;
        }
    }

    function genLiteral(node: Literal): string {
        if (node.value === null) {
            return 'nullptr';
        }

        if (typeof node.value === 'string') {
            // Use std::string literal
            return `std::string("${node.value}")`;
        }

        if (typeof node.value === 'boolean') {
            return node.value ? 'true' : 'false';
        }

        return String(node.value);
    }

    function genTemplateLiteral(node: any): string {
        includes.add('<format>');

        // Convert template literal to std::format
        // This is a simplified version - full implementation would parse ${} expressions
        const raw = node.raw as string;

        // Replace ${...} with {}
        const format = raw.replace(/\$\{([^}]+)\}/g, '{}').replace(/`/g, '"');

        // Extract expressions
        const exprs: string[] = [];
        const regex = /\$\{([^}]+)\}/g;
        let match;

        while ((match = regex.exec(raw)) !== null) {
            if (match[1]) {
                exprs.push(match[1]);
            }
        }

        if (exprs.length === 0) {
            return `std::string(${format})`;
        }

        return `std::format(${format}, ${exprs.join(', ')})`;
    }

    function genArrayExpression(node: ArrayExpression): string {
        includes.add('<vector>');

        if (node.elements.length === 0) {
            return 'std::vector<int>{}';
        }

        const elements = node.elements.map(genExpression).join(', ');

        return `std::vector{${elements}}`;
    }

    function genObjectExpression(node: ObjectExpression): string {
        if (node.properties.length === 0) {
            return '{}';
        }

        // For simple structs, use designated initializers (C++20)
        const props = node.properties.map(prop => {
            const key = prop.key.type === 'Identifier' ? prop.key.name : String((prop.key as Literal).value);
            const value = genExpression(prop.value);

            return `.${key} = ${value}`;
        });

        return `{${props.join(', ')}}`;
    }

    function genRangeExpression(node: RangeExpression): string {
        // Range expressions should be handled in for loops
        // If used standalone, we'd need std::ranges::iota_view
        includes.add('<ranges>');

        const start = genExpression(node.start);
        const end = genExpression(node.end);

        return `std::views::iota(${start}, ${end} + 1)`;
    }

    function genBinaryExpression(node: BinaryExpression): string {
        const left = genExpression(node.left);
        const right = genExpression(node.right);

        // WHY: C++ has no ?? operator; use ternary with nullptr check
        //      For std::optional, would use .value_or() instead
        if (node.operator === '??') {
            return `(${left} != nullptr ? ${left} : ${right})`;
        }

        const op = mapOperator(node.operator);

        return `(${left} ${op} ${right})`;
    }

    function mapOperator(op: string): string {
        switch (op) {
            case '===':
                return '==';
            case '!==':
                return '!=';
            case '&&':
                return '&&';
            case '||':
                return '||';
            default:
                return op;
        }
    }

    function genUnaryExpression(node: UnaryExpression): string {
        const arg = genExpression(node.argument);

        // Handle Latin-specific operators
        switch (node.operator) {
            case 'nulla':
                return `(${arg}.empty())`;
            case 'nonnulla':
                return `(!${arg}.empty())`;
            case 'negativum':
                return `(${arg} < 0)`;
            case 'positivum':
                return `(${arg} > 0)`;
        }

        return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
    }

    /**
     * Generate function call.
     *
     * TRANSFORMS:
     *   fn()    -> fn()
     *   fn?()   -> (fn ? (*fn)() : std::nullopt)  (for function pointers)
     *   fn!()   -> (*fn)()  (assert not null)
     */
    function genCallExpression(node: CallExpression): string {
        const callee = genExpression(node.callee);
        const args = node.arguments.map(genExpression).join(', ');

        // WHY: For optional call, check if function pointer is valid
        if (node.optional) {
            includes.add('<optional>');
            return `(${callee} ? (*${callee})(${args}) : std::nullopt)`;
        }
        // WHY: For non-null assertion, dereference and call
        if (node.nonNull) {
            return `(*${callee})(${args})`;
        }
        return `${callee}(${args})`;
    }

    /**
     * Generate member access.
     *
     * TRANSFORMS:
     *   obj.prop      -> obj.prop
     *   obj?.prop     -> (obj ? obj->prop : std::nullopt)  (for pointers)
     *   obj!.prop     -> obj->prop  (assert not null, just dereference)
     *   obj[idx]      -> obj[idx]
     *   obj?[idx]     -> (obj ? (*obj)[idx] : std::nullopt)
     *   obj![idx]     -> (*obj)[idx]
     */
    function genMemberExpression(node: MemberExpression): string {
        const obj = genExpression(node.object);

        if (node.computed) {
            const prop = genExpression(node.property);
            // WHY: For optional, use ternary with nullptr check
            if (node.optional) {
                includes.add('<optional>');
                return `(${obj} ? (*${obj})[${prop}] : std::nullopt)`;
            }
            // WHY: For non-null assertion, dereference and access
            if (node.nonNull) {
                return `(*${obj})[${prop}]`;
            }
            return `${obj}[${prop}]`;
        }

        const prop = (node.property as Identifier).name;

        // Handle ego -> this
        if (obj === 'this') {
            return `this->${prop}`;
        }

        if (node.optional) {
            includes.add('<optional>');
            return `(${obj} ? ${obj}->${prop} : std::nullopt)`;
        }
        if (node.nonNull) {
            return `${obj}->${prop}`;
        }
        return `${obj}.${prop}`;
    }

    /**
     * Generate arrow function as C++ lambda.
     *
     * TRANSFORMS:
     *   (x) => x + 1 -> [](auto x) { return x + 1; }
     *   (x, y) => x + y -> [](auto x, auto y) { return x + y; }
     */
    function genArrowFunction(node: ArrowFunctionExpression): string {
        const params = node.params
            .map(p => {
                const name = p.name.name;

                if (p.typeAnnotation) {
                    return `${genType(p.typeAnnotation)} ${name}`;
                }

                return `auto ${name}`;
            })
            .join(', ');

        if (node.body.type === 'BlockStatement') {
            const body = genBlockStatement(node.body);

            return `[&](${params}) ${body}`;
        }

        const body = genExpression(node.body as Expression);

        return `[&](${params}) { return ${body}; }`;
    }

    function genAssignmentExpression(node: AssignmentExpression): string {
        const left = node.left.type === 'Identifier' ? node.left.name : genExpression(node.left);
        const right = genExpression(node.right);

        return `${left} ${node.operator} ${right}`;
    }

    /**
     * Generate new expression.
     *
     * TRANSFORMS:
     *   novum Foo() -> std::make_unique<Foo>()
     *   novum Foo { x: 1 } -> Foo{.x = 1}
     */
    function genNewExpression(node: NewExpression): string {
        const callee = node.callee.name;

        // Handle { ... } or de expr overrides - use aggregate initialization
        if (node.withExpression) {
            const overrides = genExpression(node.withExpression);

            return `${callee}${overrides}`;
        }

        // No arguments - stack allocation with default init
        if (node.arguments.length === 0) {
            return `${callee}{}`;
        }

        // With arguments - use parentheses
        const args = node.arguments.map(genExpression).join(', ');

        return `${callee}(${args})`;
    }

    return genProgram(program);
}
