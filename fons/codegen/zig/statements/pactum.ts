/**
 * Zig Code Generator - PactumDeclaration (interface)
 *
 * TARGET: Zig doesn't have interfaces. We emit a comment documenting the contract
 *         and rely on duck typing / comptime checks.
 *
 * TRANSFORMS:
 *   pactum iterabilis { functio sequens() -> textus? }
 *   -> // pactum iterabilis: requires fn sequens() ?[]const u8
 */

import type { PactumDeclaration } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genPactumDeclaration(node: PactumDeclaration, g: ZigGenerator): string {
    const name = node.name.name;
    const lines: string[] = [];

    lines.push(`${g.ind()}// pactum ${name}: interface contract (Zig uses duck typing)`);

    for (const method of node.methods) {
        const methodName = method.name.name;
        const params = method.params.map(p => g.genParameter(p)).join(', ');
        const returnType = method.returnType ? g.genType(method.returnType) : 'void';

        lines.push(`${g.ind()}//   requires fn ${methodName}(${params}) ${returnType}`);
    }

    return lines.join('\n');
}
