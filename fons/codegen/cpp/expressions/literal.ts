/**
 * C++23 Code Generator - Literal and TemplateLiteral
 *
 * TRANSFORMS:
 *   "hello" -> std::string("hello")
 *   42      -> 42
 *   true    -> true
 *   null    -> nullptr
 *   `hi ${x}` -> std::format("hi {}", x)
 */

import type { Literal, TemplateLiteral } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genLiteral(node: Literal, g: CppGenerator): string {
    if (node.value === null) {
        return 'nullptr';
    }

    if (typeof node.value === 'string') {
        // WHY: Use raw to preserve escape sequences like \u0048, \n, \t as-is.
        // Manual escaping would double-escape backslashes.
        return `std::string(${node.raw})`;
    }

    if (typeof node.value === 'boolean') {
        return node.value ? 'true' : 'false';
    }

    if (typeof node.value === 'number') {
        // WHY: Scientific notation should preserve format (1.5e10), but hex/binary/octal
        // are converted to decimal for C++ compatibility with older standards
        const raw = node.raw;
        // Check for scientific notation: contains 'e' or 'E' but NOT hex prefix
        const isScientific = (raw.includes('e') || raw.includes('E')) && !raw.startsWith('0x') && !raw.startsWith('0X');
        if (isScientific) {
            return raw;
        }
        // Convert to decimal for hex (0x), binary (0b), octal (0o)
        return String(node.value);
    }

    return String(node.value);
}

export function genTemplateLiteral(node: TemplateLiteral, g: CppGenerator): string {
    g.includes.add('<format>');

    // Convert template literal to std::format
    // This is a simplified version - full implementation would parse ${} expressions
    const raw = node.raw as string;

    // Replace ${...} with {}
    const format = raw.replace(/\$\{([^}]+)\}/g, '{}').replace(/`/g, '"');

    // Extract expressions
    const exprs: string[] = [];
    const regex = /\$\{([^}]+)\}/g;
    let match;

    while ((match = regex.exec(raw)) !== null) {
        if (match[1]) {
            exprs.push(match[1]);
        }
    }

    if (exprs.length === 0) {
        return `std::string(${format})`;
    }

    return `std::format(${format}, ${exprs.join(', ')})`;
}
