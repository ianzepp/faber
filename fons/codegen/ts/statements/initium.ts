/**
 * TypeScript Code Generator - InitiumStatement (entry point)
 *
 * TRANSFORMS:
 *   initium { body } -> body (top-level statements)
 *
 * TARGET: TypeScript/JavaScript executes top-level code directly.
 *         No wrapper function needed - just emit the body statements.
 */

import type { InitiumStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genInitiumStatement(node: InitiumStatement, g: TsGenerator): string {
    // Just emit the body statements - no wrapper needed for TS
    return g.genBlockStatementContent(node.body);
}
