/**
 * Zig Code Generator - Core generator class
 *
 * Holds shared state and utilities for Zig code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter } from '../../parser/ast';
import type { SemanticType } from '../../semantic/types';

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
import { genCuraStatement } from './statements/cura';

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
import { genEstExpression } from './expressions/est';
import { genPraefixumExpression } from './expressions/praefixum';
import { genScriptumExpression } from './expressions/scriptum';

/**
 * Map Latin type names to Zig types.
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
    // Memory management
    curator: 'std.mem.Allocator',
};

export class ZigGenerator {
    depth = 0;
    inGenerator = false;

    // Track module-level constants (comptime values)
    moduleConstants = new Set<string>();

    // Track active allocator name for collection operations
    curatorStack: string[] = ['alloc'];

    constructor(public indent: string = '    ') {}

    // -------------------------------------------------------------------------
    // Allocator (curator) management
    // -------------------------------------------------------------------------

    /**
     * Get the current active allocator name.
     */
    getCurator(): string {
        return this.curatorStack[this.curatorStack.length - 1] ?? 'alloc';
    }

    /**
     * Push a new allocator name onto the stack.
     */
    pushCurator(name: string): void {
        this.curatorStack.push(name);
    }

    /**
     * Pop the current allocator from the stack.
     */
    popCurator(): void {
        if (this.curatorStack.length > 1) {
            this.curatorStack.pop();
        }
    }

    // -------------------------------------------------------------------------
    // Module constants tracking
    // -------------------------------------------------------------------------

    /**
     * Check if a name is a module-level constant.
     */
    hasModuleConstant(name: string): boolean {
        return this.moduleConstants.has(name);
    }

    /**
     * Add a module-level constant.
     */
    addModuleConstant(name: string): void {
        this.moduleConstants.add(name);
    }

    // -------------------------------------------------------------------------
    // Type inference helpers
    // -------------------------------------------------------------------------

    /**
     * Check if an expression has a string type.
     */
    isStringType(node: Expression): boolean {
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
     * Infer Zig type from expression.
     */
    inferZigType(node: Expression): string {
        // Use resolved type from semantic analysis if available
        if (node.resolvedType) {
            return this.semanticTypeToZig(node.resolvedType);
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
    semanticTypeToZig(type: SemanticType): string {
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
                return genOrdoDeclaration(node as any, this);
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
            case 'CuraStatement':
                return genCuraStatement(node, this);
            case 'BlockStatement':
                return this.genBlockStatementContent(node);
            case 'ExpressionStatement':
                return this.genExpressionStatement(node);
            case 'AdStatement':
                throw new Error('AdStatement codegen not implemented for Zig');
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

            // Handle string concatenation with + operator
            if (node.operator === '+' && (this.isStringType(node.left) || this.isStringType(node.right))) {
                return `${left} ++ ${right}`;
            }

            // Handle string comparison
            if ((node.operator === '==' || node.operator === '===') && (this.isStringType(node.left) || this.isStringType(node.right))) {
                return `std.mem.eql(u8, ${left}, ${right})`;
            }
            if ((node.operator === '!=' || node.operator === '!==') && (this.isStringType(node.left) || this.isStringType(node.right))) {
                return `!std.mem.eql(u8, ${left}, ${right})`;
            }

            // Handle operator mapping for Zig
            let op = node.operator;
            if (op === '&&') op = 'and';
            else if (op === '||') op = 'or';
            else if (op === '??') op = 'orelse';
            else if (op === '===') op = '==';
            else if (op === '!==') op = '!=';

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
                // Zig doesn't have template literals, convert to string
                return `"${node.raw.replace(/`/g, '')}"`;
            case 'EgoExpression':
                // TARGET: ego (this) becomes self in Zig struct methods
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
                // Zig async is different, use try for error handling
                return `try ${this.genExpression(node.argument)}`;
            case 'NovumExpression':
                return genNovumExpression(node, this);
            case 'ConditionalExpression':
                return `if (${this.genExpression(node.test)}) ${this.genExpression(node.consequent)} else ${this.genExpression(node.alternate)}`;
            case 'QuaExpression':
                return this.genQuaExpression(node);
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
     * Generate type cast expression.
     *
     * TRANSFORMS:
     *   x qua numerus -> @as(i64, x)
     *   data qua textus -> @as([]const u8, data)
     *   [1, 2, 3] qua numerus[] -> [_]i64{ 1, 2, 3 }
     *
     * TARGET: Zig uses @as(T, x) builtin for type coercion.
     *
     * EDGE: Array literals need special handling - Zig can't @as to a slice,
     *       must use [_]T{} syntax for proper array type.
     */
    genQuaExpression(node: import('../../parser/ast').QuaExpression): string {
        // EDGE: Array literal with type cast needs [_]T{} syntax, not @as
        // WHY: arrayShorthand means it was parsed as T[] syntax, name will be 'lista'
        const isArrayType = node.targetType.arrayShorthand || node.targetType.name === 'lista';

        if (node.expression.type === 'ArrayExpression' && isArrayType) {
            const elementTypeNode = node.targetType.typeParameters?.[0];
            if (elementTypeNode && elementTypeNode.type === 'TypeAnnotation') {
                const elementType = this.genType(elementTypeNode);
                const elements = node.expression.elements
                    .map(el => {
                        if (el.type === 'SpreadElement') {
                            return this.genExpression(el.argument);
                        }
                        return this.genExpression(el);
                    })
                    .join(', ');
                return `[_]${elementType}{ ${elements} }`;
            }
        }

        const expr = this.genExpression(node.expression);
        const targetType = this.genType(node.targetType);
        return `@as(${targetType}, ${expr})`;
    }

    /**
     * Generate expression statement.
     *
     * TARGET: Zig requires expressions to be used or discarded with _ = prefix.
     *         Calls and assignments don't need discard prefix.
     */
    genExpressionStatement(node: import('../../parser/ast').ExpressionStatement): string {
        const expr = this.genExpression(node.expression);

        // TARGET: Zig assignments and calls are statements, not expressions
        if (node.expression.type === 'CallExpression' || node.expression.type === 'AssignmentExpression') {
            return `${this.ind()}${expr};`;
        }

        // WHY: Zig requires explicit discard with _ = for unused expression results
        return `${this.ind()}_ = ${expr};`;
    }

    /**
     * Generate block statement with braces.
     */
    genBlockStatement(node: BlockStatement): string {
        if (node.body.length === 0) {
            return '{}';
        }
        this.depth++;
        const body = this.genBlockStatementContent(node);
        this.depth--;
        return `{\n${body}\n${this.ind()}}`;
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
     * WHY: Zig requires type annotations on all parameters.
     *      Rest parameters use ... prefix syntax.
     */
    genParameter(node: Parameter): string {
        const name = node.name.name;
        const preposition = node.preposition;
        const type = node.typeAnnotation ? this.genTypeWithPreposition(node.typeAnnotation, preposition) : 'anytype';
        return `${name}: ${type}`;
    }

    /**
     * Generate type with ownership preposition applied.
     *
     * TRANSFORMS:
     *   (none) textus -> []const u8 (owned, uses module allocator if needed)
     *   de textus -> []const u8 (borrowed slice, read-only)
     *   in textus -> *[]u8 (mutable pointer to slice)
     */
    genTypeWithPreposition(node: TypeAnnotation, preposition?: string): string {
        const baseType = this.genType(node);
        const typeName = node.name;

        if (!preposition) {
            return baseType;
        }

        if (preposition === 'de') {
            return this.genBorrowedType(typeName, baseType, node);
        }

        if (preposition === 'in') {
            return this.genMutableType(typeName, baseType, node);
        }

        return baseType;
    }

    /**
     * Generate borrowed (read-only) type for 'de' preposition.
     */
    genBorrowedType(typeName: string, baseType: string, node: TypeAnnotation): string {
        if (typeName === 'textus') {
            return baseType;
        }

        if (typeName === 'lista' && node.typeParameters && node.typeParameters.length > 0) {
            const elemType = node.typeParameters[0]!;
            if (elemType.type === 'TypeAnnotation') {
                const innerType = this.genType(elemType);
                return `[]const ${innerType}`;
            }
        }

        if (typeName === 'tabula' || typeName === 'copia') {
            return `*const ${baseType}`;
        }

        if (typeMap[typeName]) {
            return `*const ${baseType}`;
        }

        return `*const ${baseType}`;
    }

    /**
     * Generate mutable type for 'in' preposition.
     */
    genMutableType(typeName: string, baseType: string, node: TypeAnnotation): string {
        if (typeName === 'textus') {
            return `*[]u8`;
        }

        if (typeName === 'lista' && node.typeParameters && node.typeParameters.length > 0) {
            const elemType = node.typeParameters[0]!;
            if (elemType.type === 'TypeAnnotation') {
                const innerType = this.genType(elemType);
                return `*std.ArrayList(${innerType})`;
            }
        }

        return `*${baseType}`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        const name = node.name;

        // Handle union types - Zig doesn't have untagged unions
        if (node.union && node.union.length > 0) {
            return 'anytype';
        }

        // Handle generic types
        if (node.typeParameters && node.typeParameters.length > 0) {
            return this.genGenericType(name, node.typeParameters, node.nullable);
        }

        // Type lookup
        const base = typeMap[name] ?? node.name;

        if (node.nullable) {
            return `?${base}`;
        }

        return base;
    }

    /**
     * Generate generic type (lista<T>, tabula<K,V>, etc).
     */
    genGenericType(name: string, params: TypeParameter[], nullable?: boolean): string {
        let result: string;

        switch (name) {
            case 'lista': {
                const elemType = params[0];
                const innerType = elemType && 'name' in elemType ? this.genType(elemType as TypeAnnotation) : 'anytype';
                result = `[]${innerType}`;
                break;
            }
            case 'tabula': {
                const keyType = params[0];
                const valType = params[1];
                const keyZig = keyType && 'name' in keyType ? this.genType(keyType as TypeAnnotation) : 'anytype';
                const valZig = valType && 'name' in valType ? this.genType(valType as TypeAnnotation) : 'anytype';

                if (keyZig === '[]const u8') {
                    result = `std.StringHashMap(${valZig})`;
                } else {
                    result = `std.AutoHashMap(${keyZig}, ${valZig})`;
                }
                break;
            }
            case 'copia': {
                const elemType = params[0];
                const innerType = elemType && 'name' in elemType ? this.genType(elemType as TypeAnnotation) : 'anytype';

                if (innerType === '[]const u8') {
                    result = 'std.StringHashMap(void)';
                } else {
                    result = `std.AutoHashMap(${innerType}, void)`;
                }
                break;
            }
            case 'promissum': {
                const innerType = params[0];
                const zigType = innerType && 'name' in innerType ? this.genType(innerType as TypeAnnotation) : 'anytype';
                result = `!${zigType}`;
                break;
            }
            default:
                result = name;
        }

        if (nullable) {
            return `?${result}`;
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

        // Numeric parameters become comptime values
        if (param.type === 'Literal' && typeof param.value === 'number') {
            return String(param.value);
        }

        return null;
    }
}
