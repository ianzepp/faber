/**
 * TypeScript Code Generator - GenusDeclaration
 *
 * TRANSFORMS:
 *   genus persona { textus nomen: "X" numerus aetas: 0 }
 *   ->
 *   class persona {
 *       nomen: string = "X";
 *       aetas: number = 0;
 *       constructor(overrides: { nomen?: string, aetas?: number } = {}) {
 *           if (overrides.nomen !== undefined) { this.nomen = overrides.nomen; }
 *           if (overrides.aetas !== undefined) { this.aetas = overrides.aetas; }
 *           this.creo(); // if defined
 *       }
 *       private creo() { ... } // user's creo body, no args
 *   }
 *
 * WHY: Auto-merge design - field defaults + property overrides merged before creo runs.
 */

import type { GenusDeclaration, FunctioDeclaration, FieldDeclaration } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement, genMethodDeclaration } from './functio';

export function genGenusDeclaration(node: GenusDeclaration, g: TsGenerator, semi: boolean): string {
    const name = node.name.name;
    const typeParams = node.typeParameters ? `<${node.typeParameters.map(p => p.name).join(', ')}>` : '';
    const ext = node.extends ? ` extends ${node.extends.name}` : '';
    const impl = node.implements ? ` implements ${node.implements.map(i => i.name).join(', ')}` : '';
    const abstractMod = node.isAbstract ? 'abstract ' : '';

    const lines: string[] = [];
    lines.push(`${g.ind()}${abstractMod}class ${name}${typeParams}${ext}${impl} {`);

    g.depth++;

    const sections: string[][] = [];

    if (node.fields.length > 0) {
        sections.push(node.fields.map(f => genFieldDeclaration(f, g, semi)));
    }

    // Always generate constructor for auto-merge
    sections.push([genAutoMergeConstructor(node, g, semi)]);

    // Emit user's creo as private no-args method (if defined)
    if (node.constructor) {
        sections.push([genCreoMethod(node.constructor, g)]);
    }

    if (node.methods.length > 0) {
        sections.push(node.methods.map(m => genMethodDeclaration(m, g)));
    }

    sections.forEach((section, index) => {
        if (index > 0) {
            lines.push('');
        }
        lines.push(...section);
    });

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

/**
 * Generate auto-merge constructor.
 *
 * WHY: The constructor handles merging field defaults with { ... } or de expr overrides,
 *      then calls creo() if defined. User never writes merge boilerplate.
 */
function genAutoMergeConstructor(node: GenusDeclaration, g: TsGenerator, semi: boolean): string {
    const lines: string[] = [];

    // Build overrides type: { fieldName?: fieldType, ... }
    const overrideProps = node.fields.map(f => {
        const fieldName = f.name.name;
        const fieldType = g.genType(f.fieldType);
        return `${fieldName}?: ${fieldType}`;
    });
    const overridesType = overrideProps.length > 0 ? `{ ${overrideProps.join(', ')} }` : 'Record<string, never>';

    lines.push(`${g.ind()}constructor(overrides: ${overridesType} = {}) {`);
    g.depth++;

    // Apply each override if provided
    for (const field of node.fields) {
        const fieldName = field.name.name;
        lines.push(`${g.ind()}if (overrides.${fieldName} !== undefined) { this.${fieldName} = overrides.${fieldName}${semi ? ';' : ''} }`);
    }

    // Call creo() if user defined it
    if (node.constructor) {
        lines.push(`${g.ind()}this.creo()${semi ? ';' : ''}`);
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

/**
 * Generate user's creo as a private no-args method.
 *
 * WHY: creo() is a post-initialization hook. By the time it runs,
 *      ego (this) already has merged field values. No args needed.
 */
function genCreoMethod(node: FunctioDeclaration, g: TsGenerator): string {
    const body = node.body ? genBlockStatement(node.body, g) : '{}';
    return `${g.ind()}private creo() ${body}`;
}

/**
 * Generate field declaration within a class.
 *
 * TRANSFORMS:
 *   textus nomen: "X" -> private nomen: string = "X"
 *   publicus numerus aetas: 0 -> aetas: number = 0
 *   nexum numerus count: 0 -> #count = 0; get count() { ... } set count(v) { ... }
 *
 * WHY: Reactive (nexum) fields emit getter/setter with invalidation hook.
 *      This allows libraries to track changes without special proxy magic.
 */
function genFieldDeclaration(node: FieldDeclaration, g: TsGenerator, semi: boolean): string {
    const name = node.name.name;
    const type = g.genType(node.fieldType);

    // Reactive fields emit private backing field + getter/setter
    if (node.isReactive) {
        const init = node.init ? ` = ${g.genExpression(node.init)}` : '';
        const lines: string[] = [];

        // Private backing field with # prefix
        lines.push(`${g.ind()}#${name}${init}${semi ? ';' : ''}`);

        // Getter
        lines.push(`${g.ind()}get ${name}(): ${type} { return this.#${name}; }`);

        // Setter with invalidation
        lines.push(`${g.ind()}set ${name}(v: ${type}) { this.#${name} = v; this.__invalidate?.('${name}'); }`);

        return lines.join('\n');
    }

    // Regular fields - public by default (struct semantics)
    const visibilityMod = node.visibility === 'private' ? 'private ' : node.visibility === 'protected' ? 'protected ' : '';
    const staticMod = node.isStatic ? 'static ' : '';
    const init = node.init ? ` = ${g.genExpression(node.init)}` : '';

    return `${g.ind()}${visibilityMod}${staticMod}${name}: ${type}${init}${semi ? ';' : ''}`;
}
