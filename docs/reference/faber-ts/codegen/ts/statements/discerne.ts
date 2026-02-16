/**
 * TypeScript Code Generator - DiscerneStatement
 *
 * TRANSFORMS:
 *   # Single discriminant
 *   discerne event { casu Click ut c { use(c.x) } }
 *   -> if (event.tag === 'Click') { const c = event; use(c.x); }
 *
 *   discerne event { casu Click pro a, b { use(a, b) } casu Quit { exit() } }
 *   -> if (event.tag === 'Click') { const a = event.x; const b = event.y; use(a, b); }
 *      else if (event.tag === 'Quit') { exit(); }
 *
 *   # Multi-discriminant
 *   discerne left, right {
 *       casu Primitivum ut l, Primitivum ut r { redde l.nomen == r.nomen }
 *       casu _, _ { redde falsum }
 *   }
 *   -> if (left.tag === 'Primitivum' && right.tag === 'Primitivum') {
 *          const l = left; const r = right;
 *          return l.nomen === r.nomen;
 *      }
 *      return false;
 *
 * WHY: TypeScript discriminated unions use a 'tag' property for discrimination.
 *      Multi-discriminant matching generates combined conditions to avoid nesting.
 */

import type { DiscerneStatement, Identifier, VariantPattern } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import type { DiscretioType, SemanticType } from '../../../semantic/types';

export function genDiscerneStatement(node: DiscerneStatement, g: TsGenerator): string {
    // Generate expressions for all discriminants
    const discriminants = node.discriminants.map((d) => g.genExpression(d));

    // Get discretio type info for each discriminant
    const discretios: (DiscretioType | null)[] = node.discriminants.map((d) => {
        const t = d.resolvedType;
        return t?.kind === 'discretio' ? (t as DiscretioType) : null;
    });

    let result = '';

    // Generate if/else chain for each case
    for (let i = 0; i < node.cases.length; i++) {
        const caseNode = node.cases[i]!;
        const keyword = i === 0 ? 'if' : 'else if';

        // Build combined condition for all patterns
        const conditions: string[] = [];
        for (let j = 0; j < caseNode.patterns.length; j++) {
            const pattern = caseNode.patterns[j]!;
            const discriminant = discriminants[j]!;

            // Wildcard matches anything - no condition needed
            if (!pattern.isWildcard) {
                conditions.push(`${discriminant}.tag === '${pattern.variant.name}'`);
            }
        }

        // If all patterns are wildcards, this is a catch-all
        if (conditions.length === 0) {
            // Generate as else block if not first case, otherwise unconditional
            if (i === 0) {
                result += `${g.ind()}{\n`;
            } else {
                result += `${g.ind()}else {\n`;
            }
        } else {
            const condition = conditions.join(' && ');
            result += `${g.ind()}${keyword} (${condition}) {\n`;
        }

        g.depth++;

        // Generate bindings for each pattern
        for (let j = 0; j < caseNode.patterns.length; j++) {
            const pattern = caseNode.patterns[j]!;
            const discriminant = discriminants[j]!;
            const discretio = discretios[j] ?? null;

            // Skip wildcards - no bindings
            if (pattern.isWildcard) continue;

            result += genPatternBindings(pattern, discriminant, discretio, g);
        }

        // Generate body statements
        for (const stmt of caseNode.consequent.body) {
            result += g.genStatement(stmt) + '\n';
        }

        g.depth--;
        result += `${g.ind()}}`;

        // Add newline if more cases or default follows
        if (i < node.cases.length - 1 || node.defaultCase) {
            result += '\n';
        }
    }

    // Generate default case (ceterum)
    if (node.defaultCase) {
        if (node.cases.length > 0) {
            result += `${g.ind()}else {\n`;
        } else {
            // No cases, just default - emit as bare block
            result += `${g.ind()}{\n`;
        }

        g.depth++;

        for (const stmt of node.defaultCase.body) {
            result += g.genStatement(stmt) + '\n';
        }

        g.depth--;
        result += `${g.ind()}}`;
    }

    return result;
}

/**
 * Generate bindings for a single pattern.
 */
function genPatternBindings(
    pattern: VariantPattern,
    discriminant: string,
    discretio: DiscretioType | null,
    g: TsGenerator
): string {
    let result = '';
    const variantName = pattern.variant.name;
    const variantInfo = discretio?.variants.get(variantName);

    // Alias binding: casu Click ut c
    if (pattern.alias) {
        result += `${g.ind()}const ${pattern.alias.name} = ${discriminant};\n`;
    }
    // Positional bindings: casu Click pro a, b
    else if (pattern.bindings.length > 0) {
        if (variantInfo) {
            // With type info: map positional bindings to actual field names
            for (let j = 0; j < pattern.bindings.length; j++) {
                const binding = pattern.bindings[j]!;
                const fieldName = variantInfo.fields[j]?.name;

                if (fieldName) {
                    result += `${g.ind()}const ${binding.name} = ${discriminant}.${fieldName};\n`;
                }
            }
        } else {
            // Fallback without type info: destructure with binding names
            const bindingNames = pattern.bindings.map((b: Identifier) => b.name).join(', ');
            result += `${g.ind()}const { ${bindingNames} } = ${discriminant};\n`;
        }
    }

    return result;
}
