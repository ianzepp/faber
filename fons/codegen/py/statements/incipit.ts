/**
 * Python Code Generator - IncipitStatement (entry point)
 *
 * TRANSFORMS:
 *   incipit { body } -> if __name__ == "__main__": body
 *   incipit ergo stmt -> if __name__ == "__main__": stmt
 *
 * TARGET: Python uses the if __name__ == "__main__" idiom for entry points.
 */

import type { IncipitStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genIncipitStatement(node: IncipitStatement, g: PyGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}if __name__ == "__main__":`);
    g.depth++;
    if (node.ergoStatement) {
        lines.push(g.genStatement(node.ergoStatement));
    } else {
        lines.push(g.genBlockStatementContent(node.body!));
    }
    g.depth--;
    return lines.join('\n');
}
