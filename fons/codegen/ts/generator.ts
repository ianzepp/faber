/**
 * TypeScript Code Generator - Core generator class
 *
 * Holds shared state and utilities for TypeScript code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter, TypeParameterDeclaration } from '../../parser/ast';
import type { RequiredFeatures } from '../types';
import { createRequiredFeatures } from '../types';

// Statement handlers
import { genImportaDeclaration } from './statements/importa';
import { genDestructureDeclaration } from './statements/destructure';
import { genVariaDeclaration } from './statements/varia';
import { genFunctioDeclaration, genBlockStatement } from './statements/functio';
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
import { genProbandumStatement } from './statements/probandum';
import { genProbaStatement } from './statements/proba';
import { genCuraBlock, genCuraStatement } from './statements/cura';

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
import { genArrowFunction } from './expressions/arrow';
import { genLambdaExpression } from './expressions/lambda';
import { genAssignmentExpression } from './expressions/assignment';
import { genNovumExpression } from './expressions/novum';
import { genQuaExpression } from './expressions/qua';
import { genEstExpression } from './expressions/est';
import { genPraefixumExpression } from './expressions/praefixum';
import { genScriptumExpression } from './expressions/scriptum';
import { genCollectionDSLExpression } from './expressions/collection-dsl';

/**
 * Map Latin type names to TypeScript types.
 */
const typeMap: Record<string, string> = {
    textus: 'string',
    numerus: 'number',
    fractus: 'number',
    decimus: 'Decimal',
    magnus: 'bigint',
    bivalens: 'boolean',
    nihil: 'null',
    vacuum: 'void',
    numquam: 'never',
    octeti: 'Uint8Array',
    lista: 'Array',
    tabula: 'Map',
    copia: 'Set',
    promissum: 'Promise',
    erratum: 'Error',
    cursor: 'Iterator',
    objectum: 'object',
    object: 'object',
    ignotum: 'unknown',
};

export class TsGenerator {
    depth = 0;
    inGenerator = false;
    features: RequiredFeatures;
    semi: boolean;

    constructor(
        public indent: string = '  ',
        semi: boolean = true,
    ) {
        this.features = createRequiredFeatures();
        this.semi = semi;
    }

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
                return genImportaDeclaration(node, this, this.semi);
            case 'DestructureDeclaration':
                return genDestructureDeclaration(node, this, this.semi);
            case 'VariaDeclaration':
                return genVariaDeclaration(node, this, this.semi);
            case 'FunctioDeclaration':
                return genFunctioDeclaration(node, this);
            case 'GenusDeclaration':
                return genGenusDeclaration(node, this, this.semi);
            case 'PactumDeclaration':
                return genPactumDeclaration(node, this, this.semi);
            case 'TypeAliasDeclaration':
                return genTypeAliasDeclaration(node, this, this.semi);
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
                return genInStatement(node, this, this.semi);
            case 'EligeStatement':
                return genEligeStatement(node, this);
            case 'DiscerneStatement':
                return genDiscerneStatement(node, this);
            case 'CustodiStatement':
                return genCustodiStatement(node, this);
            case 'AdfirmaStatement':
                return genAdfirmaStatement(node, this);
            case 'ReddeStatement':
                return genReddeStatement(node, this, this.semi);
            case 'RumpeStatement':
                return genRumpeStatement(this, this.semi);
            case 'PergeStatement':
                return genPergeStatement(this, this.semi);
            case 'IaceStatement':
                return genIaceStatement(node, this, this.semi);
            case 'ScribeStatement':
                return genScribeStatement(node, this, this.semi);
            case 'TemptaStatement':
                return genTemptaStatement(node, this);
            case 'FacBlockStatement':
                return genFacBlockStatement(node, this);
            case 'BlockStatement':
                return genBlockStatement(node, this);
            case 'ExpressionStatement':
                return `${this.ind()}${this.genExpression(node.expression)}${this.semi ? ';' : ''}`;
            case 'ProbandumStatement':
                return genProbandumStatement(node, this, this.semi);
            case 'ProbaStatement':
                return genProbaStatement(node, this, this.semi);
            case 'CuraBlock':
                return genCuraBlock(node, this, this.semi);
            case 'CuraStatement':
                return genCuraStatement(node, this, this.semi);
            case 'AdStatement':
                throw new Error('AdStatement codegen not implemented for TypeScript');
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
            return `${left} ${node.operator} ${right}`;
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
                return `\`${node.raw}\``;
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
                // WHY: cede maps to yield in generators, await in async functions
                return `${this.inGenerator ? 'yield' : 'await'} ${this.genExpression(node.argument)}`;
            case 'NovumExpression':
                return genNovumExpression(node, this);
            case 'ConditionalExpression':
                return `${this.genExpression(node.test)} ? ${this.genExpression(node.consequent)} : ${this.genExpression(node.alternate)}`;
            case 'QuaExpression':
                return genQuaExpression(node, this);
            case 'EstExpression':
                return genEstExpression(node, this);
            case 'PraefixumExpression':
                return genPraefixumExpression(node, this);
            case 'ScriptumExpression':
                return genScriptumExpression(node, this);
            case 'CollectionDSLExpression':
                return genCollectionDSLExpression(node, this);
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
     */
    genParameter(node: Parameter): string {
        const name = node.name.name;
        const typeAnno = node.typeAnnotation ? `: ${this.genType(node.typeAnnotation)}` : '';
        const prefix = node.rest ? '...' : '';
        return `${prefix}${name}${typeAnno}`;
    }

    /**
     * Generate type parameters (generics).
     *
     * WHY: TypeParameterDeclaration[] is an array of individual params,
     *      each with a name Identifier. We join them into <T, U, V> syntax.
     */
    genTypeParams(params: TypeParameterDeclaration[]): string {
        return `<${params.map(p => p.name.name).join(', ')}>`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        // Track feature usage for preamble
        if (node.name === 'decimus' || node.name === 'decim') {
            this.features.decimal = true;
        }

        // Map Latin type name to TypeScript type
        const base = typeMap[node.name] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${base}<${params.join(', ')}>`;
            }
        }

        // Handle nullable: textus? -> string | null
        if (node.nullable) {
            result = `${result} | null`;
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
