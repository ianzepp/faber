/**
 * Built-in Types - Latin Type System Vocabulary
 *
 * COMPILER PHASE
 * ==============
 * Lexical analysis (type vocabulary)
 *
 * ARCHITECTURE
 * ============
 * This module defines the built-in Latin type names that map to JavaScript/
 * TypeScript primitives and standard library types. Types follow Latin noun
 * declension patterns.
 *
 * TYPE CATEGORIES:
 * - primitive: Basic scalar types (textus, numerus, bivalens)
 * - collection: Container types (lista, tabula, copia)
 * - structural: Complex types (functio, promissum, erratum)
 * - iteration: Iterator/stream types (cursor, fluxus)
 *
 * LINGUISTIC DESIGN:
 * Latin noun genders and declensions are chosen for semantic clarity:
 * - Masculine: Active entities (numerus, cursor, fluxus)
 * - Feminine: Collections (lista, tabula, copia)
 * - Neuter: Abstract concepts (datum, erratum, signum)
 *
 * TARGET MAPPING:
 * Each type maps to a JavaScript/TypeScript type for code generation.
 * Generic types (Array, Map, Set, Promise) support type parameters.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Type stem (string) to lookup
 * OUTPUT: TypeEntry with declension info and target language mapping
 * ERRORS: N/A - lookup operations do not fail
 *
 * INVARIANTS
 * ==========
 * INV-1: Type stems are lowercase
 * INV-2: Lookup is case-sensitive (exact match required)
 * INV-3: Generic types are marked with generic flag
 * INV-4: Each type has a unique stem
 *
 * @module lexicon/types-builtin
 */

import type { NounEntry } from './types';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Extended noun entry with target language mapping.
 *
 * DESIGN: Extends NounEntry because types decline like nouns. Adds target
 *         language information for code generation.
 */
export interface TypeEntry extends NounEntry {
    category: 'primitive' | 'collection' | 'structural' | 'iteration';
    generic?: boolean;
    // WHY: 3rd declension neuters like tempus/temporis have nominatives that
    //      differ from their stems. This field allows parseType to recognize them.
    nominative?: string;
}

// =============================================================================
// BUILT-IN TYPE DEFINITIONS
// =============================================================================

/**
 * Complete vocabulary of Latin type names.
 *
 * WHY: Organized by category (primitives, collections, etc.) for clarity.
 *      Each type's gender and declension chosen for linguistic appropriateness.
 *
 * CASE: Stems are lowercase. Lookup is case-sensitive (exact match required).
 */
export const builtinTypes: TypeEntry[] = [
    // ---------------------------------------------------------------------------
    // Primitives
    // ---------------------------------------------------------------------------

    // WHY: textus (4th decl masculine) - "texture, fabric, text"
    {
        stem: 'text',
        declension: 4,
        gender: 'masculine',
        meaning: 'text/string',
        category: 'primitive',
    },
    // WHY: numerus (2nd decl masculine) - "number, count"
    {
        stem: 'numer',
        declension: 2,
        gender: 'masculine',
        meaning: 'number',
        category: 'primitive',
    },
    // WHY: bivalens (3rd decl masculine) - "two-valued, having two values"
    //      Nominative differs from stem (bivalens vs bivalen-)
    {
        stem: 'bivalen',
        nominative: 'bivalens',
        declension: 3,
        gender: 'masculine',
        meaning: 'boolean',
        category: 'primitive',
    },
    // WHY: fractus (2nd decl masculine) - "broken, fractional" (floating point)
    {
        stem: 'fract',
        declension: 2,
        gender: 'masculine',
        meaning: 'floating point',
        category: 'primitive',
    },
    // WHY: decimus (2nd decl masculine) - "tenth, decimal"
    {
        stem: 'decim',
        declension: 2,
        gender: 'masculine',
        meaning: 'decimal',
        category: 'primitive',
    },
    // WHY: magnus (2nd decl masculine) - "great, large" (bigint)
    {
        stem: 'magn',
        declension: 2,
        gender: 'masculine',
        meaning: 'bigint',
        category: 'primitive',
    },
    // WHY: signum (2nd decl neuter) - "sign, mark, token"
    {
        stem: 'sign',
        declension: 2,
        gender: 'neuter',
        meaning: 'symbol',
        category: 'primitive',
    },
    // WHY: incertum (2nd decl neuter) - "uncertain, undefined"
    {
        stem: 'incert',
        declension: 2,
        gender: 'neuter',
        meaning: 'undefined',
        category: 'primitive',
    },
    // WHY: nihil (irregular neuter) - "nothing, null"
    {
        stem: 'nihil',
        declension: 3,
        gender: 'neuter',
        meaning: 'null',
        category: 'primitive',
    },
    // WHY: octeti (2nd decl masculine plural) - "octets, bytes"
    //      Raw byte data, no encoding assumptions. Distinct from textus (UTF-8 text).
    //      Nominative uses plural form since bytes are inherently a collection.
    {
        stem: 'octet',
        nominative: 'octeti',
        declension: 2,
        gender: 'masculine',
        meaning: 'bytes/octets',
        category: 'primitive',
    },
    // WHY: objectum (2nd decl neuter) - "object, thing thrown before"
    //      Generic object type for dynamic data structures.
    {
        stem: 'object',
        declension: 2,
        gender: 'neuter',
        meaning: 'object',
        category: 'primitive',
    },

    // ---------------------------------------------------------------------------
    // Collections (Generic)
    // ---------------------------------------------------------------------------

    // WHY: lista (1st decl feminine) - "list, edge, border" (feminine for containers)
    {
        stem: 'list',
        declension: 1,
        gender: 'feminine',
        meaning: 'list/array',
        category: 'collection',
        generic: true,
    },
    // WHY: tabula (1st decl feminine) - "board, tablet, table"
    {
        stem: 'tabul',
        declension: 1,
        gender: 'feminine',
        meaning: 'table/map',
        category: 'collection',
        generic: true,
    },
    // WHY: copia (1st decl feminine) - "abundance, supply, collection"
    {
        stem: 'copi',
        declension: 1,
        gender: 'feminine',
        meaning: 'set/collection',
        category: 'collection',
        generic: true,
    },

    // ---------------------------------------------------------------------------
    // Structural Types
    // ---------------------------------------------------------------------------

    // WHY: unio (3rd decl feminine) - "union, oneness" (type union)
    {
        stem: 'union',
        nominative: 'unio',
        declension: 3,
        gender: 'feminine',
        meaning: 'union',
        category: 'structural',
        generic: true,
    },
    // WHY: promissum (2nd decl neuter) - "promise, guarantee" (neuter for abstract concepts)
    {
        stem: 'promiss',
        declension: 2,
        gender: 'neuter',
        meaning: 'promise',
        category: 'structural',
        generic: true,
    },
    // WHY: forsitan (adverb as type) - "perhaps, maybe" (optional type)
    {
        stem: 'forsitan',
        declension: 3,
        gender: 'neuter',
        meaning: 'optional',
        category: 'structural',
        generic: true,
    },
    // WHY: fors (3rd decl feminine) - "chance, fortune" (result type)
    {
        stem: 'for',
        declension: 3,
        gender: 'feminine',
        meaning: 'result',
        category: 'structural',
        generic: true,
    },
    // WHY: tempus (3rd decl neuter) - "time, period"
    //      Nominative differs from stem (tempus vs tempor-)
    {
        stem: 'tempor',
        nominative: 'tempus',
        declension: 3,
        gender: 'neuter',
        meaning: 'time/date',
        category: 'structural',
    },
    // WHY: erratum (2nd decl neuter) - "error, mistake" (neuter participle)
    {
        stem: 'errat',
        declension: 2,
        gender: 'neuter',
        meaning: 'error',
        category: 'structural',
    },
    // WHY: vacuum (2nd decl neuter) - "empty space, void"
    {
        stem: 'vacu',
        declension: 2,
        gender: 'neuter',
        meaning: 'void/empty',
        category: 'structural',
    },
    // WHY: quodlibet (3rd decl neuter) - "whatever you please, anything"
    {
        stem: 'quodlibet',
        declension: 3,
        gender: 'neuter',
        meaning: 'any',
        category: 'structural',
    },
    // WHY: ignotum (2nd decl neuter) - "unknown, unfamiliar"
    {
        stem: 'ignot',
        declension: 2,
        gender: 'neuter',
        meaning: 'unknown',
        category: 'structural',
    },

    // ---------------------------------------------------------------------------
    // Iteration & Streaming
    // ---------------------------------------------------------------------------

    // WHY: cursor (3rd decl masculine) - "runner, iterator" (agent noun, masculine)
    {
        stem: 'cursor',
        declension: 3,
        gender: 'masculine',
        meaning: 'cursor/iterator',
        category: 'iteration',
        generic: true,
    },
    // WHY: fluxus (4th decl masculine) - "flow, stream, flux"
    {
        stem: 'flux',
        declension: 4,
        gender: 'masculine',
        meaning: 'flow/stream',
        category: 'iteration',
        generic: true,
    },
    // WHY: futuracursor (compound) - "future iterator" (async iterator)
    {
        stem: 'futuracursor',
        declension: 3,
        gender: 'masculine',
        meaning: 'async iterator',
        category: 'iteration',
        generic: true,
    },
    // WHY: futurusfluxus (compound) - "future stream" (async stream)
    {
        stem: 'futurusflux',
        declension: 4,
        gender: 'masculine',
        meaning: 'async stream',
        category: 'iteration',
        generic: true,
    },

    // ---------------------------------------------------------------------------
    // Systems Types (Zig/WASM targets)
    // ---------------------------------------------------------------------------

    // WHY: indicium (2nd decl neuter) - "indication, pointer"
    {
        stem: 'indici',
        declension: 2,
        gender: 'neuter',
        meaning: 'pointer',
        category: 'structural',
        generic: true,
    },
    // WHY: refera (1st decl feminine) - "reference" (from refero - to refer back)
    {
        stem: 'refer',
        declension: 1,
        gender: 'feminine',
        meaning: 'reference',
        category: 'structural',
        generic: true,
    },

    // ---------------------------------------------------------------------------
    // Utility Types (TypeScript only)
    // ---------------------------------------------------------------------------

    // WHY: pars (3rd decl feminine) - "part, portion" (Partial<T>)
    {
        stem: 'par',
        declension: 3,
        gender: 'feminine',
        meaning: 'partial',
        category: 'structural',
        generic: true,
    },
    // WHY: totum (2nd decl neuter) - "whole, total" (Required<T>)
    {
        stem: 'tot',
        declension: 2,
        gender: 'neuter',
        meaning: 'required',
        category: 'structural',
        generic: true,
    },
    // WHY: lectum (2nd decl neuter) - "read, chosen" (Readonly<T>)
    {
        stem: 'lect',
        declension: 2,
        gender: 'neuter',
        meaning: 'readonly',
        category: 'structural',
        generic: true,
    },
    // WHY: registrum (2nd decl neuter) - "record, registry" (Record<K, V>)
    {
        stem: 'registr',
        declension: 2,
        gender: 'neuter',
        meaning: 'record',
        category: 'structural',
        generic: true,
    },
    // WHY: selectum (2nd decl neuter) - "selected, picked" (Pick<T, K>)
    {
        stem: 'select',
        declension: 2,
        gender: 'neuter',
        meaning: 'pick',
        category: 'structural',
        generic: true,
    },
    // WHY: omissum (2nd decl neuter) - "omitted, left out" (Omit<T, K>)
    {
        stem: 'omiss',
        declension: 2,
        gender: 'neuter',
        meaning: 'omit',
        category: 'structural',
        generic: true,
    },
    // WHY: extractum (2nd decl neuter) - "extracted, drawn out" (Extract<T, U>)
    {
        stem: 'extract',
        declension: 2,
        gender: 'neuter',
        meaning: 'extract',
        category: 'structural',
        generic: true,
    },
    // WHY: exclusum (2nd decl neuter) - "excluded, shut out" (Exclude<T, U>)
    {
        stem: 'exclus',
        declension: 2,
        gender: 'neuter',
        meaning: 'exclude',
        category: 'structural',
        generic: true,
    },
    // WHY: nonnihil (compound) - "not null" (NonNullable<T>)
    {
        stem: 'nonnihil',
        declension: 3,
        gender: 'neuter',
        meaning: 'non-nullable',
        category: 'structural',
        generic: true,
    },
    // WHY: reditus (4th decl masculine) - "return, going back" (ReturnType<T>)
    {
        stem: 'redit',
        declension: 4,
        gender: 'masculine',
        meaning: 'return type',
        category: 'structural',
        generic: true,
    },
    // WHY: parametra (1st decl feminine plural) - "parameters" (Parameters<T>)
    {
        stem: 'parametr',
        declension: 1,
        gender: 'feminine',
        meaning: 'parameters',
        category: 'structural',
        generic: true,
    },
];

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

/**
 * Type lookup map for O(1) access.
 *
 * PERF: Pre-computed Map is faster than linear array search.
 */
const typeMap = new Map(builtinTypes.map(t => [t.stem, t]));

/**
 * Check if a stem is a built-in type.
 *
 * @param stem - The type stem to check (e.g., "text", "numer")
 * @returns true if stem is a built-in type, false otherwise
 */
export function isBuiltinType(stem: string): boolean {
    return typeMap.has(stem);
}

/**
 * Get type metadata for a stem.
 *
 * @param stem - The type stem to lookup
 * @returns TypeEntry if stem is a built-in type, undefined otherwise
 */
export function getBuiltinType(stem: string): TypeEntry | undefined {
    return typeMap.get(stem);
}
