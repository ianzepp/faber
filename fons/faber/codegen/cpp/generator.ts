/**
 * C++23 Code Generator - Core generator class
 *
 * Holds shared state and utilities for C++ code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter, BaseNode } from '../../parser/ast';
import { COMMENT_SYNTAX, formatLeadingComments, formatTrailingComments } from '../types';

// Statement handlers
import { genImportaDeclaration } from './statements/importa';
import { genDestructureDeclaration } from './statements/destructure';
import { genVariaDeclaration } from './statements/varia';
import { genFunctioDeclaration } from './statements/functio';
import { genGenusDeclaration } from './statements/genus';
import { genPactumDeclaration } from './statements/pactum';
import { genTypeAliasDeclaration } from './statements/typealias';
import { genOrdoDeclaration } from './statements/ordo';
import { genDiscretioDeclaration } from './statements/discretio';
import { genSiStatement } from './statements/si';
import { genDumStatement } from './statements/dum';
import { genIteratioStatement } from './statements/iteratio';
import { genInStatement } from './statements/in';
import { genEligeStatement } from './statements/elige';
import { genDiscerneStatement } from './statements/discerne';
import { genCustodiStatement } from './statements/custodi';
import { genAdfirmaStatement } from './statements/adfirma';
import { genReddeStatement } from './statements/redde';
import { genRumpeStatement } from './statements/rumpe';
import { genPergeStatement } from './statements/perge';
import { genIaceStatement } from './statements/iace';
import { genScribeStatement } from './statements/scribe';
import { genTemptaStatement } from './statements/tempta';
import { genFacBlockStatement } from './statements/fac';
import { genIncipitStatement } from './statements/incipit';
import { genIncipietStatement } from './statements/incipiet';
import { genPraeparaBlock, genCuraStatement } from './statements/cura';

// Expression handlers
import { genIdentifier } from './expressions/identifier';
import { genLiteral, genTemplateLiteral } from './expressions/literal';
import { genArrayExpression } from './expressions/array';
import { genObjectExpression } from './expressions/object';
import { genRangeExpression } from './expressions/range';
import { genBinaryExpression } from './expressions/binary';
import { genUnaryExpression } from './expressions/unary';
import { genCallExpression } from './expressions/call';
import { genMemberExpression } from './expressions/member';
import { genLambdaExpression } from './expressions/lambda';
import { genAssignmentExpression } from './expressions/assignment';
import { genNovumExpression } from './expressions/novum';
import { genFingeExpression } from './expressions/finge';
import { genConversionExpression } from './expressions/conversio';
import { genShiftExpression } from './expressions/shift';
import { genEstExpression } from './expressions/est';
import { genPraefixumExpression } from './expressions/praefixum';
import { genScriptumExpression } from './expressions/scriptum';
import { genRegexLiteral } from './expressions/regex';

/**
 * Map Latin type names to C++23 types.
 */
const typeMap: Record<string, string> = {
    textus: 'std::string',
    numerus: 'int64_t',
    fractus: 'double',
    decimus: 'double',
    bivalens: 'bool',
    nihil: 'void',
    vacuum: 'void',
    octeti: 'std::vector<uint8_t>',
    lista: 'std::vector',
    tabula: 'std::unordered_map',
    copia: 'std::unordered_set',
    promissum: 'std::future',
    erratum: 'std::runtime_error',
    objectum: 'std::any',
};

export class CppGenerator {
    depth = 0;
    inGenerator = false;

    // Track which headers are needed
    includes = new Set<string>();

    // Track whether we need the scope guard helper for demum (finally)
    needsScopeGuard = false;
    scopeGuardCounter = 0;

    constructor(public indent: string = '    ') {}

    /**
     * Generate indentation for current depth.
     */
    ind(): string {
        return this.indent.repeat(this.depth);
    }

    /**
     * Format leading comments for a node.
     */
    leadingComments(node: BaseNode): string {
        return formatLeadingComments(node, COMMENT_SYNTAX.cpp, this.ind());
    }

    /**
     * Format trailing comments for a node.
     */
    trailingComments(node: BaseNode): string {
        return formatTrailingComments(node, COMMENT_SYNTAX.cpp);
    }

    /**
     * Generate a statement. Dispatches to specific gen* functions.
     */
    genStatement(node: Statement): string {
        const leading = this.leadingComments(node);
        const trailing = this.trailingComments(node);
        const stmt = this.genStatementContent(node);
        return `${leading}${stmt}${trailing}`;
    }

    /**
     * Generate statement content without comments.
     */
    private genStatementContent(node: Statement): string {
        switch (node.type) {
            case 'ImportaDeclaration':
                return genImportaDeclaration(node, this);
            case 'DestructureDeclaration':
                return genDestructureDeclaration(node, this);
            case 'VariaDeclaration':
                return genVariaDeclaration(node, this);
            case 'FunctioDeclaration':
                return genFunctioDeclaration(node, this);
            case 'GenusDeclaration':
                return genGenusDeclaration(node, this);
            case 'PactumDeclaration':
                return genPactumDeclaration(node, this);
            case 'TypeAliasDeclaration':
                return genTypeAliasDeclaration(node, this);
            case 'OrdoDeclaration':
                return genOrdoDeclaration(node, this);
            case 'DiscretioDeclaration':
                return genDiscretioDeclaration(node, this);
            case 'SiStatement':
                return genSiStatement(node, this);
            case 'DumStatement':
                return genDumStatement(node, this);
            case 'IteratioStatement':
                return genIteratioStatement(node, this);
            case 'InStatement':
                return genInStatement(node, this);
            case 'EligeStatement':
                return genEligeStatement(node, this);
            case 'DiscerneStatement':
                return genDiscerneStatement(node, this);
            case 'CustodiStatement':
                return genCustodiStatement(node, this);
            case 'AdfirmaStatement':
                return genAdfirmaStatement(node, this);
            case 'ReddeStatement':
                return genReddeStatement(node, this);
            case 'RumpeStatement':
                return genRumpeStatement(this);
            case 'PergeStatement':
                return genPergeStatement(this);
            case 'IaceStatement':
                return genIaceStatement(node, this);
            case 'ScribeStatement':
                return genScribeStatement(node, this);
            case 'TemptaStatement':
                return genTemptaStatement(node, this);
            case 'FacBlockStatement':
                return genFacBlockStatement(node, this);
            case 'IncipitStatement':
                return genIncipitStatement(node, this);
            case 'IncipietStatement':
                return genIncipietStatement(node, this);
            case 'PraeparaBlock':
                return genPraeparaBlock(node, this);
            case 'CuraStatement':
                return genCuraStatement(node, this);
            case 'BlockStatement':
                return this.genBlockStatementContent(node);
            case 'ExpressionStatement':
                return `${this.ind()}${this.genExpression(node.expression)};`;
            case 'AdStatement':
                throw new Error('AdStatement codegen not implemented for C++');
            default:
                throw new Error(`Unknown statement type: ${(node as any).type}`);
        }
    }

    /**
     * Generate an expression without outer parentheses.
     *
     * WHY: Binary expressions are wrapped in parens for safety, but in some
     *      contexts (array index, RHS of assignment) the parens are unnecessary.
     *      For binary expressions, this generates the expression flat without
     *      wrapping parens, recursively stripping nested binary expression parens.
     */
    genBareExpression(node: Expression): string {
        // For binary expressions, generate without outer parens
        // and recursively generate operands bare to flatten chains like a + b + c
        if (node.type === 'BinaryExpression') {
            const left = this.genBareExpression(node.left);
            const right = this.genBareExpression(node.right);

            // Handle operator mapping for C++
            let op = node.operator;
            if (op === '===') {
                op = '==';
            } else if (op === '!==') {
                op = '!=';
            }

            // WHY: C++ has no ?? operator; use ternary with nullptr check
            if (node.operator === '??') {
                return `${left} != nullptr ? ${left} : ${right}`;
            }

            return `${left} ${op} ${right}`;
        }

        return this.genExpression(node);
    }

    /**
     * Generate an expression. Dispatches to specific gen* functions.
     */
    genExpression(node: Expression): string {
        switch (node.type) {
            case 'Identifier':
                return genIdentifier(node, this);
            case 'Literal':
                return genLiteral(node, this);
            case 'TemplateLiteral':
                return genTemplateLiteral(node, this);
            case 'EgoExpression':
                return 'this';
            case 'ArrayExpression':
                return genArrayExpression(node, this);
            case 'ObjectExpression':
                return genObjectExpression(node, this);
            case 'RangeExpression':
                return genRangeExpression(node, this);
            case 'BinaryExpression':
                return genBinaryExpression(node, this);
            case 'UnaryExpression':
                return genUnaryExpression(node, this);
            case 'CallExpression':
                return genCallExpression(node, this);
            case 'MemberExpression':
                return genMemberExpression(node, this);
            case 'LambdaExpression':
                return genLambdaExpression(node, this);
            case 'AssignmentExpression':
                return genAssignmentExpression(node, this);
            case 'CedeExpression':
                // C++ doesn't have await, would need coroutines
                return this.genExpression(node.argument);
            case 'NovumExpression':
                return genNovumExpression(node, this);
            case 'FingeExpression':
                return genFingeExpression(node, this);
            case 'ConditionalExpression':
                return `(${this.genExpression(node.test)} ? ${this.genExpression(node.consequent)} : ${this.genExpression(node.alternate)})`;
            case 'QuaExpression':
                // WHY: C++ can use static_cast for type casts
                return `static_cast<${this.genType(node.targetType)}>(${this.genExpression(node.expression)})`;
            case 'InnatumExpression': {
                // WHY: C++ uses {} initialization for std::map and std::vector
                const typeName = node.targetType.name;
                if (typeName === 'tabula') {
                    const typeParams = node.targetType.typeParameters;
                    // WHY: TypeParameter can be TypeAnnotation | Literal, only use TypeAnnotation
                    const keyParam = typeParams?.[0];
                    const valueParam = typeParams?.[1];
                    const keyType = keyParam?.type === 'TypeAnnotation' ? this.genType(keyParam) : 'auto';
                    const valueType = valueParam?.type === 'TypeAnnotation' ? this.genType(valueParam) : 'auto';

                    // Handle non-empty object literal
                    if (node.expression.type === 'ObjectExpression' && node.expression.properties.length > 0) {
                        const entries = node.expression.properties.map(prop => {
                            if (prop.type === 'SpreadElement') {
                                return this.genExpression(prop.argument);
                            }
                            // WHY: key is Identifier | Literal - use name for identifiers, value for literals
                            const key = prop.key.type === 'Identifier' ? `"${prop.key.name}"` : `"${(prop.key as any).value}"`;
                            const value = this.genExpression(prop.value);
                            return `{${key}, ${value}}`;
                        }).join(', ');
                        return `std::map<${keyType}, ${valueType}>{${entries}}`;
                    }
                    return `std::map<${keyType}, ${valueType}>{}`;
                }
                if (typeName === 'lista') {
                    const typeParams = node.targetType.typeParameters;
                    const elemParam = typeParams?.[0];
                    const elemType = elemParam?.type === 'TypeAnnotation' ? this.genType(elemParam) : 'auto';

                    // Handle non-empty array literal
                    if (node.expression.type === 'ArrayExpression' && node.expression.elements.length > 0) {
                        // WHY: Filter SpreadElement which C++ doesn't support in initializer list
                        const elements = node.expression.elements
                            .filter(el => el.type !== 'SpreadElement')
                            .map(el => this.genExpression(el as Expression))
                            .join(', ');
                        return `std::vector<${elemType}>{${elements}}`;
                    }
                    return `std::vector<${elemType}>{}`;
                }
                return this.genExpression(node.expression);
            }
            case 'ConversionExpression':
                return genConversionExpression(node, this);
            case 'ShiftExpression':
                return genShiftExpression(node, this);
            case 'EstExpression':
                return genEstExpression(node, this);
            case 'PraefixumExpression':
                return genPraefixumExpression(node, this);
            case 'ScriptumExpression':
                return genScriptumExpression(node, this);
            case 'RegexLiteral':
                return genRegexLiteral(node, this);
            default:
                throw new Error(`Unknown expression type: ${(node as any).type}`);
        }
    }

    /**
     * Generate block statement content (statements only, no braces).
     */
    genBlockStatementContent(node: BlockStatement): string {
        return node.body.map(stmt => this.genStatement(stmt)).join('\n');
    }

    /**
     * Generate function parameter.
     *
     * WHY: C++ uses const references for complex types, value for primitives.
     *      Rest parameters use initializer_list or variadic templates.
     *      Latin prepositions encode reference semantics:
     *      de = "from/concerning" = const reference (read-only)
     *      in = "into" = mutable reference (will be modified)
     *      Dual naming (textus location ut loc) uses internal name (alias) in generated code.
     *      Default values (vel) generate = default syntax.
     *      Optional params (si) without default use std::optional<T> = std::nullopt.
     */
    genParameter(node: Parameter): string {
        // Use alias (internal name) if present, otherwise external name
        const name = node.alias?.name ?? node.name.name;
        const preposition = node.preposition;
        const defaultVal = node.defaultValue ? ` = ${this.genExpression(node.defaultValue)}` : '';

        if (node.typeAnnotation) {
            let type = this.genType(node.typeAnnotation);

            // Optional without default: wrap in std::optional
            if (node.optional && !node.defaultValue) {
                this.includes.add('<optional>');
                type = `std::optional<${type}>`;
                return `${type} ${name} = std::nullopt`;
            }

            // Explicit prepositions override default behavior
            if (preposition === 'de') {
                // de = const reference (borrowed, read-only)
                return `const ${type}& ${name}${defaultVal}`;
            }

            if (preposition === 'in') {
                // in = mutable reference (will be modified)
                return `${type}& ${name}${defaultVal}`;
            }

            // Default: pass complex types by const reference
            // WHY: Strings, vectors, sets, maps, and other STL containers should
            //      be passed by const ref to avoid expensive copies.
            if (this.isComplexType(type)) {
                return `const ${type}& ${name}${defaultVal}`;
            }

            // Rest parameters become initializer_list
            if (node.rest) {
                return `std::initializer_list<${type}> ${name}${defaultVal}`;
            }

            return `${type} ${name}${defaultVal}`;
        }

        // No type annotation - use auto (requires C++20 abbreviated function template)
        // Handle optional without type
        if (node.optional && !node.defaultValue) {
            this.includes.add('<optional>');
            return `std::optional<auto> ${name} = std::nullopt`;
        }

        // Still respect prepositions
        if (preposition === 'de') {
            return `const auto& ${name}${defaultVal}`;
        }

        if (preposition === 'in') {
            return `auto& ${name}${defaultVal}`;
        }

        return `auto ${name}${defaultVal}`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        // Map Latin type name to C++ type
        const base = typeMap[node.name] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${base}<${params.join(', ')}>`;
            }
        }

        // Handle nullable: textus? -> std::optional<std::string>
        if (node.nullable) {
            this.includes.add('<optional>');
            result = `std::optional<${result}>`;
        }

        // Add includes based on type
        if (base === 'std::string') {
            this.includes.add('<string>');
        }

        if (base === 'std::vector') {
            this.includes.add('<vector>');
        }

        if (base === 'std::unordered_map') {
            this.includes.add('<unordered_map>');
        }

        if (base === 'std::unordered_set') {
            this.includes.add('<unordered_set>');
        }

        return result;
    }

    /**
     * Generate type parameter.
     */
    genTypeParameter(param: TypeParameter): string | null {
        if (param.type === 'TypeAnnotation') {
            return this.genType(param);
        }

        // Numeric parameters (e.g., for fixed-size arrays)
        if (param.type === 'Literal' && typeof param.value === 'number') {
            return String(param.value);
        }

        return null;
    }

    /**
     * Check if a C++ type should be passed by const reference.
     *
     * WHY: STL containers and strings are expensive to copy. Pass by const ref
     *      unless explicitly mutating (in preposition).
     */
    private isComplexType(type: string): boolean {
        // Primitive types that should be passed by value
        const valueTypes = ['int', 'int64_t', 'double', 'bool', 'char', 'void', 'auto'];
        if (valueTypes.includes(type)) {
            return false;
        }

        // STL containers and string should be passed by const ref
        if (type === 'std::string') {
            return true;
        }
        if (type.startsWith('std::vector')) {
            return true;
        }
        if (type.startsWith('std::unordered_set')) {
            return true;
        }
        if (type.startsWith('std::unordered_map')) {
            return true;
        }
        if (type.startsWith('std::optional')) {
            return true;
        }
        if (type.startsWith('std::array')) {
            return true;
        }

        // User-defined types (anything not in valueTypes) - pass by const ref
        // WHY: Unknown types are likely structs/classes that benefit from ref passing
        return true;
    }
}
