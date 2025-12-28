/**
 * Python Code Generator - TemptaStatement
 *
 * Generates Python try/except/finally statements from Latin tempta/cape/fini blocks.
 *
 * TRANSFORMS:
 *   tempta { x() } cape e { y() }          -> try: x() except Exception as e: y()
 *   tempta { x() } fini { cleanup() }      -> try: x() finally: cleanup()
 *   tempta { x() } cape e { y() } fini { } -> try: x() except Exception as e: y() finally: ...
 */

import type { TemptaStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genTemptaStatement(node: TemptaStatement, g: PyGenerator): string {
    const lines: string[] = [];

    lines.push(`${g.ind()}try:`);
    g.depth++;
    lines.push(g.genBlockStatementContent(node.block));
    g.depth--;

    if (node.handler) {
        lines.push(`${g.ind()}except Exception as ${node.handler.param.name}:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.handler.body));
        g.depth--;
    }

    if (node.finalizer) {
        lines.push(`${g.ind()}finally:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.finalizer));
        g.depth--;
    }

    return lines.join('\n');
}
