/**
 * TypeScript Code Generator - CuraBlock and CuraStatement
 *
 * CuraBlock (test hooks):
 *   cura ante { lexer = init() }
 *   -> beforeEach(() => { lexer = init(); });
 *
 *   cura ante omnia { db = connect() }
 *   -> beforeAll(() => { db = connect(); });
 *
 *   cura post { cleanup() }
 *   -> afterEach(() => { cleanup(); });
 *
 *   cura post omnia { db.close() }
 *   -> afterAll(() => { db.close(); });
 *
 * CuraStatement (resource management):
 *   cura aperi("file.txt") fit fd { lege(fd) }
 *   -> {
 *        const fd = aperi("file.txt");
 *        try {
 *          lege(fd);
 *        } finally {
 *          fd.solve?.();
 *        }
 *      }
 *
 *   cura cede connect(url) fit conn { ... }
 *   -> { const conn = await connect(url); try { ... } finally { conn.solve?.(); } }
 *
 *   cura resource() fit r { ... } cape err { ... }
 *   -> { const r = resource(); try { ... } catch (err) { ... } finally { r.solve?.(); } }
 *
 * WHY: Wraps in try/finally to guarantee cleanup via solve().
 *      Uses optional chaining (?.) so it works even if object doesn't implement curator.
 */

import type { CuraBlock, CuraStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genCuraBlock(node: CuraBlock, g: TsGenerator, semi: boolean): string {
    let hook: string;
    if (node.timing === 'ante') {
        hook = node.omnia ? 'beforeAll' : 'beforeEach';
    } else {
        hook = node.omnia ? 'afterAll' : 'afterEach';
    }
    const body = genBlockStatement(node.body, g);
    return `${g.ind()}${hook}(() => ${body})${semi ? ';' : ''}`;
}

export function genCuraStatement(node: CuraStatement, g: TsGenerator, semi: boolean): string {
    // For allocator curator kinds (arena/page), just emit the block contents
    // WHY: GC targets don't need allocator management, memory is automatic
    if (node.curatorKind === 'arena' || node.curatorKind === 'page') {
        return genBlockStatement(node.body, g);
    }

    // Generic resource management with try/finally
    const lines: string[] = [];
    const binding = node.binding.name;
    const resource = node.resource ? g.genExpression(node.resource) : 'undefined';
    const awaitPrefix = node.async ? 'await ' : '';

    // Opening block scope
    lines.push(`${g.ind()}{`);
    g.depth++;

    // Resource acquisition: const <binding> = [await] <resource>;
    lines.push(`${g.ind()}const ${binding} = ${awaitPrefix}${resource}${semi ? ';' : ''}`);

    // Try block
    lines.push(`${g.ind()}try ${genBlockStatement(node.body, g)}`);

    // Optional catch clause
    if (node.catchClause) {
        const catchParam = node.catchClause.param.name;
        const catchBody = genBlockStatement(node.catchClause.body, g);
        lines.push(`${g.ind()}catch (${catchParam}) ${catchBody}`);
    }

    // Finally block with solve?.()
    lines.push(`${g.ind()}finally {`);
    g.depth++;
    lines.push(`${g.ind()}${binding}.solve?.()${semi ? ';' : ''}`);
    g.depth--;
    lines.push(`${g.ind()}}`);

    // Close block scope
    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
