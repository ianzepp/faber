/**
 * Rust Code Generator - Innatum Expression (Native Type Construction)
 *
 * TRANSFORMS:
 *   {} innatum tabula<K,V> -> HashMap::new()
 *   [] innatum lista<T> -> Vec::new()
 */

import type { InnatumExpression, Expression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genInnatumExpression(node: InnatumExpression, g: RsGenerator): string {
    const typeName = node.targetType.name;

    if (typeName === 'tabula') {
        // Handle non-empty object literal: {a: 1, b: 2} innatum tabula
        if (node.expression.type === 'ObjectExpression' && node.expression.properties.length > 0) {
            const entries = node.expression.properties.map(prop => {
                if (prop.type === 'SpreadElement') {
                    return g.genExpression(prop.argument);
                }
                // WHY: key is Identifier | Literal - use name for identifiers, value for literals
                const key = prop.key.type === 'Identifier'
                    ? `"${prop.key.name}".to_string()`
                    : `"${(prop.key as any).value}".to_string()`;
                const value = g.genExpression(prop.value);
                return `(${key}, ${value})`;
            }).join(', ');
            return `HashMap::from([${entries}])`;
        }
        return 'HashMap::new()';
    }

    if (typeName === 'lista') {
        // WHY: Rust Vec requires explicit ::new() for empty, or vec![] macro
        if (node.expression.type === 'ArrayExpression' && node.expression.elements.length > 0) {
            // WHY: Filter out SpreadElement which Rust doesn't support in vec![]
            const elements = node.expression.elements
                .filter((e): e is Expression => e.type !== 'SpreadElement')
                .map(e => g.genExpression(e))
                .join(', ');
            return `vec![${elements}]`;
        }
        return 'Vec::new()';
    }

    // Fallback
    return g.genExpression(node.expression);
}
