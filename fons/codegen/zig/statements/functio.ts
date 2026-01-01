/**
 * Zig Code Generator - FunctioDeclaration (function)
 *
 * TRANSFORMS:
 *   functio salve(nomen: textus): nihil -> fn salve(nomen: []const u8) void
 *   futura functio f(): numerus -> fn f() !i64
 *   functio max(prae typus T, T a, T b) -> T -> fn max(comptime T: type, a: T, b: T) T
 *
 * TARGET: Zig uses fn not function. Async becomes error union (!T).
 *         Type parameters become comptime T: type parameters.
 *
 * EDGE: anytype is not valid as a return type in Zig. If the return type
 *       resolves to anytype (from objectum/ignotum), generate a compile error.
 */

import type { FunctioDeclaration, BlockStatement, Statement, TypeParameterDeclaration } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';
import { isAsyncFromAnnotations } from '../../types';

export function genFunctioDeclaration(node: FunctioDeclaration, g: ZigGenerator): string {
    const name = node.name.name;

    // EDGE: Abstract methods have no body - Zig doesn't support abstract methods
    if (!node.body) {
        throw new Error('Abstract methods not supported for Zig target');
    }

    // Generate type parameters as comptime T: type
    const typeParams = node.typeParams?.map(tp => genTypeParameter(tp)) ?? [];

    // Generate regular parameters
    const regularParams = node.params.map(p => g.genParameter(p));

    // Combine: type params first, then regular params
    const allParams = [...typeParams, ...regularParams].join(', ');

    const returnType = node.returnType ? g.genType(node.returnType) : 'void';

    // EDGE: anytype is not valid as return type in Zig
    if (returnType === 'anytype') {
        const body = `{ @compileError("Function '${name}' returns objectum/ignotum which has no Zig equivalent - use a concrete type"); }`;
        return `${g.ind()}fn ${name}(${allParams}) void ${body}`;
    }

    // WHY: Functions containing `iace` need error union return type
    // TARGET: Async in Zig also uses error unions (!T)
    const hasIace = blockContainsIace(node.body);
    const isAsync = node.async || isAsyncFromAnnotations(node.annotations);
    const needsErrorUnion = isAsync || hasIace;
    const retType = needsErrorUnion ? `!${returnType}` : returnType;

    // WHY: Track curator parameter for collection allocator calls
    // Find curator param: type annotation name is 'curator'
    const curatorParam = node.params.find(p => p.typeAnnotation?.name.toLowerCase() === 'curator');

    // Push curator name onto stack if present
    if (curatorParam) {
        g.pushCurator(curatorParam.name.name);
    }

    const body = g.genBlockStatement(node.body);

    // Pop curator from stack after generating body
    if (curatorParam) {
        g.popCurator();
    }

    return `${g.ind()}fn ${name}(${allParams}) ${retType} ${body}`;
}

/**
 * Generate type parameter declaration.
 *
 * TRANSFORMS:
 *   prae typus T -> comptime T: type
 *
 * TARGET: Zig uses comptime T: type for compile-time type parameters.
 */
function genTypeParameter(node: TypeParameterDeclaration): string {
    return `comptime ${node.name.name}: type`;
}

/**
 * Check if a block contains non-fatal throw statements (iace).
 *
 * WHY: Functions containing `iace` need error union return types (!T).
 *      We recursively search the AST to find any `iace` usage.
 */
function blockContainsIace(node: BlockStatement): boolean {
    return node.body.some(stmt => statementContainsIace(stmt));
}

function statementContainsIace(stmt: Statement): boolean {
    switch (stmt.type) {
        case 'IaceStatement':
            return !stmt.fatal; // iace has fatal=false, mori has fatal=true

        case 'SiStatement':
            if (blockContainsIace(stmt.consequent)) {
                return true;
            }
            if (stmt.alternate) {
                if (stmt.alternate.type === 'BlockStatement') {
                    if (blockContainsIace(stmt.alternate)) {
                        return true;
                    }
                } else if (stmt.alternate.type === 'SiStatement') {
                    if (statementContainsIace(stmt.alternate)) {
                        return true;
                    }
                }
            }
            return false;

        case 'DumStatement':
        case 'IteratioStatement':
            return blockContainsIace(stmt.body);

        case 'EligeStatement':
            for (const c of stmt.cases) {
                if (blockContainsIace(c.consequent)) {
                    return true;
                }
            }
            if (stmt.defaultCase && blockContainsIace(stmt.defaultCase)) {
                return true;
            }
            return false;

        case 'DiscerneStatement':
            for (const c of stmt.cases) {
                if (blockContainsIace(c.consequent)) {
                    return true;
                }
            }
            return false;

        case 'TemptaStatement':
            if (blockContainsIace(stmt.block)) {
                return true;
            }
            if (stmt.handler && blockContainsIace(stmt.handler.body)) {
                return true;
            }
            if (stmt.finalizer && blockContainsIace(stmt.finalizer)) {
                return true;
            }
            return false;

        case 'FacBlockStatement':
            if (blockContainsIace(stmt.body)) {
                return true;
            }
            if (stmt.catchClause && blockContainsIace(stmt.catchClause.body)) {
                return true;
            }
            return false;

        case 'CustodiStatement':
            return stmt.clauses.some(c => blockContainsIace(c.consequent));

        case 'InStatement':
            return blockContainsIace(stmt.body);

        default:
            return false;
    }
}
