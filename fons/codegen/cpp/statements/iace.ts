/**
 * C++23 Code Generator - IaceStatement
 *
 * TRANSFORMS:
 *   iace "error" -> throw std::runtime_error("error");
 *   mori "fatal" -> std::print(stderr, "FATAL: {}\n", "fatal"); std::abort();
 *
 * WHY: mori (fatal=true) is unrecoverable, iace (fatal=false) is catchable
 */

import type { IaceStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genIaceStatement(node: IaceStatement, g: CppGenerator): string {
    // WHY: mori (fatal=true) is unrecoverable, iace (fatal=false) is catchable
    if (node.fatal) {
        g.includes.add('<cstdlib>');
        g.includes.add('<print>');

        // mori -> print message and abort
        if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
            return `${g.ind()}std::print(stderr, "FATAL: {}\\n", "${node.argument.value}"); std::abort();`;
        }

        const msg = g.genExpression(node.argument);

        return `${g.ind()}std::print(stderr, "FATAL: {}\\n", ${msg}); std::abort();`;
    }

    g.includes.add('<stdexcept>');

    // Handle string literals -> runtime_error
    if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
        return `${g.ind()}throw std::runtime_error("${node.argument.value}");`;
    }

    // Handle new Error("msg")
    if (node.argument.type === 'NovumExpression' && node.argument.callee.name === 'Error') {
        const firstArg = node.argument.arguments[0];
        const msg = firstArg && firstArg.type !== 'SpreadElement' ? g.genExpression(firstArg) : '""';

        return `${g.ind()}throw std::runtime_error(${msg});`;
    }

    return `${g.ind()}throw ${g.genExpression(node.argument)};`;
}
