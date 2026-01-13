/**
 * Python Code Generator - GenusDeclaration
 *
 * TRANSFORMS:
 *   genus persona { textus nomen: "X" numerus aetas: 0 }
 *   ->
 *   class persona:
 *       nomen: str = "X"
 *       aetas: int = 0
 *
 *       def __init__(self, overrides: dict = {}):
 *           if 'nomen' in overrides:
 *               self.nomen = overrides['nomen']
 *           ...
 *
 *   genus Config { generis textus VERSION: "1.0" }
 *   ->
 *   class Config:
 *       VERSION: str = "1.0"  # Class-level (static)
 */

import type { GenusDeclaration, FieldDeclaration, FunctioDeclaration } from '../../../parser/ast';
import type { PyGenerator } from '../generator';
import { getVisibilityFromAnnotations } from '../../types';

export function genGenusDeclaration(node: GenusDeclaration, g: PyGenerator): string {
    const name = node.name.name;
    const typeParams = node.typeParameters ? `[${node.typeParameters.map(p => p.name).join(', ')}]` : '';

    // Python uses Protocol for interfaces
    const impl = node.implements ? `(${node.implements.map(i => i.name).join(', ')})` : '';

    const lines: string[] = [];
    lines.push(`${g.ind()}class ${name}${typeParams}${impl}:`);
    g.depth++;

    // Track if we have any content
    let hasContent = false;

    // Separate fields by type: static, regular, reactive
    const staticFields = node.fields.filter(f => f.isStatic);
    const instanceFields = node.fields.filter(f => !f.isStatic && !f.isReactive);
    const reactiveFields = node.fields.filter(f => f.isReactive);

    // Static fields first (class-level variables in Python)
    if (staticFields.length > 0) {
        for (const field of staticFields) {
            lines.push(genFieldDeclaration(field, g));
        }
        hasContent = true;
    }

    // Instance fields
    if (instanceFields.length > 0) {
        if (hasContent) {
            lines.push('');
        }
        for (const field of instanceFields) {
            lines.push(genFieldDeclaration(field, g));
        }
        hasContent = true;
    }

    // Reactive fields as properties with backing field
    if (reactiveFields.length > 0) {
        if (hasContent) {
            lines.push('');
        }
        for (const field of reactiveFields) {
            lines.push(genReactiveFieldDeclaration(field, g));
        }
        hasContent = true;
    }

    // Constructor (only for non-static instance fields)
    const nonStaticFields = node.fields.filter(f => !f.isStatic);
    if (nonStaticFields.length > 0 || node.constructor) {
        if (hasContent) {
            lines.push('');
        }
        lines.push(genAutoMergeConstructor(node, g));
        hasContent = true;
    }

    // User's creo as private method
    if (node.constructor) {
        lines.push('');
        lines.push(genCreoMethod(node.constructor, g));
    }

    // Methods
    if (node.methods.length > 0) {
        lines.push('');
        for (const method of node.methods) {
            lines.push(genMethodDeclaration(method, g));
        }
        hasContent = true;
    }

    // Empty class needs pass
    if (!hasContent) {
        lines.push(`${g.ind()}pass`);
    }

    g.depth--;
    return lines.join('\n');
}

/**
 * Generate auto-merge constructor for genus.
 *
 * WHY: Static fields are class-level and not set via __init__.
 */
function genAutoMergeConstructor(node: GenusDeclaration, g: PyGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}def __init__(self, overrides: dict = {}):`);
    g.depth++;

    // Only process non-static fields in constructor
    const instanceFields = node.fields.filter(f => !f.isStatic);

    if (instanceFields.length === 0 && !node.constructor) {
        lines.push(`${g.ind()}pass`);
    } else {
        // Apply each override if provided
        for (const field of instanceFields) {
            const fieldName = field.name.name;
            lines.push(`${g.ind()}if '${fieldName}' in overrides:`);
            g.depth++;
            lines.push(`${g.ind()}self.${fieldName} = overrides['${fieldName}']`);
            g.depth--;
        }

        // Call _creo() if user defined it
        if (node.constructor) {
            lines.push(`${g.ind()}self._creo()`);
        }
    }

    g.depth--;
    return lines.join('\n');
}

/**
 * Generate user's creo as a private method.
 */
function genCreoMethod(node: FunctioDeclaration, g: PyGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}def _creo(self):`);
    g.depth++;

    // Guard: abstract methods not yet supported
    if (!node.body) {
        throw new Error('Abstract methods not yet supported for Python target');
    }

    if (node.body.body.length === 0) {
        lines.push(`${g.ind()}pass`);
    } else {
        lines.push(g.genBlockStatementContent(node.body));
    }

    g.depth--;
    return lines.join('\n');
}

/**
 * Generate field declaration within a class.
 */
function genFieldDeclaration(node: FieldDeclaration, g: PyGenerator): string {
    const name = node.name.name;
    const type = g.genType(node.fieldType);
    const init = node.init ? ` = ${g.genExpression(node.init)}` : '';

    // Python doesn't have private/static modifiers in the same way
    // Use underscore prefix convention for private
    const visibility = getVisibilityFromAnnotations(node.annotations);
    const prefix = visibility === 'private' ? '_' : '';

    return `${g.ind()}${prefix}${name}: ${type}${init}`;
}

/**
 * Generate reactive (nexum) field declaration as property with backing field.
 *
 * TRANSFORMS:
 *   nexum numerus count: 0
 *   ->
 *   _count: int = 0
 *   @property
 *   def count(self) -> int:
 *       return self._count
 *   @count.setter
 *   def count(self, value: int):
 *       self._count = value
 *       if hasattr(self, '_pingo'):
 *           self._pingo()
 */
function genReactiveFieldDeclaration(node: FieldDeclaration, g: PyGenerator): string {
    const name = node.name.name;
    const type = g.genType(node.fieldType);
    const init = node.init ? ` = ${g.genExpression(node.init)}` : '';

    const lines: string[] = [];

    // Private backing field
    lines.push(`${g.ind()}_${name}: ${type}${init}`);
    lines.push('');

    // Getter property
    lines.push(`${g.ind()}@property`);
    lines.push(`${g.ind()}def ${name}(self) -> ${type}:`);
    g.depth++;
    lines.push(`${g.ind()}return self._${name}`);
    g.depth--;
    lines.push('');

    // Setter with invalidation
    lines.push(`${g.ind()}@${name}.setter`);
    lines.push(`${g.ind()}def ${name}(self, value: ${type}):`);
    g.depth++;
    lines.push(`${g.ind()}self._${name} = value`);
    lines.push(`${g.ind()}if hasattr(self, '_pingo'):`);
    g.depth++;
    lines.push(`${g.ind()}self._pingo()`);
    g.depth--;
    g.depth--;

    return lines.join('\n');
}

/**
 * Generate method declaration within a class.
 */
function genMethodDeclaration(node: FunctioDeclaration, g: PyGenerator): string {
    const asyncMod = node.async ? 'async ' : '';
    const name = node.name.name;

    // Add self as first parameter
    const params = ['self', ...node.params.map(p => g.genParameter(p))].join(', ');

    // Build return type
    let returnType = '';
    if (node.returnType) {
        let baseType = g.genType(node.returnType);
        if (node.async && node.generator) {
            baseType = `AsyncIterator[${baseType}]`;
        } else if (node.generator) {
            baseType = `Iterator[${baseType}]`;
        } else if (node.async) {
            baseType = `Awaitable[${baseType}]`;
        }
        returnType = ` -> ${baseType}`;
    }

    const prevInGenerator = g.inGenerator;
    g.inGenerator = node.generator;

    // Guard: abstract methods not yet supported
    if (!node.body) {
        throw new Error('Abstract methods not yet supported for Python target');
    }

    const header = `${g.ind()}${asyncMod}def ${name}(${params})${returnType}:`;
    g.depth++;
    const body = node.body.body.length === 0 ? `${g.ind()}pass` : g.genBlockStatementContent(node.body);
    g.depth--;

    g.inGenerator = prevInGenerator;

    return `${header}\n${body}`;
}
