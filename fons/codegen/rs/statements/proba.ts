/**
 * Rust Code Generator - ProbaStatement
 *
 * TRANSFORMS:
 *   proba "should work" { ... }
 *   -> #[test]
 *      fn should_work() { ... }
 *
 *   omitte proba "skip this" { ... }
 *   -> #[ignore]
 *      #[test]
 *      fn skip_this() { ... }
 *
 *   futurum proba "todo" { ... }
 *   -> // TODO: todo
 */

import type { ProbaStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genProbaStatement(node: ProbaStatement, g: RsGenerator): string {
    const testName = node.name.replace(/\s+/g, '_').toLowerCase();

    if (node.modifier === 'omitte') {
        return `${g.ind()}#[ignore]\n${g.ind()}#[test]\n${g.ind()}fn ${testName}() ${genBlockStatement(node.body, g)}`;
    }

    if (node.modifier === 'futurum') {
        return `${g.ind()}// TODO: ${node.name}`;
    }

    return `${g.ind()}#[test]\n${g.ind()}fn ${testName}() ${genBlockStatement(node.body, g)}`;
}
