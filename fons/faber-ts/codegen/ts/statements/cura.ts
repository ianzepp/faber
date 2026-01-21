/**
 * TypeScript Code Generator - PraeparaBlock and CuraStatement
 *
 * PraeparaBlock (test hooks):
 *   praepara { lexer = init() }
 *   -> beforeEach(() => { lexer = init(); });
 *
 *   praepara omnia { db = connect() }
 *   -> beforeAll(() => { db = connect(); });
 *
 *   praeparabit omnia { db = cede connect() }
 *   -> beforeAll(async () => { db = await connect(); });
 *
 *   postpara { cleanup() }
 *   -> afterEach(() => { cleanup(); });
 *
 *   postpara omnia { db.close() }
 *   -> afterAll(() => { db.close(); });
 *
 *   postparabit omnia { cede db.close() }
 *   -> afterAll(async () => { await db.close(); });
 *
 * CuraStatement (resource management):
 *   cura aperi("file.txt") fixum fd { lege(fd) }
 *   -> {
 *        const fd = aperi("file.txt");
 *        try {
 *          lege(fd);
 *        } finally {
 *          fd.solve?.();
 *        }
 *      }
 *
 *   cura aperi("file.txt") varia fd { ... }
 *   -> { let fd = aperi("file.txt"); try { ... } finally { fd.solve?.(); } }
 *
 *   cura resource() fixum r { ... } cape err { ... }
 *   -> { const r = resource(); try { ... } catch (err) { ... } finally { r.solve?.(); } }
 *
 * WHY: Wraps in try/finally to guarantee cleanup via solve().
 *      Uses optional chaining (?.) so it works even if object doesn't implement curator.
 */

import type { PraeparaBlock, CuraStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genPraeparaBlock(node: PraeparaBlock, g: TsGenerator, semi: boolean): string {
    let hook: string;
    if (node.timing === 'praepara') {
        hook = node.omnia ? 'beforeAll' : 'beforeEach';
    } else {
        hook = node.omnia ? 'afterAll' : 'afterEach';
    }
    const asyncPrefix = node.async ? 'async ' : '';
    const body = genBlockStatement(node.body, g);
    return `${g.ind()}${hook}(${asyncPrefix}() => ${body})${semi ? ';' : ''}`;
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

    // Use let for mutable bindings, const for immutable
    const bindingKeyword = node.mutable ? 'let' : 'const';

    // Opening block scope
    lines.push(`${g.ind()}{`);
    g.depth++;

    // Resource acquisition: const/let <binding> = [await] <resource>;
    lines.push(`${g.ind()}${bindingKeyword} ${binding} = ${awaitPrefix}${resource}${semi ? ';' : ''}`);

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
