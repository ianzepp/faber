/**
 * Norma Registry - Loads stdlib definitions from fons/norma/*.fab
 *
 * VERTICAL SLICE: Proves annotation-driven codegen works.
 * Currently uses hardcoded data to avoid circular dependencies.
 * TODO: Load from norma/*.fab files at build time, not runtime.
 *
 * See consilia/futura/norma-faber.md for the full design.
 */

// =============================================================================
// TYPES
// =============================================================================

/** Translation for a single target */
export interface VerteTranslation {
    /** Simple method name (e.g., 'push') */
    method?: string;
    /** Template string with S placeholders */
    template?: string;
    /** Parameter names for template form */
    params?: string[];
}

/** Method entry in the registry */
export interface NormaMethod {
    /** Method name (e.g., 'adde') */
    name: string;
    /** Translations per target */
    translations: Map<string, VerteTranslation>;
    /** Morphology forms from @ radix (if present) */
    radixForms?: string[];
}

/** Collection entry in the registry */
export interface NormaCollection {
    /** Collection name (e.g., 'lista') */
    name: string;
    /** Native type mappings from @ innatum */
    innatum: Map<string, string>;
    /** Methods defined for this collection */
    methods: Map<string, NormaMethod>;
}

// =============================================================================
// HARDCODED REGISTRY (vertical slice - avoids circular dependency)
// =============================================================================
// TODO: Generate this from norma/*.fab at build time, not runtime

/** Global registry - hardcoded for vertical slice */
const registry: Map<string, NormaCollection> = new Map([
    ['lista', {
        name: 'lista',
        innatum: new Map([
            ['ts', 'Array'],
            ['py', 'list'],
            ['rs', 'Vec'],
            ['cpp', 'std::vector'],
            ['zig', 'Lista'],
        ]),
        methods: new Map([
            ['adde', {
                name: 'adde',
                translations: new Map([
                    ['ts', { method: 'push' }],
                    ['py', { method: 'append' }],
                    ['rs', { method: 'push' }],
                    ['cpp', { method: 'push_back' }],
                    ['zig', { template: '§.adde(§, §)', params: ['ego', 'elem', 'alloc'] }],
                ]),
                radixForms: ['add', 'imperativus', 'perfectum'],
            }],
            ['addita', {
                name: 'addita',
                translations: new Map([
                    ['ts', { template: '[...§, §]', params: ['ego', 'elem'] }],
                    ['py', { template: '[*§, §]', params: ['ego', 'elem'] }],
                    ['rs', { template: '{ let mut v = §.clone(); v.push(§); v }', params: ['ego', 'elem'] }],
                    ['cpp', { template: '[&]{ auto v = §; v.push_back(§); return v; }()', params: ['ego', 'elem'] }],
                    ['zig', { template: '§.addita(§, §)', params: ['ego', 'elem', 'alloc'] }],
                ]),
            }],
        ]),
    }],
]);

// =============================================================================
// LOOKUP API
// =============================================================================

/**
 * Look up a norma translation for a method call.
 *
 * @param target Target language (ts, py, rs, cpp, zig)
 * @param collection Collection name (lista, tabula, copia)
 * @param method Method name (adde, filtrata, etc.)
 * @returns Translation if found, undefined otherwise
 */
export function getNormaTranslation(
    target: string,
    collection: string,
    method: string,
): VerteTranslation | undefined {
    const coll = registry.get(collection);
    if (!coll) return undefined;

    const m = coll.methods.get(method);
    if (!m) return undefined;

    return m.translations.get(target);
}

/**
 * Apply a norma template translation.
 *
 * @param template Template string with S placeholders
 * @param params Parameter names from @ verte
 * @param obj The object/receiver expression
 * @param args The argument expressions
 * @returns Generated code string
 */
export function applyNormaTemplate(
    template: string,
    params: string[],
    obj: string,
    args: string[],
): string {
    // Build value map: ego -> obj, other params -> args
    const values: string[] = [];
    for (const param of params) {
        if (param === 'ego') {
            values.push(obj);
        }
        else if (param === 'alloc') {
            // WHY: Allocator is handled specially by Zig codegen, skip for now
            values.push('allocator');
        }
        else {
            // Take next arg
            values.push(args.shift() || '');
        }
    }

    // Replace S placeholders positionally
    let result = template;
    let idx = 0;
    // WHY: Use regex to replace each S one at a time
    result = result.replace(/S/g, () => values[idx++] || '');

    return result;
}

/**
 * Check if a method has a norma definition for the given target.
 */
export function hasNormaMethod(
    target: string,
    collection: string,
    method: string,
): boolean {
    return getNormaTranslation(target, collection, method) !== undefined;
}

/**
 * Get all collections defined in norma files.
 */
export function getNormaCollections(): string[] {
    return Array.from(registry.keys());
}
