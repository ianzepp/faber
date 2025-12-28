/**
 * TypeScript Code Generator - ProbandumStatement
 *
 * TRANSFORMS:
 *   probandum "Tokenizer" {
 *       ante { lexer = init() }
 *       proba "parses numbers" { ... }
 *   }
 *   ->
 *   describe("Tokenizer", () => {
 *       beforeEach(() => { lexer = init(); });
 *       test("parses numbers", () => { ... });
 *   });
 *
 * WHY: Maps to Bun/Jest/Vitest describe() for test organization.
 *      Setup/teardown blocks map to beforeEach/afterEach or beforeAll/afterAll.
 */

import type { ProbandumStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genProbaStatement } from './proba';
import { genCuraBlock } from './cura';
import { genBlockStatement } from './functio';

export function genProbandumStatement(node: ProbandumStatement, g: TsGenerator, semi: boolean): string {
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
            case 'CuraBlock':
                lines.push(genCuraBlock(member, g, semi));
                break;
        }
    }

    g.depth--;
    lines.push(`${g.ind()}})${semi ? ';' : ''}`);

    return lines.join('\n');
}
