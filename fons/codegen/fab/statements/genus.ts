/**
 * Faber Code Generator - GenusDeclaration
 *
 * TRANSFORMS:
 *   GenusDeclaration -> genus name<T>? sub parent? implet interfaces? { fields, methods }
 */

import type { GenusDeclaration, FunctioDeclaration, FieldDeclaration } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genGenusDeclaration(node: GenusDeclaration, g: FabGenerator): string {
    const parts: string[] = [];

    // Abstract modifier
    if (node.isAbstract) {
        parts.push('abstractus');
    }

    parts.push('genus');
    parts.push(node.name.name);

    // Type parameters
    if (node.typeParameters && node.typeParameters.length > 0) {
        parts[parts.length - 1] += `<${node.typeParameters.map(p => p.name).join(', ')}>`;
    }

    // Inheritance
    if (node.extends) {
        parts.push('sub', node.extends.name);
    }

    // Implements
    if (node.implements && node.implements.length > 0) {
        parts.push('implet', node.implements.map(i => i.name).join(', '));
    }

    const lines: string[] = [];
    lines.push(`${g.ind()}${parts.join(' ')} {`);

    g.depth++;

    // Fields
    for (const field of node.fields) {
        lines.push(genFieldDeclaration(field, g));
    }

    // Blank line between fields and constructor/methods
    if (node.fields.length > 0 && (node.constructor || node.methods.length > 0)) {
        lines.push('');
    }

    // Constructor (creo)
    if (node.constructor) {
        lines.push(genCreoDeclaration(node.constructor, g));
    }

    // Blank line between constructor and methods
    if (node.constructor && node.methods.length > 0) {
        lines.push('');
    }

    // Methods
    for (const method of node.methods) {
        lines.push(genMethodDeclaration(method, g));
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

function genFieldDeclaration(node: FieldDeclaration, g: FabGenerator): string {
    const parts: string[] = [];

    // Visibility
    if (node.visibility === 'private') {
        parts.push('privatus');
    } else if (node.visibility === 'protected') {
        parts.push('protectus');
    }

    // Static
    if (node.isStatic) {
        parts.push('generis');
    }

    // Reactive
    if (node.isReactive) {
        parts.push('nexum');
    }

    // Type
    parts.push(g.genType(node.fieldType));

    // Name
    parts.push(node.name.name);

    // Default value (use : for declarative defaults)
    if (node.init) {
        parts.push(':');
        parts.push(g.genExpression(node.init));
    }

    return `${g.ind()}${parts.join(' ')}`;
}

function genCreoDeclaration(node: FunctioDeclaration, g: FabGenerator): string {
    const params = node.params.map(p => g.genParameter(p)).join(', ');
    const body = node.body ? genBlockStatement(node.body, g) : '{}';

    return `${g.ind()}functio creo(${params}) ${body}`;
}

function genMethodDeclaration(node: FunctioDeclaration, g: FabGenerator): string {
    const lines: string[] = [];

    // Build annotation modifiers
    const annotationMods: string[] = [];

    // Visibility
    if (node.visibility === 'private') {
        annotationMods.push('privata');
    } else if (node.visibility === 'protected') {
        annotationMods.push('protecta');
    }

    // Abstract
    if (node.isAbstract) {
        annotationMods.push('abstracta');
    }

    // Async/generator
    if (node.async) {
        annotationMods.push('futura');
    }
    if (node.generator) {
        annotationMods.push('cursor');
    }

    // Emit annotation line if we have modifiers
    if (annotationMods.length > 0) {
        lines.push(`${g.ind()}@ ${annotationMods.join(' ')}`);
    }

    const parts: string[] = [];

    parts.push('functio');
    parts.push(node.name.name);

    // Parameters
    const params = node.params.map(p => g.genParameter(p)).join(', ');
    parts[parts.length - 1] += `(${params})`;

    // curata NAME stays inline (binds a name)
    if (node.curatorName) {
        parts.push('curata');
        parts.push(node.curatorName);
    }

    // Return type
    if (node.returnType) {
        parts.push('->');
        parts.push(g.genType(node.returnType));
    }

    // Body
    if (node.body) {
        parts.push(genBlockStatement(node.body, g));
    }

    lines.push(`${g.ind()}${parts.join(' ')}`);

    return lines.join('\n');
}
