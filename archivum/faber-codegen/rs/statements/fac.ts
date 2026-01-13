/**
 * Rust Code Generator - FacBlockStatement
 *
 * TRANSFORMS:
 *   fac { ... } -> { ... }
 *   fac { ... } dum cond -> loop { ...; if !(cond) { break; } }
 *
 * WHY: Rust has no do-while, so we use loop with break.
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genFacBlockStatement(node: FacBlockStatement, g: RsGenerator): string {
    if (node.test) {
        // Do-while: loop { body; if !(cond) { break; } }
        const test = g.genExpression(node.test);
        let result = `${g.ind()}loop {\n`;

        g.depth++;
        const bodyStatements = node.body.body.map(stmt => g.genStatement(stmt)).join('\n');
        if (bodyStatements) {
            result += bodyStatements + '\n';
        }
        result += `${g.ind()}if !(${test}) {\n`;
        g.depth++;
        result += `${g.ind()}break;\n`;
        g.depth--;
        result += `${g.ind()}}\n`;
        g.depth--;
        result += `${g.ind()}}`;

        return result;
    }

    // Scope block
    return `${g.ind()}${genBlockStatement(node.body, g)}`;
}
