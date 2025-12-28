/**
 * C++23 Code Generator - FacBlockStatement
 *
 * Generates C++ try/catch blocks from Latin fac/cape (do-catch).
 *
 * TRANSFORMS:
 *   fac { x() } cape e { y() } -> try { x(); } catch (const std::exception& e) { y(); }
 *   fac { x() }                -> { x(); }  (no wrapping if no catch)
 *
 * WHY: fac is a simpler variant of tempta - just an inline block with optional catch.
 *      Without a catch clause, the block contents are emitted directly.
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genFacBlockStatement(node: FacBlockStatement, g: CppGenerator): string {
    const lines: string[] = [];

    // If there's a catch clause, wrap in try-catch
    if (node.catchClause) {
        lines.push(`${g.ind()}try ${genBlockStatement(node.body, g)}`);
        lines.push(`${g.ind()}catch (const std::exception& ${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`);
    } else {
        // No catch - just emit the block
        lines.push(`${g.ind()}${genBlockStatement(node.body, g)}`);
    }

    return lines.join('\n');
}
