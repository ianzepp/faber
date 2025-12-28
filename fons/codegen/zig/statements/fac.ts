/**
 * Zig Code Generator - FacBlockStatement (inline try/catch)
 *
 * TRANSFORMS:
 *   fac { riskyOperation() } -> { riskyOperation(); }
 *   fac { op() } cape err { handle(err) } -> // fac block with error capture
 *
 * TARGET: Zig uses error unions and catch syntax for error handling,
 *         not try-catch blocks. fac without cape is just a scope block.
 *         fac with cape requires different patterns (catch on expressions).
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genFacBlockStatement(node: FacBlockStatement, g: ZigGenerator): string {
    if (node.catchClause) {
        // With cape, emit as commented block since Zig catch works differently
        // Real implementation would need to use catch on error-returning expressions
        let result = `${g.ind()}// fac block with catch - Zig uses 'catch' on error union expressions\n`;
        result += g.genBlockStatementContent(node.body);

        result += `\n${g.ind()}// catch clause for: ${node.catchClause.param.name}`;
        // The catch body is not directly usable without error union context

        return result;
    }

    // Without cape, just emit the block
    return g.genBlockStatementContent(node.body);
}
