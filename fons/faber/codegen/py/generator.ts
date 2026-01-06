/**
 * Python Code Generator - Core generator class
 *
 * Holds shared state and utilities for Python code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter, BaseNode } from '../../parser/ast';
import type { RequiredFeatures } from '../types';
import { createRequiredFeatures, COMMENT_SYNTAX, formatLeadingComments, formatTrailingComments } from '../types';

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
import { genLiteral } from './expressions/literal';
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
import { genEstExpression } from './expressions/est';
import { genPraefixumExpression } from './expressions/praefixum';
import { genScriptumExpression } from './expressions/scriptum';
import { genRegexLiteral } from './expressions/regex';

/**
 * Map Latin type names to Python type hints.
 */
const typeMap: Record<string, string> = {
    textus: 'str',
    numerus: 'int',
    fractus: 'float',
    decimus: 'Decimal',
    magnus: 'int',
    bivalens: 'bool',
    nihil: 'None',
    vacuum: 'None',
    numquam: 'NoReturn',
    octeti: 'bytes',
    objectum: 'object',
    object: 'object',
    lista: 'list',
    tabula: 'dict',
    copia: 'set',
    promissum: 'Awaitable',
    erratum: 'Exception',
    cursor: 'Iterator',
    ignotum: 'Any',
};

export class PyGenerator {
    depth = 0;
    inGenerator = false;
    features: RequiredFeatures;

    constructor(public indent: string = '    ') {
        this.features = createRequiredFeatures();
    }

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
        return formatLeadingComments(node, COMMENT_SYNTAX.py, this.ind());
    }

    /**
     * Format trailing comments for a node.
     */
    trailingComments(node: BaseNode): string {
        return formatTrailingComments(node, COMMENT_SYNTAX.py);
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
                return `${this.ind()}${this.genExpression(node.expression)}`;
            case 'AdStatement':
                throw new Error('AdStatement codegen not implemented for Python');
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

            // Handle operator mapping for Python
            let op = node.operator;
            if (op === '&&') {
                op = 'and';
            } else if (op === '||') {
                op = 'or';
            } else if (op === '===') {
                op = '==';
            } else if (op === '!==') {
                op = '!=';
            }

            // WHY: Python has no ?? operator; use conditional expression
            if (node.operator === '??') {
                return `${left} if ${left} is not None else ${right}`;
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
                // WHY: Python f-strings use {expr} not ${expr}
                return `f"${node.raw.replace(/\$\{/g, '{')}"`;

            case 'EgoExpression':
                return 'self';
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
                return `${this.inGenerator ? 'yield' : 'await'} ${this.genExpression(node.argument)}`;
            case 'NovumExpression':
                return genNovumExpression(node, this);
            case 'FingeExpression':
                return genFingeExpression(node, this);
            case 'ConditionalExpression':
                return `${this.genExpression(node.consequent)} if ${this.genExpression(node.test)} else ${this.genExpression(node.alternate)}`;
            case 'QuaExpression':
                // WHY: Python is dynamically typed, type casts have no runtime effect.
                return this.genExpression(node.expression);
            case 'InnatumExpression':
                // WHY: Python uses {} for dict (tabula) and [] for list (lista) natively.
                // Non-empty object literals work directly as dicts.
                return this.genExpression(node.expression);
            case 'ConversionExpression':
                return genConversionExpression(node, this);
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
     * Generate block statement content (without braces).
     */
    genBlockStatementContent(node: BlockStatement): string {
        if (node.body.length === 0) {
            return `${this.ind()}pass`;
        }
        return node.body.map(stmt => this.genStatement(stmt)).join('\n');
    }

    /**
     * Generate function parameter.
     *
     * WHY: Dual naming (textus location ut loc) uses internal name (alias) in generated code.
     *      The external name is for callsite documentation only.
     *      Default values (vel) generate = default syntax.
     *      Optional params (si) without default generate name: type | None = None.
     */
    genParameter(node: Parameter): string {
        // Use alias (internal name) if present, otherwise external name
        const name = node.alias?.name ?? node.name.name;
        const prefix = node.rest ? '*' : '';

        // Handle optional parameters
        if (node.optional && !node.defaultValue) {
            // Optional without default: name: type | None = None
            const typeAnno = node.typeAnnotation ? `: ${this.genType(node.typeAnnotation)} | None` : '';
            return `${prefix}${name}${typeAnno} = None`;
        }

        const typeAnno = node.typeAnnotation ? `: ${this.genType(node.typeAnnotation)}` : '';
        const defaultVal = node.defaultValue ? ` = ${this.genExpression(node.defaultValue)}` : '';
        return `${prefix}${name}${typeAnno}${defaultVal}`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        // Track feature usage for preamble
        if (node.name === 'decimus' || node.name === 'decim') {
            this.features.decimal = true;
        }

        // Map Latin type name to Python type
        const base = typeMap[node.name] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${base}[${params.join(', ')}]`;
            }
        }

        // Handle nullable: textus? -> str | None
        if (node.nullable) {
            result = `${result} | None`;
        }

        // Handle union types
        if (node.union && node.union.length > 0) {
            result = node.union.map(t => this.genType(t)).join(' | ');
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

        // Ignore numeric parameters (e.g., numerus<32>)
        if (param.type === 'Literal') {
            return null;
        }

        return null;
    }
}
