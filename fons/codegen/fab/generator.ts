/**
 * Faber Code Generator - Core generator class
 *
 * Outputs canonical Faber source code from AST.
 * Primary use case: TS-to-Faber transformation, refactoring tools.
 *
 * STYLE GUIDE (canonical output):
 * - Control flow: si/sin/secus for conditionals; casu/ceterum for elige/discerne
 * - Operators: &&/|| (not et/aut), non (not !)
 * - Functions: -> for return type (not fit/fiet/fiunt/fient)
 * - Lambdas: pro x: expr (not pro x redde expr)
 * - Indentation: 4 spaces
 * - Braces: same line
 */

import type {
    Statement,
    Expression,
    BlockStatement,
    Parameter,
    TypeAnnotation,
    TypeParameter,
    TypeParameterDeclaration,
    BaseNode,
} from '../../parser/ast';
import { COMMENT_SYNTAX, formatLeadingComments, formatTrailingComments } from '../types';

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
import { genPraeparaBlock, genCuraStatement } from './statements/cura';
import { genIncipitStatement } from './statements/incipit';
import { genIncipietStatement } from './statements/incipiet';

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
import { genQuaExpression } from './expressions/qua';
import { genEstExpression } from './expressions/est';
import { genPraefixumExpression } from './expressions/praefixum';
import { genScriptumExpression } from './expressions/scriptum';
import { genCollectionDSLExpression } from './expressions/collection-dsl';
import { genRegexLiteral } from './expressions/regex';

export class FabGenerator {
    depth = 0;
    indent: string;

    constructor(indent: string = '    ') {
        this.indent = indent;
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
        return formatLeadingComments(node, COMMENT_SYNTAX.fab, this.ind());
    }

    /**
     * Format trailing comments for a node.
     */
    trailingComments(node: BaseNode): string {
        return formatTrailingComments(node, COMMENT_SYNTAX.fab);
    }

    /**
     * Generate a statement with comments.
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
            case 'BlockStatement':
                return genBlockStatement(node, this);
            case 'ExpressionStatement':
                return `${this.ind()}${this.genExpression(node.expression)}`;
            case 'ProbandumStatement':
                return genProbandumStatement(node, this);
            case 'ProbaStatement':
                return genProbaStatement(node, this);
            case 'PraeparaBlock':
                return genPraeparaBlock(node, this);
            case 'CuraStatement':
                return genCuraStatement(node, this);
            case 'IncipitStatement':
                return genIncipitStatement(node, this);
            case 'IncipietStatement':
                return genIncipietStatement(node, this);
            case 'AdStatement':
                throw new Error('AdStatement codegen not implemented for Faber');
            default:
                throw new Error(`Unknown statement type: ${(node as any).type}`);
        }
    }

    /**
     * Generate an expression.
     */
    genExpression(node: Expression): string {
        switch (node.type) {
            case 'Identifier':
                return genIdentifier(node);
            case 'Literal':
                return genLiteral(node);
            case 'TemplateLiteral':
                return `\`${node.raw}\``;
            case 'EgoExpression':
                return 'ego';
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
                return `cede ${this.genExpression(node.argument)}`;
            case 'NovumExpression':
                return genNovumExpression(node, this);
            case 'FingeExpression':
                return genFingeExpression(node, this);
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
            case 'RegexLiteral':
                return genRegexLiteral(node, this);
            case 'LegeExpression':
                return node.mode === 'line' ? 'lege lineam' : 'lege';
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
     * Generate function parameter in Faber syntax.
     *
     * WHY: Faber uses type-first: `textus name` not `name: textus`
     *      Order: preposition, si, ceteri, type, name, ut, vel
     */
    genParameter(node: Parameter): string {
        const parts: string[] = [];

        // Preposition (de/in/ex)
        if (node.preposition) {
            parts.push(node.preposition);
        }

        // Optional modifier (si)
        if (node.optional) {
            parts.push('si');
        }

        // Rest modifier
        if (node.rest) {
            parts.push('ceteri');
        }

        // Type annotation (type-first)
        if (node.typeAnnotation) {
            parts.push(this.genType(node.typeAnnotation));
        }

        // Parameter name
        parts.push(node.name.name);

        // Alias (ut)
        if (node.alias) {
            parts.push('ut', node.alias.name);
        }

        // Default value (vel)
        if (node.defaultValue) {
            parts.push('vel', this.genExpression(node.defaultValue));
        }

        return parts.join(' ');
    }

    /**
     * Generate type parameters (generics): prae typus T, prae typus U -> <T, U>
     */
    genTypeParams(params: TypeParameterDeclaration[]): string {
        // In Faber source, type params are inline: prae typus T
        // But for declarations like genus<T>, we use angle brackets
        return `<${params.map(p => p.name.name).join(', ')}>`;
    }

    /**
     * Generate inline type parameter declarations.
     * Returns: "prae typus T, prae typus U, " (with trailing comma if non-empty)
     */
    genInlineTypeParams(params: TypeParameterDeclaration[]): string {
        if (!params || params.length === 0) {
            return '';
        }
        return params.map(p => `prae typus ${p.name.name}`).join(', ') + ', ';
    }

    /**
     * Generate type annotation.
     */
    genType(node: TypeAnnotation): string {
        // Handle union types
        if (node.union && node.union.length > 0) {
            return `unio<${node.union.map(t => this.genType(t)).join(', ')}>`;
        }

        let result = node.name;

        // Handle generic type parameters
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${result}<${params.join(', ')}>`;
            }
        }

        // Handle nullable
        if (node.nullable) {
            result = `${result}?`;
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

        if (param.type === 'Literal') {
            return String(param.value);
        }

        return null;
    }
}
