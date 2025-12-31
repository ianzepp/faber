/**
 * Python Code Generator - IncipietStatement (async entry point)
 *
 * TRANSFORMS:
 *   incipiet { body } ->
 *       async def main():
 *           body
 *       if __name__ == "__main__":
 *           asyncio.run(main())
 *
 * TARGET: Python uses asyncio.run() for async entry points.
 */

import type { IncipietStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genIncipietStatement(node: IncipietStatement, g: PyGenerator): string {
    const lines: string[] = [];

    // Define async main function
    lines.push(`${g.ind()}async def main():`);
    g.depth++;
    if (node.ergoStatement) {
        lines.push(g.genStatement(node.ergoStatement));
    } else {
        lines.push(g.genBlockStatementContent(node.body!));
    }
    g.depth--;

    // Entry point guard with asyncio.run
    lines.push('');
    lines.push(`${g.ind()}if __name__ == "__main__":`);
    g.depth++;
    lines.push(`${g.ind()}asyncio.run(main())`);
    g.depth--;

    return lines.join('\n');
}
