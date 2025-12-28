/**
 * C++23 Code Generator - TemptaStatement
 *
 * TRANSFORMS:
 *   tempta { ... } cape err { ... } -> try { ... } catch (const std::exception& err) { ... }
 *   tempta { ... } demum { ... } -> auto _demum_0 = _ScopeGuard([&]{ ... }); try { ... }
 *
 * WHY: C++ doesn't have finally. We use a scope guard pattern:
 *      The finally block runs via RAII destructor on scope exit.
 */

import type { TemptaStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genTemptaStatement(node: TemptaStatement, g: CppGenerator): string {
    const lines: string[] = [];

    // WHY: C++ doesn't have finally. We use a scope guard pattern:
    // The finally block runs via RAII destructor on scope exit.
    if (node.finalizer) {
        g.includes.add('<utility>');
        g.needsScopeGuard = true;

        // Generate a unique ID for this scope guard
        const guardId = `_demum_${g.scopeGuardCounter++}`;
        const finalizerCode = node.finalizer.body.map(stmt => g.genStatement(stmt).trim()).join(' ');

        lines.push(`${g.ind()}auto ${guardId} = _ScopeGuard([&]{ ${finalizerCode} });`);
    }

    lines.push(`${g.ind()}try ${genBlockStatement(node.block, g)}`);

    if (node.handler) {
        const param = node.handler.param.name;

        lines.push(`${g.ind()}catch (const std::exception& ${param}) ${genBlockStatement(node.handler.body, g)}`);
    }

    return lines.join('\n');
}
