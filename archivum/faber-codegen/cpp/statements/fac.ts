/**
 * C++23 Code Generator - FacBlockStatement
 *
 * Generates C++ try/catch blocks from Latin fac/cape (do-catch).
 *
 * TRANSFORMS:
 *   fac { x() } cape e { y() } -> try { x(); } catch (const std::exception& e) { y(); }
 *   fac { x() }                -> { x(); }  (no wrapping if no catch)
 *   fac { x() } dum cond       -> do { x(); } while (cond);
 *   fac { x() } cape e { y() } dum cond -> try { do { x(); } while (cond); } catch (...) { y(); }
 *
 * WHY: fac is a simpler variant of tempta - just an inline block with optional catch.
 *      Without a catch clause, the block contents are emitted directly.
 *      With dum, it becomes a do-while loop (C++ has native do-while).
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genFacBlockStatement(node: FacBlockStatement, g: CppGenerator): string {
    const lines: string[] = [];

    if (node.test) {
        // Do-while loop: fac { body } dum condition
        const test = g.genExpression(node.test);
        const body = genBlockStatement(node.body, g);

        if (node.catchClause) {
            // With cape: try { do { } while (); } catch (...) { }
            lines.push(`${g.ind()}try {`);
            g.depth++;
            lines.push(`${g.ind()}do ${body} while (${test});`);
            g.depth--;
            lines.push(`${g.ind()}}`);
            lines.push(`${g.ind()}catch (const std::exception& ${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`);
        }
        else {
            lines.push(`${g.ind()}do ${body} while (${test});`);
        }

        return lines.join('\n');
    }

    // If there's a catch clause, wrap in try-catch
    if (node.catchClause) {
        lines.push(`${g.ind()}try ${genBlockStatement(node.body, g)}`);
        lines.push(`${g.ind()}catch (const std::exception& ${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`);
    }
    else {
        // No catch - just emit the block
        lines.push(`${g.ind()}${genBlockStatement(node.body, g)}`);
    }

    return lines.join('\n');
}
