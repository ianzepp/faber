/**
 * TypeScript Code Generator - ProbandumStatement
 *
 * TRANSFORMS (Legacy Mode - inProbaStandalone = false):
 *   probandum "Tokenizer" {
 *       praepara { lexer = init() }
 *       proba "parses numbers" { ... }
 *   }
 *   ->
 *   describe("Tokenizer", () => {
 *       beforeEach(() => { lexer = init(); });
 *       test("parses numbers", () => { ... });
 *   });
 *
 * TRANSFORMS (Standalone Mode - inProbaStandalone = true):
 *   probandum "Tokenizer" { ... }
 *   ->
 *   // Suite: Tokenizer
 *   [... test functions ...]
 *
 * WHY: Maps to Bun/Jest/Vitest describe() for test organization,
 *      or to suite stack for standalone test functions.
 */

import type { ProbandumStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genProbaStatement } from './proba';
import { genPraeparaBlock } from './cura';
import { genBlockStatement } from './functio';

/**
 * Sanitize name for use as identifier (replace spaces/special chars with _)
 */
function sanitizeName(name: string): string {
    return name
        .replace(/ /g, '_')
        .replace(/-/g, '_')
        .replace(/\./g, '_')
        .replace(/\//g, '_')
        .replace(/'/g, '')
        .replace(/"/g, '');
}

export function genProbandumStatement(node: ProbandumStatement, g: TsGenerator, semi: boolean): string {
    // Legacy mode: generate describe()
    if (!g.inProbaStandalone) {
        const lines: string[] = [];
        lines.push(`${g.ind()}describe("${node.name}", () => {`);

        g.depth++;

        for (const member of node.body) {
            switch (member.type) {
                case 'ProbandumStatement':
                    lines.push(genProbandumStatement(member, g, semi));
                    break;
                case 'ProbaStatement':
                    lines.push(genProbaStatement(member, g, semi));
                    break;
                case 'PraeparaBlock':
                    lines.push(genPraeparaBlock(member, g, semi));
                    break;
            }
        }

        g.depth--;
        lines.push(`${g.ind()}})${semi ? ';' : ''}`);

        return lines.join('\n');
    }

    // Standalone mode: push suite name, generate members, pop
    g.probaSuiteStack.push(sanitizeName(node.name));

    const lines: string[] = [];
    lines.push(`${g.ind()}// Suite: ${node.name}`);

    for (const member of node.body) {
        switch (member.type) {
            case 'ProbandumStatement':
                lines.push(genProbandumStatement(member, g, semi));
                break;
            case 'ProbaStatement':
                lines.push(genProbaStatement(member, g, semi));
                break;
            case 'PraeparaBlock':
                // In standalone mode, hooks are not supported
                lines.push(`${g.ind()}// TODO: setup/teardown hooks not supported in standalone mode`);
                break;
        }
    }

    g.probaSuiteStack.pop();

    return lines.join('\n');
}
