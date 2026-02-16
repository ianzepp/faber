/**
 * TypeScript Code Generator - Lege Expression (read stdin)
 *
 * TRANSFORMS:
 *   lege        -> await Bun.stdin.text()
 *   lege lineam -> await Bun.stdin.readLine()
 *
 * TARGET: Bun's stdin API for reading input as text.
 *
 * WHY: Bootstrap compiler needs to read source from stdin.
 *      Uses Bun runtime which is our default TS execution environment.
 */

import type { LegeExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genLegeExpression(node: LegeExpression, _g: TsGenerator): string {
    if (node.mode === 'line') {
        // WHY: Bun doesn't have a simple readLine, use readline module via createInterface
        // For simplicity in generated code, we use a helper that reads one line
        // TODO: Consider generating a helper function for cleaner output
        return '(await (async () => { const rl = require("readline").createInterface({ input: process.stdin }); for await (const line of rl) { rl.close(); return line; } return ""; })())';
    }

    // WHY: Bun.stdin.text() returns a Promise<string> with all stdin content
    return 'await Bun.stdin.text()';
}
