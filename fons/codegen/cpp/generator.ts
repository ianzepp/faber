/**
 * C++23 Code Generator - Core generator class
 *
 * Holds shared state and utilities for C++ code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter } from '../../parser/ast';

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
import { genArrowFunction } from './expressions/arrow';
import { genLambdaExpression } from './expressions/lambda';
import { genAssignmentExpression } from './expressions/assignment';
import { genNovumExpression } from './expressions/novum';
import { genEstExpression } from './expressions/est';
import { genPraefixumExpression } from './expressions/praefixum';
import { genScriptumExpression } from './expressions/scriptum';

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
     * Generate a statement. Dispatches to specific gen* functions.
     */
    genStatement(node: Statement): string {
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
            if (op === '===') op = '==';
            else if (op === '!==') op = '!=';

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
            case 'ArrowFunctionExpression':
                return genArrowFunction(node, this);
            case 'LambdaExpression':
                return genLambdaExpression(node, this);
            case 'AssignmentExpression':
                return genAssignmentExpression(node, this);
            case 'CedeExpression':
                // C++ doesn't have await, would need coroutines
                return this.genExpression(node.argument);
            case 'NovumExpression':
                return genNovumExpression(node, this);
            case 'ConditionalExpression':
                return `(${this.genExpression(node.test)} ? ${this.genExpression(node.consequent)} : ${this.genExpression(node.alternate)})`;
            case 'QuaExpression':
                // WHY: C++ can use static_cast for type casts
                return `static_cast<${this.genType(node.targetType)}>(${this.genExpression(node.expression)})`;
            case 'EstExpression':
                return genEstExpression(node, this);
            case 'PraefixumExpression':
                return genPraefixumExpression(node, this);
            case 'ScriptumExpression':
                return genScriptumExpression(node, this);
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
     * WHY: C++ uses type-first syntax: "const std::string& name"
     *      Rest parameters use initializer_list or variadic templates.
     *      Latin prepositions encode reference semantics:
     *      de = "from/concerning" = const reference (read-only)
     *      in = "into" = mutable reference (will be modified)
     */
    genParameter(node: Parameter): string {
        const name = node.name.name;
        const preposition = node.preposition;

        if (node.typeAnnotation) {
            const type = this.genType(node.typeAnnotation);

            // Explicit prepositions override default behavior
            if (preposition === 'de') {
                // de = const reference (borrowed, read-only)
                return `const ${type}& ${name}`;
            }

            if (preposition === 'in') {
                // in = mutable reference (will be modified)
                return `${type}& ${name}`;
            }

            // Default: pass strings and vectors by const reference
            if (type === 'std::string' || type.startsWith('std::vector')) {
                return `const ${type}& ${name}`;
            }

            // Rest parameters become initializer_list
            if (node.rest) {
                return `std::initializer_list<${type}> ${name}`;
            }

            return `${type} ${name}`;
        }

        // No type annotation - use auto (requires C++20 abbreviated function template)
        // Still respect prepositions
        if (preposition === 'de') {
            return `const auto& ${name}`;
        }

        if (preposition === 'in') {
            return `auto& ${name}`;
        }

        return `auto ${name}`;
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
}
