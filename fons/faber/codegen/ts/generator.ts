/**
 * TypeScript Code Generator - Core generator class
 *
 * Holds shared state and utilities for TypeScript code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
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
import type { RequiredFeatures } from '../types';
import { createRequiredFeatures, COMMENT_SYNTAX, formatLeadingComments, formatTrailingComments } from '../types';
import type { Position } from '../../tokenizer/types';

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
import { genInnatumExpression } from './expressions/innatum';
import { genConversionExpression } from './expressions/conversio';
import { genShiftExpression } from './expressions/shift';
import { genEstExpression } from './expressions/est';
import { genPraefixumExpression } from './expressions/praefixum';
import { genScriptumExpression } from './expressions/scriptum';
import { genCollectionDSLExpression } from './expressions/collection-dsl';
import { genAbExpression } from './expressions/ab';
import { genRegexLiteral } from './expressions/regex';
import { genLegeExpression } from './expressions/lege';

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

/** CLI command parameter metadata */
export interface CliParam {
    name: string;
    type: string;
    optional: boolean;
    shortFlag?: string;
    longFlag?: string;
    defaultValue?: string;
    description?: string;
}

/** CLI option from @ optio annotation */
export interface CliOption {
    type: string;           // bivalens, textus, numerus, etc.
    external: string;       // CLI flag name (may have hyphens)
    internal: string;       // Internal binding name
    short?: string;         // Short flag (single char)
    defaultValue?: string;  // Default value
    description?: string;   // Help text
}

/** CLI operand from @ operandus annotation */
export interface CliOperand {
    type: string;           // textus, numerus, etc.
    name: string;           // Binding name
    rest: boolean;          // Is this a variadic/rest operand (ceteri)
    defaultValue?: string;  // Default value (makes operand optional)
    description?: string;   // Help text
}

/** Single-command CLI info (no subcommands) */
export interface CliSingleCommand {
    options: CliOption[];
    operands: CliOperand[];
}

/** CLI command tree node - can be a leaf (with handler) or branch (with children) */
export interface CliCommandNode {
    name: string;           // This node's name (e.g., "add" in "remote/add")
    fullPath: string;       // Full path for help text (e.g., "remote/add")
    alias?: string;
    description?: string;   // For mounted submodules (from @ descriptio)
    // Leaf node properties (has a handler function)
    functionName?: string;
    modulePrefix?: string;  // For imported commands (e.g., "configModule" for configModule.list)
    params?: CliParam[];
    // Branch node properties
    children: Map<string, CliCommandNode>;
}

/** CLI program metadata collected from @ cli annotations on incipit */
export interface CliProgram {
    name: string;
    version?: string;
    description?: string;
    root: CliCommandNode;           // Tree root containing all commands (subcommand mode)
    singleCommand?: CliSingleCommand;  // Single-command mode info (no subcommands)
}

export class TsGenerator {
    depth = 0;
    inGenerator = false;
    inFlumina = false; // WHY: Track if we're inside a fit function body for Responsum protocol
    inFiunt = false; // WHY: Track if we're inside a fiunt function body for flow() protocol
    inFiet = false; // WHY: Track if we're inside a fiet function body for async Responsum protocol
    inFient = false; // WHY: Track if we're inside a fient function body for async flow() protocol
    inClass = false; // WHY: Track if we're inside a class body (for visibility handling)
    genusNames = new Set<string>(); // WHY: Track genus type names for proper instantiation
    features: RequiredFeatures;
    semi: boolean;
    codegenErrors: Array<{ message: string; position?: Position }> = [];
    sourceFilePath?: string; // WHY: Needed to resolve @subsidia paths to absolute
    keepRelativeImports = false; // WHY: For build command, keep relative imports

    // CLI codegen state
    cli?: CliProgram;
    /** Module imports needed for CLI dispatcher (alias -> relative path) */
    cliModuleImports: Map<string, string> = new Map();

    /** HAL imports needed for @subsidia pactums (name -> import path) */
    halImports: Map<string, string> = new Map();

    /** Subsidia mappings from semantic analysis (import source -> target -> impl path) */
    subsidiaImports: Map<string, Map<string, string>> = new Map();

    constructor(
        public indent: string = '  ',
        semi: boolean = true,
    ) {
        this.features = createRequiredFeatures();
        this.semi = semi;
    }

    /**
     * Record a codegen error without throwing immediately.
     *
     * WHY: Prefer collecting multiple errors in one pass so users can fix
     *      issues in batches (especially missing type info for norma lowering).
     */
    codegenError(message: string, node?: BaseNode): void {
        this.codegenErrors.push({
            message,
            position: node?.position,
        });
    }

    /**
     * Check if a type name is a genus (class with methods).
     *
     * WHY: Object literals cast to genus types need `new` instantiation,
     *      not just type assertion, to get the class methods.
     */
    isGenus(typeName: string): boolean {
        return this.genusNames.has(typeName);
    }

    /**
     * Generate indentation for current depth.
     */
    ind(): string {
        return this.indent.repeat(this.depth);
    }

    /**
     * Format leading comments for a node.
     *
     * WHY: Comments attached to AST nodes are emitted before the node content.
     */
    leadingComments(node: BaseNode): string {
        return formatLeadingComments(node, this.ind());
    }

    /**
     * Format trailing comments for a node.
     *
     * WHY: Trailing comments appear on the same line after the node content.
     */
    trailingComments(node: BaseNode): string {
        return formatTrailingComments(node);
    }

    /**
     * Generate a statement. Dispatches to specific gen* functions.
     *
     * WHY: Wraps statement generation with comment emission. Leading comments
     *      appear before the statement, trailing comments on the same line.
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
            case 'PraeparaBlock':
                return genPraeparaBlock(node, this, this.semi);
            case 'CuraStatement':
                return genCuraStatement(node, this, this.semi);
            case 'IncipitStatement':
                return genIncipitStatement(node, this);
            case 'IncipietStatement':
                return genIncipietStatement(node, this);
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
            case 'LambdaExpression':
                return genLambdaExpression(node, this);
            case 'AssignmentExpression':
                return genAssignmentExpression(node, this);
            case 'CedeExpression':
                // WHY: In fiunt context, cede yields respond.item() for Responsum protocol
                if (this.inFiunt) {
                    return `yield respond.item(${this.genExpression(node.argument)})`;
                }
                // WHY: In fient context, cede yields respond.item() but awaits the value first
                if (this.inFient) {
                    return `yield respond.item(await ${this.genExpression(node.argument)})`;
                }
                // WHY: cede maps to yield in generators, await in async functions
                return `${this.inGenerator ? 'yield' : 'await'} ${this.genExpression(node.argument)}`;
            case 'NovumExpression':
                return genNovumExpression(node, this);
            case 'FingeExpression':
                return genFingeExpression(node, this);
            case 'ConditionalExpression':
                return `${this.genExpression(node.test)} ? ${this.genExpression(node.consequent)} : ${this.genExpression(node.alternate)}`;
            case 'QuaExpression':
                return genQuaExpression(node, this);
            case 'InnatumExpression':
                return genInnatumExpression(node, this);
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
            case 'CollectionDSLExpression':
                return genCollectionDSLExpression(node, this);
            case 'AbExpression':
                return genAbExpression(node, this);
            case 'RegexLiteral':
                return genRegexLiteral(node, this);
            case 'LegeExpression':
                return genLegeExpression(node, this);
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
     * WHY: Dual naming (textus location ut loc) uses internal name (alias) in generated code.
     *      The external name is for callsite documentation only.
     *      Default values (vel) generate = default syntax.
     *      Optional params (si) without default generate name?: type syntax.
     */
    genParameter(node: Parameter): string {
        // Use alias (internal name) if present, otherwise external name
        const name = node.alias?.name ?? node.name.name;
        const prefix = node.rest ? '...' : '';
        const defaultVal = node.defaultValue ? ` = ${this.genExpression(node.defaultValue)}` : '';

        // Optional without default: name?: type
        // Optional with default: name: type = value (default makes it optional)
        const optionalMark = node.optional && !node.defaultValue ? '?' : '';
        const typeAnno = node.typeAnnotation ? `: ${this.genType(node.typeAnnotation)}` : '';

        return `${prefix}${name}${optionalMark}${typeAnno}${defaultVal}`;
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
        // Handle function types: (T, U) -> V => (arg0: T, arg1: U) => V
        if (node.parameterTypes && node.returnType) {
            const params = node.parameterTypes.map((p, i) => `arg${i}: ${this.genType(p)}`).join(', ');
            const ret = this.genType(node.returnType);
            return `(${params}) => ${ret}`;
        }

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
