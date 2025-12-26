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
    ImportaDeclaration,
    VariaDeclaration,
    FunctioDeclaration,
    GenusDeclaration,
    FieldDeclaration,
    PactumDeclaration,
    TypeAliasDeclaration,
    DiscretioDeclaration,
    SiStatement,
    DumStatement,
    IteratioStatement,
    InStatement,
    EligeStatement,
    CustodiStatement,
    AdfirmaStatement,
    ReddeStatement,
    BlockStatement,
    IaceStatement,
    ScribeStatement,
    TemptaStatement,
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
    NovumExpression,
    Identifier,
    Literal,
    Parameter,
    TypeAnnotation,
    LambdaExpression,
    PraefixumExpression,
} from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { getListaMethod, getListaHeaders } from './norma/lista';
import { getTabulaMethod, getTabulaHeaders } from './norma/tabula';
import { getCopiaMethod, getCopiaHeaders } from './norma/copia';

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

    // Track whether we need the scope guard helper for demum (finally)
    let needsScopeGuard = false;
    let scopeGuardCounter = 0;

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

        // Build preamble (includes + helpers)
        const preambleParts: string[] = [];

        if (includeLines) {
            preambleParts.push(includeLines);
        }

        // Add scope guard helper if demum was used
        if (needsScopeGuard) {
            preambleParts.push('');
            preambleParts.push(genScopeGuardHelper());
        }

        if (preambleParts.length > 0) {
            return preambleParts.join('\n') + '\n' + lines.join('\n');
        }

        return lines.join('\n');
    }

    /**
     * Check if a statement is a declaration (vs runtime code).
     */
    function isDeclaration(node: Statement): boolean {
        switch (node.type) {
            case 'FunctioDeclaration':
            case 'GenusDeclaration':
            case 'PactumDeclaration':
            case 'TypeAliasDeclaration':
            case 'OrdoDeclaration':
            case 'DiscretioDeclaration':
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

    /**
     * Generate scope guard helper for demum (finally) blocks.
     *
     * WHY: C++ lacks finally. This RAII helper runs cleanup on scope exit.
     * The lambda captures by reference, so it can access local variables.
     */
    function genScopeGuardHelper(): string {
        return `
template<typename F>
struct _ScopeGuard {
    F fn;
    ~_ScopeGuard() { fn(); }
};
`.trim();
    }

    // -------------------------------------------------------------------------
    // Statements
    // -------------------------------------------------------------------------

    /**
     * Generate code for any statement type.
     */
    function genStatement(node: Statement): string {
        switch (node.type) {
            case 'ImportaDeclaration':
                return genImportaDeclaration(node);
            case 'VariaDeclaration':
                return genVariaDeclaration(node);
            case 'FunctioDeclaration':
                return genFunctioDeclaration(node);
            case 'GenusDeclaration':
                return genGenusDeclaration(node);
            case 'PactumDeclaration':
                return genPactumDeclaration(node);
            case 'TypeAliasDeclaration':
                return genTypeAliasDeclaration(node);
            case 'DiscretioDeclaration':
                return genDiscretioDeclaration(node);
            case 'SiStatement':
                return genSiStatement(node);
            case 'DumStatement':
                return genDumStatement(node);
            case 'IteratioStatement':
                return genIteratioStatement(node);
            case 'InStatement':
                return genInStatement(node);
            case 'EligeStatement':
                return genEligeStatement(node);
            case 'CustodiStatement':
                return genCustodiStatement(node);
            case 'AdfirmaStatement':
                return genAdfirmaStatement(node);
            case 'ReddeStatement':
                return genReddeStatement(node);
            case 'IaceStatement':
                return genIaceStatement(node);
            case 'ScribeStatement':
                return genScribeStatement(node);
            case 'TemptaStatement':
                return genTemptaStatement(node);
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
    function genImportaDeclaration(node: ImportaDeclaration): string {
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
    function genVariaDeclaration(node: VariaDeclaration): string {
        const isConst = node.kind === 'fixum';

        // Handle destructuring (not directly supported in C++, expand it)
        if (node.name.type === 'ObjectPattern') {
            return genObjectDestructuringDeclaration(node);
        }

        // Handle array destructuring (expand to indexed access)
        if (node.name.type === 'ArrayPattern') {
            return genArrayDestructuringDeclaration(node);
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
     * Generate object destructuring as multiple declarations.
     *
     * TRANSFORMS:
     *   fixum { a, b } = obj -> const auto& a = obj.a; const auto& b = obj.b;
     */
    function genObjectDestructuringDeclaration(node: VariaDeclaration): string {
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
     * Generate array destructuring as multiple declarations.
     *
     * TRANSFORMS:
     *   fixum [a, b, c] = arr -> const auto& _tmp = arr; const auto& a = _tmp[0]; ...
     *   [a, ceteri rest] = arr -> const auto& a = _tmp[0]; const auto rest = std::vector(_tmp.begin() + 1, _tmp.end());
     */
    function genArrayDestructuringDeclaration(node: VariaDeclaration): string {
        if (node.name.type !== 'ArrayPattern') {
            throw new Error('Expected ArrayPattern');
        }

        const lines: string[] = [];
        const isConst = node.kind === 'fixum';
        const constPrefix = isConst ? 'const ' : '';
        const initExpr = node.init ? genExpression(node.init) : 'undefined';

        // Create temp variable
        lines.push(`${ind()}${constPrefix}auto& _tmp = ${initExpr};`);

        let idx = 0;
        for (const elem of node.name.elements) {
            if (elem.skip) {
                // Skip this position
                idx++;
                continue;
            }

            const localName = elem.name.name;

            if (elem.rest) {
                // Rest pattern: collect remaining elements
                // Use vector constructor with iterators
                lines.push(`${ind()}${constPrefix}auto ${localName} = std::vector<decltype(_tmp)::value_type>(_tmp.begin() + ${idx}, _tmp.end());`);
            } else {
                // Regular element: indexed access
                lines.push(`${ind()}${constPrefix}auto& ${localName} = _tmp[${idx}];`);
                idx++;
            }
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
    function genFunctioDeclaration(node: FunctioDeclaration): string {
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
     * Generate genus declaration as C++ struct with auto-merge constructor.
     *
     * TRANSFORMS:
     *   genus Persona { textus nomen: "anon" numerus aetas: 0 }
     *   -> struct Persona {
     *          std::string nomen = "anon";
     *          int64_t aetas = 0;
     *
     *          Persona() = default;
     *
     *          template<typename Overrides>
     *              requires std::is_aggregate_v<Overrides>
     *          Persona(const Overrides& o) {
     *              if constexpr (requires { o.nomen; }) nomen = o.nomen;
     *              if constexpr (requires { o.aetas; }) aetas = o.aetas;
     *              _creo();
     *          }
     *
     *          void _creo() { // user's creo body }
     *      };
     *
     * WHY: Auto-merge design - C++20's if constexpr (requires { ... }) pattern
     *      provides compile-time field checking, similar to Zig's @hasField.
     *      This allows `novum Persona { .nomen = "Marcus" }` to only override
     *      specified fields while keeping others at their defaults.
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

        // Always generate auto-merge constructor for struct types with fields
        if (node.fields.length > 0) {
            lines.push('');
            lines.push(genAutoMergeConstructor(node));
        }

        // User's creo as _creo() private method
        if (node.constructor) {
            lines.push('');
            lines.push(genCreoMethod(node.constructor));
        }

        // Methods
        for (const method of node.methods) {
            lines.push('');
            lines.push(genMethodDeclaration(method));
        }

        depth--;
        lines.push(`${ind()}};`);

        // WHY: Auto-merge constructor requires <type_traits> for std::is_aggregate_v
        if (node.fields.length > 0) {
            includes.add('<type_traits>');
        }

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
     * Generate auto-merge constructor using C++20 concepts.
     *
     * WHY: The template constructor accepts any aggregate type and uses
     *      if constexpr (requires { o.field; }) to check for field presence
     *      at compile time. This enables partial initialization like:
     *      `Persona{{.nomen = "Marcus"}}` - only nomen is overridden.
     */
    function genAutoMergeConstructor(node: GenusDeclaration): string {
        const name = node.name.name;
        const lines: string[] = [];
        const hasCreo = !!node.constructor;

        // Default constructor
        lines.push(`${ind()}${name}() = default;`);
        lines.push('');

        // Template auto-merge constructor
        lines.push(`${ind()}template<typename Overrides>`);
        lines.push(`${ind()}    requires std::is_aggregate_v<Overrides>`);
        lines.push(`${ind()}${name}(const Overrides& o) {`);
        depth++;

        // Apply each field override if present in the overrides struct
        for (const field of node.fields) {
            const fieldName = field.name.name;
            lines.push(`${ind()}if constexpr (requires { o.${fieldName}; }) ${fieldName} = o.${fieldName};`);
        }

        // Call _creo() if user defined it
        if (hasCreo) {
            lines.push(`${ind()}_creo();`);
        }

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate user's creo as a private _creo() method.
     *
     * WHY: creo() is a post-initialization hook. By the time it runs,
     *      the struct already has merged field values. Named _creo to
     *      avoid conflicts with user-defined methods.
     */
    function genCreoMethod(node: FunctioDeclaration): string {
        const lines: string[] = [];

        lines.push(`${ind()}void _creo() {`);
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
     * Generate method declaration.
     */
    function genMethodDeclaration(node: FunctioDeclaration): string {
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

    /**
     * Generate discretio (tagged union) as std::variant.
     *
     * TRANSFORMS:
     *   discretio Event { Click { numerus x, numerus y }, Quit }
     *   -> struct Click { int64_t x; int64_t y; };
     *      struct Quit {};
     *      using Event = std::variant<Click, Quit>;
     *
     * WHY: std::variant is C++17's type-safe union. Each variant becomes
     *      a struct, and we use std::variant to hold any of them.
     */
    function genDiscretioDeclaration(node: DiscretioDeclaration): string {
        includes.add('<variant>');

        const name = node.name.name;
        const lines: string[] = [];
        const variantNames: string[] = [];

        // Generate a struct for each variant
        for (const variant of node.variants) {
            const variantName = variant.name.name;
            variantNames.push(variantName);

            if (variant.fields.length === 0) {
                // Unit variant: empty struct
                lines.push(`${ind()}struct ${variantName} {};`);
            } else {
                // Variant with fields
                lines.push(`${ind()}struct ${variantName} {`);
                depth++;

                for (const field of variant.fields) {
                    const fieldName = field.name.name;
                    const fieldType = genType(field.fieldType);
                    lines.push(`${ind()}${fieldType} ${fieldName};`);
                }

                depth--;
                lines.push(`${ind()}};`);
            }
        }

        // Generate the variant type alias
        lines.push(`${ind()}using ${name} = std::variant<${variantNames.join(', ')}>;`);

        return lines.join('\n');
    }

    // -------------------------------------------------------------------------
    // Control Flow
    // -------------------------------------------------------------------------

    function genSiStatement(node: SiStatement): string {
        let result = `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`;

        if (node.alternate) {
            if (node.alternate.type === 'SiStatement') {
                result += ` else ${genSiStatement(node.alternate).trim()}`;
            } else {
                result += ` else ${genBlockStatement(node.alternate)}`;
            }
        }

        return result;
    }

    function genDumStatement(node: DumStatement): string {
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
    function genIteratioStatement(node: IteratioStatement): string {
        const varName = node.variable.name;
        const body = genBlockStatement(node.body);

        // Handle range expressions
        if (node.iterable.type === 'RangeExpression') {
            const range = node.iterable;
            const start = genExpression(range.start);
            const end = genExpression(range.end);
            const step = range.step ? genExpression(range.step) : '1';
            const cmp = range.inclusive ? '<=' : '<';

            return `${ind()}for (int64_t ${varName} = ${start}; ${varName} ${cmp} ${end}; ${varName} += ${step}) ${body}`;
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
    function genInStatement(node: InStatement): string {
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
     *
     * NOTE: Variant matching not yet implemented for C++.
     *       For now, only value cases are supported.
     */
    function genEligeStatement(node: EligeStatement): string {
        const discriminant = genExpression(node.discriminant);
        const lines: string[] = [];

        lines.push(`${ind()}switch (${discriminant}) {`);

        for (const caseNode of node.cases) {
            if (caseNode.type === 'EligeCasus') {
                // Value matching: si expression { ... }
                const test = genExpression(caseNode.test);

                lines.push(`${ind()}case ${test}: {`);
                depth++;

                for (const stmt of caseNode.consequent.body) {
                    lines.push(genStatement(stmt));
                }

                lines.push(`${ind()}break;`);
                depth--;
                lines.push(`${ind()}}`);
            } else {
                // Variant matching: ex VariantName pro bindings { ... }
                // TODO: Implement std::visit or variant pattern matching for C++
                lines.push(`${ind()}// TODO: Variant case ${caseNode.variant.name} not yet implemented for C++`);
            }
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

    function genCustodiStatement(node: CustodiStatement): string {
        const lines: string[] = [];

        for (const clause of node.clauses) {
            const test = genExpression(clause.test);
            const body = genBlockStatement(clause.consequent);

            lines.push(`${ind()}if (${test}) ${body}`);
        }

        return lines.join('\n');
    }

    function genAdfirmaStatement(node: AdfirmaStatement): string {
        includes.add('<cassert>');

        const test = genExpression(node.test);

        if (node.message) {
            // C++ assert doesn't support messages easily, use comma operator trick
            const msg = genExpression(node.message);

            return `${ind()}assert((${test}) && ${msg});`;
        }

        return `${ind()}assert(${test});`;
    }

    function genReddeStatement(node: ReddeStatement): string {
        if (node.argument) {
            return `${ind()}return ${genExpression(node.argument)};`;
        }

        return `${ind()}return;`;
    }

    function genIaceStatement(node: IaceStatement): string {
        // WHY: mori (fatal=true) is unrecoverable, iace (fatal=false) is catchable
        if (node.fatal) {
            includes.add('<cstdlib>');
            includes.add('<print>');

            // mori -> print message and abort
            if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
                return `${ind()}std::print(stderr, "FATAL: {}\\n", "${node.argument.value}"); std::abort();`;
            }

            const msg = genExpression(node.argument);

            return `${ind()}std::print(stderr, "FATAL: {}\\n", ${msg}); std::abort();`;
        }

        includes.add('<stdexcept>');

        // Handle string literals -> runtime_error
        if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
            return `${ind()}throw std::runtime_error("${node.argument.value}");`;
        }

        // Handle new Error("msg")
        if (node.argument.type === 'NovumExpression' && node.argument.callee.name === 'Error') {
            const firstArg = node.argument.arguments[0];
            const msg = firstArg && firstArg.type !== 'SpreadElement' ? genExpression(firstArg) : '""';

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

    function genTemptaStatement(node: TemptaStatement): string {
        const lines: string[] = [];

        // WHY: C++ doesn't have finally. We use a scope guard pattern:
        // The finally block runs via RAII destructor on scope exit.
        if (node.finalizer) {
            includes.add('<utility>');
            needsScopeGuard = true;

            // Generate a unique ID for this scope guard
            const guardId = `_demum_${scopeGuardCounter++}`;
            const finalizerCode = node.finalizer.body.map(stmt => genStatement(stmt).trim()).join(' ');

            lines.push(`${ind()}auto ${guardId} = _ScopeGuard([&]{ ${finalizerCode} });`);
        }

        lines.push(`${ind()}try ${genBlockStatement(node.block)}`);

        if (node.handler) {
            const param = node.handler.param.name;

            lines.push(`${ind()}catch (const std::exception& ${param}) ${genBlockStatement(node.handler.body)}`);
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
            case 'CedeExpression':
                // C++ doesn't have await, would need coroutines
                return genExpression(node.argument);
            case 'EgoExpression':
                return 'this';
            case 'NovumExpression':
                return genNovumExpression(node);
            case 'ConditionalExpression':
                return `(${genExpression(node.test)} ? ${genExpression(node.consequent)} : ${genExpression(node.alternate)})`;
            case 'LambdaExpression':
                return genLambdaExpression(node);
            case 'PraefixumExpression':
                return genPraefixumExpression(node);
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

        const elements = node.elements
            .filter((el): el is Expression => el.type !== 'SpreadElement')
            .map(genExpression)
            .join(', ');

        return `std::vector{${elements}}`;
    }

    function genObjectExpression(node: ObjectExpression): string {
        if (node.properties.length === 0) {
            return '{}';
        }

        // For simple structs, use designated initializers (C++20)
        const props = node.properties
            .filter(prop => prop.type !== 'SpreadElement')
            .map(prop => {
                const p = prop as any;
                const key = p.key.type === 'Identifier' ? p.key.name : String((p.key as Literal).value);
                const value = genExpression(p.value);

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

    /**
     * Generate praefixum (compile-time) expression.
     *
     * TRANSFORMS:
     *   praefixum(256 * 4) -> constexpr auto _praefixum_0 = (256 * 4); ... _praefixum_0
     *   praefixum { ... redde x } -> [&]{ ... return x; }()  (can't be constexpr for complex blocks)
     *
     * WHY: C++23 constexpr is limited to certain operations. For simple expressions,
     *      we use constexpr. For blocks, we emit an IIFE (which may not be constexpr).
     */
    function genPraefixumExpression(node: PraefixumExpression): string {
        if (node.body.type === 'BlockStatement') {
            // Block form: emit as IIFE
            // WHY: constexpr lambdas exist but have restrictions. For now, just use regular IIFE.
            const body = genBlockStatement(node.body);

            return `[&]${body}()`;
        }

        // Expression form: wrap in constexpr evaluation
        // WHY: In expression context, we can't declare a constexpr variable.
        //      Just parenthesize and trust the optimizer.
        return `(${genExpression(node.body)})`;
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
     *   lista.adde(x) -> lista.push_back(x)
     *   lista.filtrata(fn) -> (lista | views::filter(fn) | ranges::to<vector>())
     */
    function genCallExpression(node: CallExpression): string {
        const args = node.arguments
            .filter((arg): arg is Expression => arg.type !== 'SpreadElement')
            .map(genExpression)
            .join(', ');

        // Check for collection methods (method calls on lista/tabula/copia)
        if (node.callee.type === 'MemberExpression' && !node.callee.computed) {
            const methodName = (node.callee.property as Identifier).name;
            const obj = genExpression(node.callee.object);

            // WHY: Use semantic type info to dispatch to correct collection registry.
            // This prevents method name collisions (e.g., accipe means different
            // things for lista vs tabula).
            const objType = node.callee.object.resolvedType;
            const collectionName = objType?.kind === 'generic' ? objType.name : null;

            // Dispatch based on resolved type
            if (collectionName === 'tabula') {
                const method = getTabulaMethod(methodName);
                if (method) {
                    for (const header of getTabulaHeaders(methodName)) {
                        includes.add(header);
                    }
                    if (typeof method.cpp === 'function') {
                        return method.cpp(obj, args);
                    }
                    return `${obj}.${method.cpp}(${args})`;
                }
            } else if (collectionName === 'copia') {
                const method = getCopiaMethod(methodName);
                if (method) {
                    for (const header of getCopiaHeaders(methodName)) {
                        includes.add(header);
                    }
                    if (typeof method.cpp === 'function') {
                        return method.cpp(obj, args);
                    }
                    return `${obj}.${method.cpp}(${args})`;
                }
            } else if (collectionName === 'lista') {
                const method = getListaMethod(methodName);
                if (method) {
                    for (const header of getListaHeaders(methodName)) {
                        includes.add(header);
                    }
                    if (typeof method.cpp === 'function') {
                        return method.cpp(obj, args);
                    }
                    return `${obj}.${method.cpp}(${args})`;
                }
            }

            // Fallback: no type info - try lista (most common)
            const listaMethod = getListaMethod(methodName);
            if (listaMethod) {
                for (const header of getListaHeaders(methodName)) {
                    includes.add(header);
                }
                if (typeof listaMethod.cpp === 'function') {
                    return listaMethod.cpp(obj, args);
                }
                return `${obj}.${listaMethod.cpp}(${args})`;
            }
        }

        const callee = genExpression(node.callee);

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
            // Check for slice syntax: arr[1..3] or arr[1 usque 3]
            if (node.property.type === 'RangeExpression') {
                return genSliceExpression(obj, node.property);
            }

            // Check for negative index - C++ doesn't support negative indices
            if (isNegativeIndex(node.property)) {
                const negExpr = node.property as UnaryExpression;
                const absVal = genExpression(negExpr.argument);
                return `${obj}[${obj}.size() - ${absVal}]`;
            }

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
     * Check if an expression is a negative numeric literal.
     */
    function isNegativeIndex(expr: Expression): boolean {
        if (expr.type === 'UnaryExpression' && expr.operator === '-' && expr.argument.type === 'Literal') {
            return typeof expr.argument.value === 'number';
        }
        if (expr.type === 'Literal' && typeof expr.value === 'number' && expr.value < 0) {
            return true;
        }
        return false;
    }

    /**
     * Generate slice expression from range inside brackets.
     *
     * TRANSFORMS:
     *   arr[1..3]       -> std::vector<T>(arr.begin() + 1, arr.begin() + 3)
     *   arr[1 usque 3]  -> std::vector<T>(arr.begin() + 1, arr.begin() + 4)
     *   arr[-3..-1]     -> std::vector<T>(arr.end() - 3, arr.end() - 1)
     *
     * WHY: C++ doesn't have native slice syntax; use vector range constructor.
     *      This creates a copy (which matches the semantics of other targets).
     */
    function genSliceExpression(obj: string, range: RangeExpression): string {
        includes.add('<vector>');

        let startExpr: string;
        let endExpr: string;

        // Handle start
        if (isNegativeIndex(range.start)) {
            const negExpr = range.start as UnaryExpression;
            const absVal = genExpression(negExpr.argument);
            startExpr = `${obj}.end() - ${absVal}`;
        } else {
            startExpr = `${obj}.begin() + ${genExpression(range.start)}`;
        }

        // Handle end
        if (isNegativeIndex(range.end)) {
            const negExpr = range.end as UnaryExpression;
            const absVal = (negExpr.argument as Literal).value as number;
            if (range.inclusive) {
                const inclusiveEnd = absVal - 1;
                if (inclusiveEnd === 0) {
                    endExpr = `${obj}.end()`;
                } else {
                    endExpr = `${obj}.end() - ${inclusiveEnd}`;
                }
            } else {
                endExpr = `${obj}.end() - ${absVal}`;
            }
        } else if (range.inclusive) {
            if (range.end.type === 'Literal' && typeof range.end.value === 'number') {
                endExpr = `${obj}.begin() + ${range.end.value + 1}`;
            } else {
                endExpr = `${obj}.begin() + ${genExpression(range.end)} + 1`;
            }
        } else {
            endExpr = `${obj}.begin() + ${genExpression(range.end)}`;
        }

        // Use decltype to infer element type
        return `std::vector<typename decltype(${obj})::value_type>(${startExpr}, ${endExpr})`;
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

    /**
     * Generate Latin lambda expression (pro x redde expr).
     *
     * TRANSFORMS:
     *   pro x: x + 1 -> [](auto x) { return x + 1; }
     *   pro x -> numerus: x + 1 -> [](auto x) -> int64_t { return x + 1; }
     *
     * TARGET: C++ lambdas support trailing return types with ->
     */
    function genLambdaExpression(node: LambdaExpression): string {
        const params = node.params.map(p => `auto ${p.name}`).join(', ');

        // Add return type annotation if present
        const returnTypeAnno = node.returnType ? ` -> ${genType(node.returnType)}` : '';

        if (node.body.type === 'BlockStatement') {
            const body = genBlockStatement(node.body);

            return `[&](${params})${returnTypeAnno} ${body}`;
        }

        const body = genExpression(node.body as Expression);

        return `[&](${params})${returnTypeAnno} { return ${body}; }`;
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
    function genNovumExpression(node: NovumExpression): string {
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
        const args = node.arguments
            .filter((arg): arg is Expression => arg.type !== 'SpreadElement')
            .map(genExpression)
            .join(', ');

        return `${callee}(${args})`;
    }

    return genProgram(program);
}
