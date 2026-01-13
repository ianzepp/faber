/**
 * C++23 Code Generator - OrdoDeclaration
 *
 * TRANSFORMS:
 *   ordo Status { Active, Inactive, Pending }
 *   -> enum class Status { Active, Inactive, Pending };
 *
 * WHY: C++11 enum class provides type-safe enumerations.
 */

import type { OrdoDeclaration } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genOrdoDeclaration(node: OrdoDeclaration, g: CppGenerator): string {
    const name = node.name.name;
    const members = node.members.map(m => m.name.name).join(', ');

    return `${g.ind()}enum class ${name} { ${members} };`;
}
