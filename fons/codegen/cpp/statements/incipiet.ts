/**
 * C++ Code Generator - IncipietStatement (async entry point)
 *
 * TRANSFORMS:
 *   incipiet { body } ->
 *       int main() {
 *           auto future = std::async(std::launch::async, []() {
 *               body
 *           });
 *           future.get();
 *           return 0;
 *       }
 *
 * TARGET: C++ uses std::async for async operations.
 *         Requires <future> header.
 */

import type { IncipietStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genIncipietStatement(node: IncipietStatement, g: CppGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}int main() {`);
    g.depth++;
    lines.push(`${g.ind()}auto future = std::async(std::launch::async, []() {`);
    g.depth++;
    if (node.ergoStatement) {
        lines.push(g.genStatement(node.ergoStatement));
    } else {
        lines.push(g.genBlockStatementContent(node.body!));
    }
    g.depth--;
    lines.push(`${g.ind()}});`);
    lines.push(`${g.ind()}future.get();`);
    lines.push(`${g.ind()}return 0;`);
    g.depth--;
    lines.push(`${g.ind()}}`);
    return lines.join('\n');
}
