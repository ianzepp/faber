/**
 * TypeScript Code Generator - ProbaStatement
 *
 * TRANSFORMS:
 *   proba "parses integers" { adfirma parse("42") est 42 }
 *   -> test("parses integers", () => { ... });
 *
 *   proba omitte "blocked by #42" { ... }
 *   -> test.skip("blocked by #42", () => { ... });
 *
 *   proba futurum "needs feature" { ... }
 *   -> test.todo("needs feature");
 *
 * WHY: Maps to test()/test.skip()/test.todo() for test runners.
 */

import type { ProbaStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genProbaStatement(node: ProbaStatement, g: TsGenerator, semi: boolean): string {
    if (node.modifier === 'omitte') {
        // Skip: test.skip("reason: name", () => { ... })
        const reason = node.modifierReason ? `${node.modifierReason}: ` : '';
        const body = genBlockStatement(node.body, g);
        return `${g.ind()}test.skip("${reason}${node.name}", () => ${body})${semi ? ';' : ''}`;
    }

    if (node.modifier === 'futurum') {
        // Todo: test.todo("reason: name")
        const reason = node.modifierReason ? `${node.modifierReason}: ` : '';
        return `${g.ind()}test.todo("${reason}${node.name}")${semi ? ';' : ''}`;
    }

    // Regular test
    const body = genBlockStatement(node.body, g);
    return `${g.ind()}test("${node.name}", () => ${body})${semi ? ';' : ''}`;
}
