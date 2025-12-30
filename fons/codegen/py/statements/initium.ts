/**
 * Python Code Generator - InitiumStatement (entry point)
 *
 * TRANSFORMS:
 *   initium { body } -> if __name__ == "__main__": body
 *
 * TARGET: Python uses the if __name__ == "__main__" idiom for entry points.
 */

import type { InitiumStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genInitiumStatement(node: InitiumStatement, g: PyGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}if __name__ == "__main__":`);
    g.depth++;
    lines.push(g.genBlockStatementContent(node.body));
    g.depth--;
    return lines.join('\n');
}
