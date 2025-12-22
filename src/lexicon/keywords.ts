/**
 * Keywords - Latin Reserved Words and Operators
 *
 * COMPILER PHASE
 * ==============
 * Lexical analysis (keyword recognition)
 *
 * ARCHITECTURE
 * ============
 * This module defines the reserved Latin keywords for Faber Romanus. Keywords
 * are words with special syntactic meaning that cannot be used as identifiers.
 * They map directly to control flow, declarations, operators, and literal values
 * in the target language.
 *
 * Unlike identifiers, keywords are case-insensitive in Latin (following the
 * historical tradition that Latin had no distinction between upper/lowercase
 * until medieval times). "SI", "Si", and "si" all mean "if".
 *
 * Keywords are categorized by their syntactic role:
 * - control: Control flow statements (si, dum, redde)
 * - declaration: Variable/function declarations (esto, functio)
 * - operator: Logical operators (et, aut, non)
 * - value: Literal values (verum, falsum, nihil)
 * - preposition: Case-governing prepositions (in, ex, cum, ad)
 * - modifier: Function/variable modifiers (futura for async)
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin word (string) to check
 * OUTPUT: KeywordEntry if word is a keyword, undefined if not
 * ERRORS: N/A - lookup operations do not fail
 *
 * INVARIANTS
 * ==========
 * INV-1: All keywords are stored in lowercase
 * INV-2: Keyword lookup is case-insensitive (normalized to lowercase)
 * INV-3: Keywords are disjoint from type names (types use TitleCase)
 * INV-4: Map lookup is O(1) for performance
 *
 * @module lexicon/keywords
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Metadata for a Latin keyword.
 *
 * DESIGN: Category enables different parsing strategies for different keyword types.
 *         For example, control keywords expect blocks, operators expect operands.
 */
export interface KeywordEntry {
    latin: string;
    meaning: string;
    category: 'control' | 'declaration' | 'operator' | 'value' | 'preposition' | 'modifier';
}

// =============================================================================
// KEYWORD DEFINITIONS
// =============================================================================

/**
 * Complete list of Latin reserved words.
 *
 * WHY: Organized by category for maintainability. Control flow first, then
 *      declarations, then smaller categories. This mirrors typical language specs.
 */
export const keywords: KeywordEntry[] = [
    // ---------------------------------------------------------------------------
    // Control Flow
    // ---------------------------------------------------------------------------

    // WHY: Latin uses imperative mood (command forms) for control structures
    { latin: 'si', meaning: 'if', category: 'control' },
    { latin: 'aliter', meaning: 'else', category: 'control' },
    { latin: 'dum', meaning: 'while', category: 'control' },
    { latin: 'fac', meaning: 'do', category: 'control' },
    { latin: 'pro', meaning: 'for', category: 'control' },
    { latin: 'elige', meaning: 'switch', category: 'control' },
    { latin: 'ergo', meaning: 'then', category: 'control' },
    { latin: 'nulla', meaning: 'none/empty', category: 'operator' },
    { latin: 'nonnulla', meaning: 'some/non-empty', category: 'operator' },
    { latin: 'quando', meaning: 'case', category: 'control' },
    { latin: 'rumpe', meaning: 'break', category: 'control' },
    { latin: 'perge', meaning: 'continue', category: 'control' },
    { latin: 'redde', meaning: 'return', category: 'control' },
    { latin: 'custodi', meaning: 'guard', category: 'control' },
    { latin: 'adfirma', meaning: 'assert', category: 'control' },
    { latin: 'tempta', meaning: 'try', category: 'control' },
    { latin: 'cape', meaning: 'catch', category: 'control' },
    { latin: 'demum', meaning: 'finally', category: 'control' },
    { latin: 'iace', meaning: 'throw', category: 'control' },
    { latin: 'scribe', meaning: 'print', category: 'control' },
    { latin: 'exspecta', meaning: 'await', category: 'control' },

    // ---------------------------------------------------------------------------
    // Declarations
    // ---------------------------------------------------------------------------

    // WHY: "esto" (be!) and "fixum" (fixed) express mutability in Latin terms
    { latin: 'esto', meaning: 'let', category: 'declaration' },
    { latin: 'fixum', meaning: 'const', category: 'declaration' },
    { latin: 'functio', meaning: 'function', category: 'declaration' },
    { latin: 'novum', meaning: 'new', category: 'declaration' },
    { latin: 'importa', meaning: 'import', category: 'declaration' },
    { latin: 'exporta', meaning: 'export', category: 'declaration' },
    { latin: 'typus', meaning: 'type', category: 'declaration' },
    // WHY: "genus" (kind) for data structures - compiles to class/struct
    { latin: 'genus', meaning: 'struct/class', category: 'declaration' },
    // WHY: "pactum" (agreement) for contracts - compiles to interface/trait
    { latin: 'pactum', meaning: 'interface/trait', category: 'declaration' },
    // WHY: "ordo" (order/rank) for enumerations - named constants
    { latin: 'ordo', meaning: 'enum', category: 'declaration' },

    // ---------------------------------------------------------------------------
    // Modifiers
    // ---------------------------------------------------------------------------

    // WHY: "futura" (future/about to be) naturally expresses async operations
    { latin: 'futura', meaning: 'async', category: 'modifier' },
    // WHY: "publicus" for visibility - default is private
    { latin: 'publicus', meaning: 'public', category: 'modifier' },

    // ---------------------------------------------------------------------------
    // Operators
    // ---------------------------------------------------------------------------

    // WHY: Latin conjunctions map naturally to logical operators
    { latin: 'et', meaning: '&&', category: 'operator' },
    { latin: 'aut', meaning: '||', category: 'operator' },
    { latin: 'non', meaning: '!', category: 'operator' },

    // ---------------------------------------------------------------------------
    // Literal Values
    // ---------------------------------------------------------------------------

    { latin: 'verum', meaning: 'true', category: 'value' },
    { latin: 'falsum', meaning: 'false', category: 'value' },
    { latin: 'nihil', meaning: 'null', category: 'value' },
    // WHY: "ego" (I) for self-reference in methods - like "this" or "self"
    { latin: 'ego', meaning: 'this/self', category: 'value' },

    // ---------------------------------------------------------------------------
    // Prepositions
    // ---------------------------------------------------------------------------

    // WHY: Latin prepositions govern grammatical cases and express relationships
    { latin: 'in', meaning: 'in', category: 'preposition' },
    { latin: 'ex', meaning: 'of', category: 'preposition' },
    { latin: 'cum', meaning: 'with', category: 'preposition' },
    { latin: 'ad', meaning: 'to', category: 'preposition' },
    { latin: 'per', meaning: 'by/through', category: 'preposition' },
];

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

/**
 * Keyword lookup map for O(1) access.
 *
 * PERF: Pre-computed Map is much faster than linear array search for keywords.
 *       With 30+ keywords, this matters during tokenization of large files.
 */
const keywordMap = new Map(keywords.map(k => [k.latin, k]));

/**
 * Check if a word is a reserved keyword.
 *
 * WHY: Case-insensitive matching follows Latin convention (no case distinction
 *      in classical Latin). "SI", "Si", "si" all recognized as keywords.
 *
 * @param word - The word to check
 * @returns true if word is a keyword, false otherwise
 */
export function isKeyword(word: string): boolean {
    return keywordMap.has(word.toLowerCase());
}

/**
 * Get keyword metadata for a word.
 *
 * @param word - The word to lookup
 * @returns KeywordEntry if word is a keyword, undefined otherwise
 */
export function getKeyword(word: string): KeywordEntry | undefined {
    return keywordMap.get(word.toLowerCase());
}
