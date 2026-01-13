/**
 * Rust Code Generator - FunctioDeclaration
 *
 * TRANSFORMS:
 *   functio greet(textus name) -> textus { } -> fn greet(name: &str) -> String { }
 *   futura functio fetch() -> textus { } -> async fn fetch() -> String { }
 *
 * NOTE: Untyped parameters emit `_` which rustc will reject with a helpful error.
 *       This is intentional - Rust requires explicit types, unlike TypeScript.
 */

import type { FunctioDeclaration, BlockStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { isAsyncFromAnnotations, isExternaFromAnnotations } from '../../types';

export function genFunctioDeclaration(node: FunctioDeclaration, g: RsGenerator): string {
    const isAsync = node.async || isAsyncFromAnnotations(node.annotations);
    const asyncMod = isAsync ? 'async ' : '';
    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');
    const typeParams = node.typeParams ? `<${node.typeParams.map(tp => tp.name.name).join(', ')}>` : '';

    let returnType = '';
    if (node.returnType) {
        returnType = ` -> ${g.genType(node.returnType)}`;
    }

    // External function declarations use Rust's extern syntax
    if (isExternaFromAnnotations(node.annotations)) {
        return `${g.ind()}extern "C" { fn ${name}${typeParams}(${params})${returnType}; }`;
    }

    if (!node.body) {
        throw new Error('Abstract methods not supported for Rust target');
    }

    const body = genBlockStatement(node.body, g);

    return `${g.ind()}${asyncMod}fn ${name}${typeParams}(${params})${returnType} ${body}`;
}

/**
 * Generate method declaration within an impl block.
 */
export function genMethodDeclaration(node: FunctioDeclaration, g: RsGenerator): string {
    const isAsync = node.async || isAsyncFromAnnotations(node.annotations);
    const asyncMod = isAsync ? 'async ' : '';
    const name = node.name.name;
    const params = ['&self', ...node.params.map(p => g.genParameter(p))].join(', ');

    let returnType = '';
    if (node.returnType) {
        returnType = ` -> ${g.genType(node.returnType)}`;
    }

    // External method declarations not supported in impl blocks
    if (isExternaFromAnnotations(node.annotations)) {
        throw new Error('External methods (@externa) not supported in Rust impl blocks');
    }

    if (!node.body) {
        throw new Error('Abstract methods not supported for Rust target');
    }

    const body = genBlockStatement(node.body, g);

    return `${g.ind()}${asyncMod}fn ${name}(${params})${returnType} ${body}`;
}

/**
 * Generate block statement.
 */
export function genBlockStatement(node: BlockStatement, g: RsGenerator): string {
    if (node.body.length === 0) {
        return '{}';
    }

    g.depth++;
    const body = node.body.map(stmt => g.genStatement(stmt)).join('\n');
    g.depth--;

    return `{\n${body}\n${g.ind()}}`;
}

/**
 * Generate block statement inline (for match arms).
 */
export function genBlockStatementInline(node: BlockStatement, g: RsGenerator): string {
    if (node.body.length === 0) {
        return '{}';
    }

    if (node.body.length === 1) {
        const stmt = node.body[0]!;
        if (stmt.type === 'ExpressionStatement') {
            return g.genExpression(stmt.expression);
        }
        if (stmt.type === 'ReddeStatement' && stmt.argument) {
            return g.genExpression(stmt.argument);
        }
    }

    return genBlockStatement(node, g);
}
