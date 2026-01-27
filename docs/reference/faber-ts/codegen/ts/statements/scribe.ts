/**
 * TypeScript Code Generator - ScribeStatement
 *
 * TRANSFORMS:
 *   scribe("hello") -> console.log("hello");
 *   scribe("debug", level: "debug") -> console.debug("debug");
 *   scribe("warn", level: "warn") -> console.warn("warn");
 */

import type { ScribeStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genScribeStatement(node: ScribeStatement, g: TsGenerator, semi: boolean): string {
    const args = node.arguments.map(arg => g.genExpression(arg)).join(', ');
    const method = node.level === 'debug' ? 'debug' : node.level === 'warn' ? 'warn' : 'log';
    return `${g.ind()}console.${method}(${args})${semi ? ';' : ''}`;
}
