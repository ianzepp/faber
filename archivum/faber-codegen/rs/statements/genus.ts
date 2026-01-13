/**
 * Rust Code Generator - GenusDeclaration
 *
 * TRANSFORMS:
 *   genus Persona { textus nomen, numerus aetas }
 *   -> struct Persona { nomen: String, aetas: f64 }
 *      impl Persona { ... methods ... }
 */

import type { GenusDeclaration, FieldDeclaration } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement, genMethodDeclaration } from './functio';

export function genGenusDeclaration(node: GenusDeclaration, g: RsGenerator): string {
    const name = node.name.name;
    const typeParams = node.typeParameters ? `<${node.typeParameters.map(p => p.name).join(', ')}>` : '';

    const lines: string[] = [];

    // Struct definition
    lines.push(`${g.ind()}struct ${name}${typeParams} {`);
    g.depth++;

    for (const field of node.fields) {
        lines.push(genFieldDeclaration(field, g));
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    // Impl block for methods
    if (node.methods.length > 0 || node.constructor) {
        lines.push('');
        lines.push(`${g.ind()}impl${typeParams} ${name}${typeParams} {`);
        g.depth++;

        // Constructor as new() if defined
        if (node.constructor) {
            lines.push(genConstructorAsNew(node, g));
        }

        for (const method of node.methods) {
            lines.push(genMethodDeclaration(method, g));
        }

        g.depth--;
        lines.push(`${g.ind()}}`);
    }

    return lines.join('\n');
}

/**
 * Generate field declaration within a struct.
 */
function genFieldDeclaration(node: FieldDeclaration, g: RsGenerator): string {
    const visibility = node.visibility === 'private' ? '' : 'pub ';
    const name = node.name.name;
    const type = g.genType(node.fieldType);

    return `${g.ind()}${visibility}${name}: ${type},`;
}

/**
 * Generate constructor as Rust new() method.
 */
function genConstructorAsNew(node: GenusDeclaration, g: RsGenerator): string {
    const lines: string[] = [];
    const typeParams = node.typeParameters ? `<${node.typeParameters.map(p => p.name).join(', ')}>` : '';
    const name = node.name.name;

    lines.push(`${g.ind()}pub fn new() -> ${name}${typeParams} {`);
    g.depth++;

    // Initialize fields with defaults
    lines.push(`${g.ind()}${name} {`);
    g.depth++;

    for (const field of node.fields) {
        const fieldName = field.name.name;
        if (field.init) {
            lines.push(`${g.ind()}${fieldName}: ${g.genExpression(field.init)},`);
        } else {
            lines.push(`${g.ind()}${fieldName}: Default::default(),`);
        }
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
