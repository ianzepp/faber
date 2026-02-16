/**
 * TypeScript Code Generator - TemptaStatement
 *
 * TRANSFORMS:
 *   tempta { ... } cape e { ... } -> try { ... } catch (e) { ... }
 *   tempta { ... } cape e { ... } demum { ... } -> try { ... } catch (e) { ... } finally { ... }
 */

import type { TemptaStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genTemptaStatement(node: TemptaStatement, g: TsGenerator): string {
    let result = `${g.ind()}try ${genBlockStatement(node.block, g)}`;

    if (node.handler) {
        result += ` catch (${node.handler.param.name}) ${genBlockStatement(node.handler.body, g)}`;
    }

    if (node.finalizer) {
        result += ` finally ${genBlockStatement(node.finalizer, g)}`;
    }

    return result;
}
