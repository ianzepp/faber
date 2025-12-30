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
 * Keywords are case-sensitive and lowercase only. "si" is the if keyword,
 * "SI" is a valid constant identifier.
 *
 * Keywords are categorized by their syntactic role:
 * - control: Control flow statements (si, dum, redde)
 * - declaration: Variable/function declarations (varia, functio)
 * - operator: Logical operators (et, aut, non)
 * - value: Literal values (verum, falsum, nihil)
 * - preposition: Case-governing prepositions (in, ex, de, ad)
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
 * INV-2: Keyword lookup is case-sensitive (exact match required)
 * INV-3: Keywords are disjoint from type names
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
    category: 'control' | 'declaration' | 'operator' | 'value' | 'preposition' | 'modifier' | 'dsl';
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
    // Literal: si / aliter si / aliter
    // Poetic:  si / sin / secus
    // Ternary: condition sic truthy secus falsy (or ? :)
    { latin: 'si', meaning: 'if', category: 'control' },
    { latin: 'aliter', meaning: 'else', category: 'control' },
    { latin: 'sin', meaning: 'else if', category: 'control' },
    { latin: 'secus', meaning: 'else/:', category: 'control' },
    { latin: 'dum', meaning: 'while', category: 'control' },
    { latin: 'fac', meaning: 'do', category: 'control' },
    { latin: 'pro', meaning: 'for', category: 'control' },
    { latin: 'elige', meaning: 'switch', category: 'control' },
    { latin: 'ergo', meaning: 'then', category: 'control' },
    { latin: 'nulla', meaning: 'none/empty', category: 'operator' },
    { latin: 'nonnulla', meaning: 'some/non-empty', category: 'operator' },
    { latin: 'nihil', meaning: 'is null', category: 'operator' },
    { latin: 'nonnihil', meaning: 'is not null', category: 'operator' },
    { latin: 'negativum', meaning: '< 0', category: 'operator' },
    { latin: 'positivum', meaning: '> 0', category: 'operator' },
    { latin: 'rumpe', meaning: 'break', category: 'control' },
    { latin: 'perge', meaning: 'continue', category: 'control' },
    { latin: 'redde', meaning: 'return', category: 'control' },
    { latin: 'custodi', meaning: 'guard', category: 'control' },
    { latin: 'adfirma', meaning: 'assert', category: 'control' },

    // ---------------------------------------------------------------------------
    // Testing
    // ---------------------------------------------------------------------------

    // WHY: "probandum" (gerundive of probare) = "that which must be tested"
    //      Used for test suite declarations
    { latin: 'probandum', meaning: 'describe', category: 'control' },
    // WHY: "proba" (imperative of probare) = "test!" / "prove!"
    //      Used for individual test cases
    { latin: 'proba', meaning: 'test', category: 'control' },
    // WHY: "omitte" (imperative of omittere) = "skip!" / "omit!"
    //      Modifier for skipped tests
    { latin: 'omitte', meaning: 'skip', category: 'modifier' },
    // WHY: "futurum" (neuter noun) = "the future" / "pending"
    //      Modifier for todo/pending tests
    { latin: 'futurum', meaning: 'todo', category: 'modifier' },
    // WHY: "omnia" (neuter plural) = "all things"
    //      Used with cura ante/post for beforeAll/afterAll semantics
    { latin: 'omnia', meaning: 'all', category: 'modifier' },
    // WHY: "cura" (care, concern) for resource management
    //      Used for test setup/teardown: cura ante { }, cura post { }
    //      Also for scoped resources: cura aperi "file" fit fd { }
    { latin: 'cura', meaning: 'care', category: 'control' },
    // WHY: Curator kinds for cura statements - explicit resource type
    //      arena = arena allocator (sand, open space)
    //      page = page allocator (pagina)
    { latin: 'arena', meaning: 'arena', category: 'control' },
    { latin: 'page', meaning: 'page', category: 'control' },
    { latin: 'tempta', meaning: 'try', category: 'control' },
    { latin: 'cape', meaning: 'catch', category: 'control' },
    { latin: 'demum', meaning: 'finally', category: 'control' },
    { latin: 'iace', meaning: 'throw', category: 'control' },
    { latin: 'mori', meaning: 'panic', category: 'control' },
    { latin: 'scribe', meaning: 'print', category: 'control' },
    { latin: 'vide', meaning: 'debug', category: 'control' },
    { latin: 'mone', meaning: 'warn', category: 'control' },
    // WHY: "scriptum" (that which has been written) for string formatting
    //      Perfect passive participle of scribere - the noun form of the action
    //      scribe = write to output, scriptum = create a formatted string
    { latin: 'scriptum', meaning: 'format string', category: 'control' },
    { latin: 'cede', meaning: 'await', category: 'control' },

    // ---------------------------------------------------------------------------
    // Declarations
    // ---------------------------------------------------------------------------

    // WHY: "varia" (be!) and "fixum" (fixed) express mutability in Latin terms
    { latin: 'varia', meaning: 'let', category: 'declaration' },
    { latin: 'fixum', meaning: 'const', category: 'declaration' },
    // WHY: "nexum" (bound) for reactive fields - changes trigger re-render
    { latin: 'nexum', meaning: 'reactive', category: 'modifier' },
    // WHY: Gerundive forms for async bindings — "that which will be fixed/varied"
    //      Implies await without explicit cede keyword
    { latin: 'figendum', meaning: 'const await', category: 'declaration' },
    { latin: 'variandum', meaning: 'let await', category: 'declaration' },
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
    // WHY: "discretio" (distinction) for tagged unions - discriminated variants
    { latin: 'discretio', meaning: 'tagged union', category: 'declaration' },
    // WHY: "discerne" (distinguish!) for variant matching - pairs with discretio
    //      Imperative of discernere (to separate, distinguish)
    { latin: 'discerne', meaning: 'match variant', category: 'control' },

    // ---------------------------------------------------------------------------
    // Modifiers
    // ---------------------------------------------------------------------------

    // WHY: "futura" (future/about to be) naturally expresses async operations
    { latin: 'futura', meaning: 'async', category: 'modifier' },
    // WHY: "prae" (before) for compile-time type parameters
    //      Short form used in function parameters: prae typus T
    { latin: 'prae', meaning: 'comptime', category: 'modifier' },
    // WHY: "praefixum" (pre-fixed, past participle of praefigere) for compile-time blocks
    //      Extends fixum vocabulary: "fixed" → "pre-fixed"
    //      Used for compile-time evaluation: praefixum { ... } or praefixum(expr)
    { latin: 'praefixum', meaning: 'comptime block', category: 'modifier' },
    // WHY: "cursor" (runner) for generator/iterator functions
    { latin: 'cursor', meaning: 'generator', category: 'modifier' },
    // WHY: "publicus/privatus" for visibility - default is public (struct semantics)
    { latin: 'publicus', meaning: 'public', category: 'modifier' },
    { latin: 'privatus', meaning: 'private', category: 'modifier' },
    // WHY: "generis" (of the genus) for type-level/static members
    { latin: 'generis', meaning: 'static', category: 'modifier' },
    // WHY: "implet" (fulfills) for implementing interfaces
    { latin: 'implet', meaning: 'implements', category: 'modifier' },
    // WHY: "sub" (under) for class inheritance - child is "under" parent
    //      Target-gated: TS/Py/C++ only; error on Rust/Zig (no class inheritance)
    { latin: 'sub', meaning: 'extends', category: 'modifier' },
    // WHY: "abstractus" for abstract classes and methods
    //      Target-gated: TS/Py/C++ only; error on Rust/Zig
    { latin: 'abstractus', meaning: 'abstract', category: 'modifier' },
    // WHY: "protectus" for protected visibility (between private and public)
    //      Target-gated: TS/Py/C++ only; error on Rust/Zig
    { latin: 'protectus', meaning: 'protected', category: 'modifier' },

    // ---------------------------------------------------------------------------
    // Operators
    // ---------------------------------------------------------------------------

    // WHY: Latin conjunctions map naturally to logical operators
    { latin: 'et', meaning: '&&', category: 'operator' },
    { latin: 'aut', meaning: '||', category: 'operator' },
    { latin: 'non', meaning: '!', category: 'operator' },
    // WHY: "est" (is) for strict equality - natural Latin copula
    { latin: 'est', meaning: '===', category: 'operator' },
    // WHY: "sic" (thus) and "secus" (otherwise) for ternary expressions
    // sic/secus are the Latin equivalent of ?/:
    { latin: 'sic', meaning: '?', category: 'operator' },
    // WHY: "fieri" conjugation encodes return semantics:
    //   fit   = becomes (sync, single return)
    //   fiet  = will become (async, single return)
    //   fiunt = become [plural] (sync, yields many)
    //   fient = will become [plural] (async, yields many)
    { latin: 'fit', meaning: '->', category: 'operator' },
    { latin: 'fiet', meaning: 'async ->', category: 'operator' },
    { latin: 'fiunt', meaning: 'yields ->', category: 'operator' },
    { latin: 'fient', meaning: 'async yields ->', category: 'operator' },

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
    // WHY: 'de' and 'in' also encode ownership semantics for systems targets (Rust/Zig):
    //      de = borrowed/read-only (&T, []const u8)
    //      in = mutable borrow (&mut T, *T)
    { latin: 'de', meaning: 'from/concerning', category: 'preposition' },
    { latin: 'in', meaning: 'in/into', category: 'preposition' },
    { latin: 'ex', meaning: 'of', category: 'preposition' },
    { latin: 'ad', meaning: 'to', category: 'preposition' },
    { latin: 'per', meaning: 'by/through', category: 'preposition' },
    // WHY: "qua" (as, in the capacity of) for type assertions
    { latin: 'qua', meaning: 'as (type)', category: 'preposition' },
    // WHY: "ut" (as) for aliasing in imports and destructuring
    //      ex norma importa scribe ut s
    //      ex persona fixum nomen ut n
    { latin: 'ut', meaning: 'as (alias)', category: 'preposition' },
    // WHY: "vel" (or) for default values in destructuring
    { latin: 'vel', meaning: 'or', category: 'operator' },
    // WHY: "ante" (before) and "usque" (up to) for range operators
    //      ante = exclusive (0 ante 10 = 0-9)
    //      usque = inclusive (0 usque 10 = 0-10)
    // NOTE: "ante" is also used in test setup blocks (cura ante { })
    { latin: 'ante', meaning: 'before (exclusive)', category: 'operator' },
    { latin: 'usque', meaning: 'up to (inclusive)', category: 'operator' },

    // ---------------------------------------------------------------------------
    // Spread/Rest Operators
    // ---------------------------------------------------------------------------

    // WHY: "sparge" (scatter/spread) for spreading elements in arrays/objects/calls
    //      Latin spargere = "to scatter, spread, sprinkle"
    { latin: 'sparge', meaning: '...', category: 'operator' },
    // WHY: "ceteri" (the rest/others) for collecting remaining elements
    //      Latin ceteri = "the rest, the others, the remaining"
    { latin: 'ceteri', meaning: '...rest', category: 'operator' },

    // ---------------------------------------------------------------------------
    // Collection DSL
    // ---------------------------------------------------------------------------

    // WHY: 'ab' (away from) is the DSL entry point for collection filtering
    //      All filtering uses ab; ex remains unchanged for iteration/import/destructuring
    //      Include/exclude is handled via 'non' keyword for symmetric negation
    { latin: 'ab', meaning: 'filter from', category: 'dsl' },
    // WHY: 'ubi' (where) introduces filter conditions in ab expressions
    { latin: 'ubi', meaning: 'where', category: 'dsl' },
    // WHY: DSL verbs for collection transforms after filtering
    //      These provide concise syntax for common collection operations
    { latin: 'prima', meaning: 'first n', category: 'dsl' },
    { latin: 'ultima', meaning: 'last n', category: 'dsl' },
    { latin: 'summa', meaning: 'sum', category: 'dsl' },
    { latin: 'ordina', meaning: 'sort', category: 'dsl' },
    { latin: 'collige', meaning: 'pluck', category: 'dsl' },
    { latin: 'grupa', meaning: 'group by', category: 'dsl' },

    // ---------------------------------------------------------------------------
    // Regex DSL
    // ---------------------------------------------------------------------------

    // WHY: "sed" (the Unix stream editor) is synonymous with pattern matching.
    //      The Latin word "sed" means "but" — semantically unrelated, but the
    //      Unix association makes it instantly recognizable for regex patterns.
    { latin: 'sed', meaning: 'regex', category: 'dsl' },
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
 * WHY: Case-sensitive matching — "si" is a keyword, "SI" is a valid identifier.
 *
 * @param word - The word to check
 * @returns true if word is a keyword, false otherwise
 */
export function isKeyword(word: string): boolean {
    return keywordMap.has(word);
}

/**
 * Get keyword metadata for a word.
 *
 * @param word - The word to lookup
 * @returns KeywordEntry if word is a keyword, undefined otherwise
 */
export function getKeyword(word: string): KeywordEntry | undefined {
    return keywordMap.get(word);
}
