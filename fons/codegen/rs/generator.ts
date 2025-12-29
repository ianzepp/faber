/**
 * Rust Code Generator - Core generator class
 *
 * Holds shared state and utilities for Rust code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter } from '../../parser/ast';

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
import { genExpressionStatement } from './statements/expression';
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

/**
 * Map Latin type names to Rust type names.
 */
const typeMap: Record<string, string> = {
    textus: 'String',
    numerus: 'f64',
    fractus: 'f64',
    magnus: 'i128',
    bivalens: 'bool',
    nihil: '()',
    vacuum: '()',
    lista: 'Vec',
    tabula: 'HashMap',
    copia: 'HashSet',
    promissum: 'Future',
    octeti: 'Vec<u8>',
};

export class RsGenerator {
    depth = 0;
    inGenerator = false;

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
                return genExpressionStatement(node, this);
            case 'ProbandumStatement':
                return genProbandumStatement(node, this);
            case 'ProbaStatement':
                return genProbaStatement(node, this);
            case 'CuraBlock':
                return genCuraBlock(node, this);
            case 'CuraStatement':
                return genCuraStatement(node, this);
            case 'AdStatement':
                throw new Error('AdStatement codegen not implemented for Rust');
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
                return `format!("${node.raw}")`;
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
            case 'ArrowFunctionExpression':
                return genArrowFunction(node, this);
            case 'LambdaExpression':
                return genLambdaExpression(node, this);
            case 'AssignmentExpression':
                return genAssignmentExpression(node, this);
            case 'CedeExpression':
                return `${this.genExpression(node.argument)}.await`;
            case 'NovumExpression':
                return genNovumExpression(node, this);
            case 'ConditionalExpression':
                return `if ${this.genExpression(node.test)} { ${this.genExpression(node.consequent)} } else { ${this.genExpression(node.alternate)} }`;
            case 'QuaExpression':
                return genQuaExpression(node, this);
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
     * WHY: Rust uses name: Type syntax like Latin.
     *      References (&) and mutability (mut) depend on usage.
     */
    genParameter(node: Parameter): string {
        const name = node.name.name;
        const type = node.typeAnnotation ? this.genType(node.typeAnnotation) : '_';

        // WHY: Strings typically passed as &str for borrowing
        if (type === 'String') {
            return `${name}: &str`;
        }

        return `${name}: ${type}`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        // Map Latin type name to Rust type
        const base = typeMap[node.name] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${base}<${params.join(', ')}>`;
            }
        }

        // Handle nullable: textus? -> Option<String>
        if (node.nullable) {
            result = `Option<${result}>`;
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

        // Numeric parameters for const generics
        if (param.type === 'Literal' && typeof param.value === 'number') {
            return String(param.value);
        }

        return null;
    }
}
