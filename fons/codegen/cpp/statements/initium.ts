/**
 * C++ Code Generator - InitiumStatement (entry point)
 *
 * TRANSFORMS:
 *   initium { body } -> int main() { body; return 0; }
 *
 * TARGET: C++ uses int main() as the program entry point.
 */

import type { InitiumStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genInitiumStatement(node: InitiumStatement, g: CppGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}int main() {`);
    g.depth++;
    lines.push(g.genBlockStatementContent(node.body));
    lines.push(`${g.ind()}return 0;`);
    g.depth--;
    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}
