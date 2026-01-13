/**
 * C++ Code Generator - IncipitStatement (entry point)
 *
 * TRANSFORMS:
 *   incipit { body } -> int main() { body; return 0; }
 *   incipit ergo stmt -> int main() { stmt; return 0; }
 *
 * TARGET: C++ uses int main() as the program entry point.
 */

import type { IncipitStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genIncipitStatement(node: IncipitStatement, g: CppGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}int main() {`);
    g.depth++;
    if (node.ergoStatement) {
        lines.push(g.genStatement(node.ergoStatement));
    } else {
        lines.push(g.genBlockStatementContent(node.body!));
    }
    lines.push(`${g.ind()}return 0;`);
    g.depth--;
    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}
