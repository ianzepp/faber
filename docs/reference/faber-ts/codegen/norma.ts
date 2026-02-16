/**
 * Norma Registry - TypeScript stdlib translations.
 *
 * Data is generated at build time by: bun run build:norma
 * Source: fons/norma/ → fons/faber-ts/codegen/norma.ts.gen.ts
 */

import { norma, radixForms, type Translation } from './norma.ts.gen';
import { parseMethodum, parseMethodumWithStem } from './morphology';

// =============================================================================
// TYPES
// =============================================================================

/** Translation for a method call (re-export for compatibility) */
export type VerteTranslation = Translation;

// =============================================================================
// LOOKUP API
// =============================================================================

/**
 * Look up a norma translation for a method call.
 *
 * WHY: This is the core lookup bridging Latin method names to target implementations.
 *   - "norma" (Latin: "rule/standard") defines the canonical API contract
 *   - Keyed by collection (lista, tabula, textus) then method name
 *   - Returns undefined for user-defined methods (pass-through to native)
 *
 * WHY: The two-level lookup (collection → method):
 *   - Same method name can have different implementations per collection
 *   - e.g., lista.adde() vs tabula.adde() map to different target code
 *   - Flat namespace would require prefixing (lista_adde) - uglier API
 *
 * WHY: _target param is currently unused but preserved:
 *   - norma.gen.ts is TS-specific; future: norma.rs.gen.ts, etc.
 *   - Signature anticipates multi-target without breaking callers
 *
 * @param target Target language (only 'ts' supported)
 * @param collection Collection name (lista, tabula, copia)
 * @param method Method name (adde, filtrata, etc.)
 * @returns Translation if found, undefined otherwise
 */
export function getNormaTranslation(_target: string, collection: string, method: string): VerteTranslation | undefined {
    return norma[collection]?.methods[method];
}

/**
 * Apply a norma template translation.
 *
 * @param template Template string with § placeholders
 * @param params Parameter names from @ verte
 * @param obj The object/receiver expression
 * @param args The argument expressions
 * @returns Generated code string
 */
export function applyNormaTemplate(template: string, params: string[], obj: string, args: string[]): string {
    // Build value map: ego -> obj, other params -> args
    const values: string[] = [];
    const argsCopy = [...args];
    for (const param of params) {
        if (param === 'ego') {
            values.push(obj);
        }
        else {
            values.push(argsCopy.shift() || '');
        }
    }

    // Replace § placeholders - supports both positional and indexed
    // §  = next value (implicit positional)
    // §0, §1, etc. = explicit index into values array
    let result = template;
    let implicitIdx = 0;

    result = result.replace(/§(\d+)?/g, (_, indexStr) => {
        if (indexStr !== undefined) {
            const idx = parseInt(indexStr, 10);
            return values[idx] || '';
        }
        return values[implicitIdx++] || '';
    });

    return result;
}

/**
 * Check if a method has a norma definition.
 */
export function hasNormaMethod(_target: string, collection: string, method: string): boolean {
    return norma[collection]?.methods[method] !== undefined;
}

/**
 * Get all collections defined in norma.
 */
export function getNormaCollections(): string[] {
    return Object.keys(norma);
}

/**
 * Find receiver-type collections that define a given method.
 *
 * WHY: When the semantic analyzer cannot resolve a receiver type (UNKNOWN),
 *      TS codegen must not guess. This helper lets codegen detect "this looks
 *      like a stdlib method" and emit a compiler error prompting a type fix.
 */
export function getNormaReceiverCollectionsForMethod(_target: string, method: string): string[] {
    const receiverCollections = ['lista', 'tabula', 'copia', 'textus'];
    const matches: string[] = [];
    for (const collection of receiverCollections) {
        if (norma[collection]?.methods[method]) {
            matches.push(collection);
        }
    }
    return matches;
}

/**
 * Apply a norma module function call (no receiver object).
 *
 * For module functions like mathesis.pavimentum(x) or solum.lege(path),
 * there's no 'ego' receiver - just direct function arguments.
 *
 * @param target Target language (only 'ts' supported)
 * @param module Module name (mathesis, solum, etc.)
 * @param func Function name (pavimentum, lege, etc.)
 * @param args The argument expressions
 * @returns Generated code string, or undefined if not found
 */
export function applyNormaModuleCall(_target: string, module: string, func: string, args: string[]): string | undefined {
    const translation = norma[module]?.methods[func];
    if (!translation?.template) {
        return undefined;
    }

    const values = [...args];
    let result = translation.template;
    let implicitIdx = 0;

    result = result.replace(/§(\d+)?/g, (_, indexStr) => {
        if (indexStr !== undefined) {
            const idx = parseInt(indexStr, 10);
            return values[idx] || '';
        }
        return values[implicitIdx++] || '';
    });

    return result;
}

// =============================================================================
// MORPHOLOGY VALIDATION
// =============================================================================

/** Result of morphology validation */
export interface MorphologyValidation {
    valid: boolean;
    error?: string;
    stem?: string;
    form?: string;
}

/**
 * Get the radixForms for a method in a collection.
 */
export function getNormaRadixForms(collection: string, method: string): string[] | undefined {
    return radixForms[collection]?.[method];
}

/** Receiver ownership for method calls */
export type ReceiverOwnership = 'de' | 'in' | undefined;

/**
 * Get the implied receiver ownership for a stdlib method.
 *
 * WHY: Latin morphology encodes mutation semantics:
 *   - imperativus (adde, filtra) → mutates receiver → 'in' (&mut self)
 *   - perfectum (addita, filtrata) → returns new → 'de' (&self)
 */
export function getReceiverOwnership(collection: string, methodName: string): ReceiverOwnership {
    if (!norma[collection]) return undefined;

    const hasMethod = norma[collection]!.methods[methodName] !== undefined;
    if (!hasMethod) return undefined;

    const forms = getNormaRadixForms(collection, methodName);

    if (forms && forms.length > 0) {
        const stem = forms[0]!;
        const parsed = parseMethodumWithStem(methodName, stem);
        if (parsed) {
            return parsed.flags.mutare ? 'in' : 'de';
        }
    }

    const parsed = parseMethodum(methodName);
    if (parsed) {
        return parsed.flags.mutare ? 'in' : 'de';
    }

    return undefined;
}

/**
 * Validate morphology for a method call on a stdlib collection.
 */
export function validateMorphology(collection: string, methodName: string): MorphologyValidation {
    if (!norma[collection]) {
        return { valid: true };
    }

    const directRadix = getNormaRadixForms(collection, methodName);

    if (directRadix) {
        const declaredStem = directRadix[0]!;
        const declaredForms = directRadix.slice(1);

        const stemParsed = parseMethodumWithStem(methodName, declaredStem);
        const greedyParsed = parseMethodum(methodName);

        let parsed = stemParsed;
        if (stemParsed && !declaredForms.includes(stemParsed.form)) {
            if (greedyParsed && declaredForms.includes(greedyParsed.form)) {
                parsed = { ...greedyParsed, stem: declaredStem };
            }
        }

        if (!parsed) {
            parsed = stemParsed || greedyParsed;
        }

        if (parsed) {
            if (parsed.stem !== declaredStem) {
                return {
                    valid: false,
                    error: `Morphology mismatch: '${methodName}' has stem '${parsed.stem}', expected '${declaredStem}'`,
                    stem: parsed.stem,
                    form: parsed.form,
                };
            }

            if (!declaredForms.includes(parsed.form)) {
                return {
                    valid: false,
                    error: `Morphology form '${parsed.form}' not declared for stem '${declaredStem}'. Valid forms: ${declaredForms.join(', ')}`,
                    stem: parsed.stem,
                    form: parsed.form,
                };
            }
        }

        return { valid: true };
    }

    const hasMethod = norma[collection]!.methods[methodName] !== undefined;
    if (hasMethod) {
        return { valid: true };
    }

    // Check if method could be an undeclared form of a known verb stem
    const collectionRadix = radixForms[collection];
    if (collectionRadix) {
        for (const [_, forms] of Object.entries(collectionRadix)) {
            const declaredStem = forms[0]!;
            const declaredForms = forms.slice(1);

            const stemParsed = parseMethodumWithStem(methodName, declaredStem);
            const greedyParsed = parseMethodum(methodName);

            let parsed = stemParsed;
            if (stemParsed && !declaredForms.includes(stemParsed.form)) {
                if (greedyParsed && declaredForms.includes(greedyParsed.form)) {
                    parsed = { ...greedyParsed, stem: declaredStem };
                }
            }

            if (parsed) {
                if (!declaredForms.includes(parsed.form)) {
                    return {
                        valid: false,
                        error: `Unknown method '${methodName}': stem '${parsed.stem}' exists but form '${parsed.form}' is not declared. Valid forms: ${declaredForms.join(', ')}`,
                        stem: parsed.stem,
                        form: parsed.form,
                    };
                }
                break;
            }
        }
    }

    return { valid: true };
}
