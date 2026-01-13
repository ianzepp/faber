/**
 * Rust Code Generator - ProbandumStatement
 *
 * TRANSFORMS:
 *   probandum "test suite" { proba "test" { ... } }
 *   -> // Test suite: test suite
 *      #[cfg(test)]
 *      mod tests {
 *          #[test]
 *          fn test() { ... }
 *      }
 */

import type { ProbandumStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genProbaStatement } from './proba';
import { genPraeparaBlock } from './cura';

export function genProbandumStatement(node: ProbandumStatement, g: RsGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}// Test suite: ${node.name}`);
    lines.push(`${g.ind()}#[cfg(test)]`);
    lines.push(`${g.ind()}mod tests {`);
    g.depth++;

    for (const member of node.body) {
        switch (member.type) {
            case 'ProbandumStatement':
                lines.push(genProbandumStatement(member, g));
                break;
            case 'ProbaStatement':
                lines.push(genProbaStatement(member, g));
                break;
            case 'PraeparaBlock':
                lines.push(genPraeparaBlock(member, g));
                break;
        }
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
