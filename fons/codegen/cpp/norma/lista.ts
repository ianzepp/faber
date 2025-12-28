/**
 * Lista Method Registry - C++23 translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (C++23 target)
 *
 * ARCHITECTURE
 * ============
 * This module defines C++23 translations for lista<T> (std::vector) methods.
 * C++23 ranges provide functional-style operations that map well to Latin verbs.
 *
 * C++23 IDIOMS
 * ============
 * | Latin         | C++23                                              |
 * |---------------|----------------------------------------------------|
 * | adde          | vec.push_back(x)                                   |
 * | filtrata      | vec | views::filter(pred) | ranges::to<vector>()  |
 * | mappata       | vec | views::transform(fn) | ranges::to<vector>() |
 * | reducta       | ranges::fold_left(vec, init, fn)                   |
 * | inversa       | vec | views::reverse | ranges::to<vector>()       |
 * | ordinata      | ranges::sort(copy), return copy                    |
 *
 * LATIN VERB CONJUGATION
 * ======================
 * Latin verb forms encode mutability:
 * | Form       | Mutates | Example                    |
 * |------------|---------|----------------------------|
 * | Imperative | Yes     | adde (add!), purga (clear!)|
 * | Participle | No      | addita (having been added) |
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin method name from CallExpression
 * OUTPUT: C++23 code string
 * ERRORS: Returns undefined if method name not recognized
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Generator function type for C++ collection methods.
 *
 * WHY: The args parameter is a string[] (not a joined string) to preserve
 *      argument boundaries for multi-parameter lambdas that contain commas.
 */
export type CppGenerator = (obj: string, args: string[]) => string;

export interface ListaMethod {
    /** The Latin method name */
    latin: string;

    /** True if method mutates the vector in place */
    mutates: boolean;

    /** True if method is async (not applicable for C++) */
    async: boolean;

    /**
     * C++23 translation.
     * - string: simple method rename (obj.latin() -> obj.cpp())
     * - function: custom code generation with structured args
     */
    cpp: string | CppGenerator;

    /** Required headers for this method */
    headers?: string[];
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin array methods with C++23 translations.
 *
 * WHY: C++23 ranges provide functional operations that mirror JS/TS patterns,
 * making translation more direct than older C++ standards.
 */
export const LISTA_METHODS: Record<string, ListaMethod> = {
    // -------------------------------------------------------------------------
    // ADDING ELEMENTS
    // -------------------------------------------------------------------------

    /** Add element to end (mutates) */
    adde: {
        latin: 'adde',
        mutates: true,
        async: false,
        cpp: 'push_back',
    },

    /** Add element to end (returns new vector) */
    addita: {
        latin: 'addita',
        mutates: false,
        async: false,
        // WHY: C++ has no spread. Copy, push, return via IIFE.
        cpp: (obj, args) => `[&]{ auto v = ${obj}; v.push_back(${args.join(', ')}); return v; }()`,
    },

    /** Add element to start (mutates) */
    praepone: {
        latin: 'praepone',
        mutates: true,
        async: false,
        cpp: (obj, args) => `${obj}.insert(${obj}.begin(), ${args[0]})`,
    },

    /** Add element to start (returns new vector) */
    praeposita: {
        latin: 'praeposita',
        mutates: false,
        async: false,
        cpp: (obj, args) => `[&]{ auto v = ${obj}; v.insert(v.begin(), ${args[0]}); return v; }()`,
    },

    // -------------------------------------------------------------------------
    // REMOVING ELEMENTS
    // -------------------------------------------------------------------------

    /** Remove and return last element (mutates) */
    remove: {
        latin: 'remove',
        mutates: true,
        async: false,
        // WHY: pop_back returns void in C++, need to get value first
        cpp: (obj, _args) => `[&]{ auto v = ${obj}.back(); ${obj}.pop_back(); return v; }()`,
    },

    /** Remove last element (returns new vector without last) */
    remota: {
        latin: 'remota',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `std::vector(${obj}.begin(), ${obj}.end() - 1)`,
        headers: ['<vector>'],
    },

    /** Remove and return first element (mutates) */
    decapita: {
        latin: 'decapita',
        mutates: true,
        async: false,
        cpp: (obj, _args) => `[&]{ auto v = ${obj}.front(); ${obj}.erase(${obj}.begin()); return v; }()`,
    },

    /** Remove first element (returns new vector without first) */
    decapitata: {
        latin: 'decapitata',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `std::vector(${obj}.begin() + 1, ${obj}.end())`,
        headers: ['<vector>'],
    },

    /** Clear all elements (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        async: false,
        cpp: 'clear',
    },

    // -------------------------------------------------------------------------
    // ACCESSING ELEMENTS
    // -------------------------------------------------------------------------

    /** Get first element */
    primus: {
        latin: 'primus',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `${obj}.front()`,
    },

    /** Get last element */
    ultimus: {
        latin: 'ultimus',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `${obj}.back()`,
    },

    /** Get element at index */
    accipe: {
        latin: 'accipe',
        mutates: false,
        async: false,
        cpp: (obj, args) => `${obj}.at(${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // PROPERTIES
    // -------------------------------------------------------------------------

    /** Get length */
    longitudo: {
        latin: 'longitudo',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `${obj}.size()`,
    },

    /** Check if empty */
    vacua: {
        latin: 'vacua',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `${obj}.empty()`,
    },

    // -------------------------------------------------------------------------
    // SEARCHING
    // -------------------------------------------------------------------------

    /** Check if contains element */
    continet: {
        latin: 'continet',
        mutates: false,
        async: false,
        cpp: (obj, args) => `(std::find(${obj}.begin(), ${obj}.end(), ${args[0]}) != ${obj}.end())`,
        headers: ['<algorithm>'],
    },

    /** Find index of element (-1 if not found) */
    indiceDe: {
        latin: 'indiceDe',
        mutates: false,
        async: false,
        // WHY: C++ returns iterator, need to convert to index or -1
        cpp: (obj, args) =>
            `[&]{ auto it = std::find(${obj}.begin(), ${obj}.end(), ${args[0]}); return it != ${obj}.end() ? std::distance(${obj}.begin(), it) : -1; }()`,
        headers: ['<algorithm>'],
    },

    /** Find first element matching predicate */
    inveni: {
        latin: 'inveni',
        mutates: false,
        async: false,
        cpp: (obj, args) => `*std::find_if(${obj}.begin(), ${obj}.end(), ${args[0]})`,
        headers: ['<algorithm>'],
    },

    /** Find index of first element matching predicate */
    inveniIndicem: {
        latin: 'inveniIndicem',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            `[&]{ auto it = std::find_if(${obj}.begin(), ${obj}.end(), ${args[0]}); return it != ${obj}.end() ? std::distance(${obj}.begin(), it) : -1; }()`,
        headers: ['<algorithm>'],
    },

    // -------------------------------------------------------------------------
    // TRANSFORMATIONS (return new vectors) - C++23 Ranges
    // -------------------------------------------------------------------------

    /** Filter elements (returns new vector) */
    filtrata: {
        latin: 'filtrata',
        mutates: false,
        async: false,
        // WHY: C++23 ranges::to<vector> materializes the view
        cpp: (obj, args) => `(${obj} | std::views::filter(${args[0]}) | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Map elements (returns new vector) */
    mappata: {
        latin: 'mappata',
        mutates: false,
        async: false,
        cpp: (obj, args) => `(${obj} | std::views::transform(${args[0]}) | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Reduce to single value */
    reducta: {
        latin: 'reducta',
        mutates: false,
        async: false,
        // WHY: Faber uses (init, fn), C++23 fold_left uses (range, init, fn)
        cpp: (obj, args) => {
            if (args.length >= 2) {
                return `std::ranges::fold_left(${obj}, ${args[0]}, ${args[1]})`;
            }
            return `std::ranges::fold_left(${obj}, ${args[0]})`;
        },
        headers: ['<ranges>'],
    },

    /** Flat map (map + flatten one level) */
    explanata: {
        latin: 'explanata',
        mutates: false,
        async: false,
        // WHY: C++23 views::join flattens after transform
        cpp: (obj, args) => `(${obj} | std::views::transform(${args[0]}) | std::views::join | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Flatten one level */
    plana: {
        latin: 'plana',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `(${obj} | std::views::join | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Reverse (returns new vector) */
    inversa: {
        latin: 'inversa',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `(${obj} | std::views::reverse | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Sort (returns new vector) */
    ordinata: {
        latin: 'ordinata',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            args.length > 0
                ? `[&]{ auto v = ${obj}; std::ranges::sort(v, ${args[0]}); return v; }()`
                : `[&]{ auto v = ${obj}; std::ranges::sort(v); return v; }()`,
        headers: ['<algorithm>'],
    },

    /** Slice (returns new vector) */
    sectio: {
        latin: 'sectio',
        mutates: false,
        async: false,
        // WHY: Args are start, end - use subrange
        cpp: (obj, args) => {
            if (args.length >= 2) {
                return `std::vector(${obj}.begin() + ${args[0]}, ${obj}.begin() + ${args[1]})`;
            }
            return `std::vector(${obj}.begin() + ${args[0]}, ${obj}.end())`;
        },
        headers: ['<vector>'],
    },

    /** Take first n elements */
    prima: {
        latin: 'prima',
        mutates: false,
        async: false,
        cpp: (obj, args) => `(${obj} | std::views::take(${args[0]}) | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Take last n elements */
    ultima: {
        latin: 'ultima',
        mutates: false,
        async: false,
        // WHY: No direct views::take_last, use drop + size
        cpp: (obj, args) => `(${obj} | std::views::drop(${obj}.size() - ${args[0]}) | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Skip first n elements */
    omitte: {
        latin: 'omitte',
        mutates: false,
        async: false,
        cpp: (obj, args) => `(${obj} | std::views::drop(${args[0]}) | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    // -------------------------------------------------------------------------
    // PREDICATES
    // -------------------------------------------------------------------------

    /** Check if all elements match predicate */
    omnes: {
        latin: 'omnes',
        mutates: false,
        async: false,
        cpp: (obj, args) => `std::ranges::all_of(${obj}, ${args[0]})`,
        headers: ['<algorithm>'],
    },

    /** Check if any element matches predicate */
    aliquis: {
        latin: 'aliquis',
        mutates: false,
        async: false,
        cpp: (obj, args) => `std::ranges::any_of(${obj}, ${args[0]})`,
        headers: ['<algorithm>'],
    },

    // -------------------------------------------------------------------------
    // AGGREGATION
    // -------------------------------------------------------------------------

    /** Join elements to string */
    coniunge: {
        latin: 'coniunge',
        mutates: false,
        async: false,
        // WHY: C++ has no native join. Use ranges with format or accumulate.
        cpp: (obj, args) => {
            const sep = args.length > 0 ? args[0] : '""';
            return `[&]{ std::string r; for (size_t i = 0; i < ${obj}.size(); ++i) { if (i > 0) r += ${sep}; r += ${obj}[i]; } return r; }()`;
        },
        headers: ['<string>'],
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate with callback (no return value) */
    perambula: {
        latin: 'perambula',
        mutates: false,
        async: false,
        cpp: (obj, args) => `std::ranges::for_each(${obj}, ${args[0]})`,
        headers: ['<algorithm>'],
    },

    // -------------------------------------------------------------------------
    // MUTATING VARIANTS (in-place operations)
    // -------------------------------------------------------------------------

    /** Filter in place (mutates) */
    filtra: {
        latin: 'filtra',
        mutates: true,
        async: false,
        // WHY: erase-remove idiom with negated predicate
        cpp: (obj, args) => `${obj}.erase(std::remove_if(${obj}.begin(), ${obj}.end(), [&](auto& x) { return !(${args[0]})(x); }), ${obj}.end())`,
        headers: ['<algorithm>'],
    },

    /** Sort in place (mutates) */
    ordina: {
        latin: 'ordina',
        mutates: true,
        async: false,
        cpp: (obj, args) => (args.length > 0 ? `std::ranges::sort(${obj}, ${args[0]})` : `std::ranges::sort(${obj})`),
        headers: ['<algorithm>'],
    },

    /** Reverse in place (mutates) */
    inverte: {
        latin: 'inverte',
        mutates: true,
        async: false,
        cpp: (obj, _args) => `std::ranges::reverse(${obj})`,
        headers: ['<algorithm>'],
    },

    // -------------------------------------------------------------------------
    // LODASH-INSPIRED METHODS
    // -------------------------------------------------------------------------

    /** Group by key function -> map<K, vector<T>> */
    congrega: {
        latin: 'congrega',
        mutates: false,
        async: false,
        // WHY: C++ has no groupBy. Manual implementation.
        cpp: (obj, args) =>
            `[&]{ std::unordered_map<decltype((${args[0]})(${obj}[0])), std::vector<decltype(${obj}[0])>> m; for (auto& x : ${obj}) m[(${args[0]})(x)].push_back(x); return m; }()`,
        headers: ['<unordered_map>', '<vector>'],
    },

    /** Remove duplicates */
    unica: {
        latin: 'unica',
        mutates: false,
        async: false,
        // WHY: Convert to set and back for uniqueness
        cpp: (obj, _args) =>
            `[&]{ std::unordered_set<decltype(${obj}[0])> s(${obj}.begin(), ${obj}.end()); return std::vector(s.begin(), s.end()); }()`,
        headers: ['<unordered_set>', '<vector>'],
    },

    /** Flatten all levels - NOT FULLY SUPPORTED */
    planaOmnia: {
        latin: 'planaOmnia',
        mutates: false,
        async: false,
        // WHY: Recursive flatten requires template metaprogramming, use single level
        cpp: (obj, _args) => `(${obj} | std::views::join | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Split into chunks of size n */
    fragmenta: {
        latin: 'fragmenta',
        mutates: false,
        async: false,
        // WHY: C++23 has views::chunk
        cpp: (obj, args) => `(${obj} | std::views::chunk(${args[0]}) | std::ranges::to<std::vector<std::vector<decltype(${obj}[0])>>>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Remove falsy values (nullopt, empty, etc.) */
    densa: {
        latin: 'densa',
        mutates: false,
        async: false,
        // WHY: Filter out default/zero values
        cpp: (obj, _args) => `(${obj} | std::views::filter([](auto& x) { return static_cast<bool>(x); }) | std::ranges::to<std::vector>())`,
        headers: ['<ranges>', '<vector>'],
    },

    /** Partition by predicate -> pair<vector, vector> */
    partire: {
        latin: 'partire',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            `[&]{ std::vector<decltype(${obj}[0])> t, f; for (auto& x : ${obj}) ((${args[0]})(x) ? t : f).push_back(x); return std::make_pair(t, f); }()`,
        headers: ['<vector>', '<utility>'],
    },

    /** Shuffle (Fisher-Yates) */
    misce: {
        latin: 'misce',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `[&]{ auto v = ${obj}; std::random_device rd; std::mt19937 g(rd()); std::ranges::shuffle(v, g); return v; }()`,
        headers: ['<algorithm>', '<random>'],
    },

    /** Random element */
    specimen: {
        latin: 'specimen',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `${obj}[std::random_device{}() % ${obj}.size()]`,
        headers: ['<random>'],
    },

    /** Random n elements */
    specimina: {
        latin: 'specimina',
        mutates: false,
        async: false,
        cpp: (obj, args) =>
            `[&]{ auto v = ${obj}; std::random_device rd; std::mt19937 g(rd()); std::ranges::shuffle(v, g); v.resize(std::min(${args[0]}, v.size())); return v; }()`,
        headers: ['<algorithm>', '<random>'],
    },

    // -------------------------------------------------------------------------
    // NUMERIC AGGREGATION
    // -------------------------------------------------------------------------

    /** Sum of numbers */
    summa: {
        latin: 'summa',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `std::accumulate(${obj}.begin(), ${obj}.end(), decltype(${obj}[0]){})`,
        headers: ['<numeric>'],
    },

    /** Average of numbers */
    medium: {
        latin: 'medium',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `(std::accumulate(${obj}.begin(), ${obj}.end(), 0.0) / ${obj}.size())`,
        headers: ['<numeric>'],
    },

    /** Minimum value */
    minimus: {
        latin: 'minimus',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `*std::ranges::min_element(${obj})`,
        headers: ['<algorithm>'],
    },

    /** Maximum value */
    maximus: {
        latin: 'maximus',
        mutates: false,
        async: false,
        cpp: (obj, _args) => `*std::ranges::max_element(${obj})`,
        headers: ['<algorithm>'],
    },

    /** Minimum by key function */
    minimusPer: {
        latin: 'minimusPer',
        mutates: false,
        async: false,
        cpp: (obj, args) => `*std::ranges::min_element(${obj}, [&](auto& a, auto& b) { return (${args[0]})(a) < (${args[0]})(b); })`,
        headers: ['<algorithm>'],
    },

    /** Maximum by key function */
    maximusPer: {
        latin: 'maximusPer',
        mutates: false,
        async: false,
        cpp: (obj, args) => `*std::ranges::max_element(${obj}, [&](auto& a, auto& b) { return (${args[0]})(a) < (${args[0]})(b); })`,
        headers: ['<algorithm>'],
    },

    /** Count elements matching predicate */
    numera: {
        latin: 'numera',
        mutates: false,
        async: false,
        cpp: (obj, args) => `std::ranges::count_if(${obj}, ${args[0]})`,
        headers: ['<algorithm>'],
    },
};

// =============================================================================
// LOOKUP FUNCTIONS
// =============================================================================

/**
 * Look up a Latin method name and return its definition.
 */
export function getListaMethod(name: string): ListaMethod | undefined {
    return LISTA_METHODS[name];
}

/**
 * Check if a method name is a known lista method.
 */
export function isListaMethod(name: string): boolean {
    return name in LISTA_METHODS;
}

/**
 * Get all headers required by a method.
 */
export function getListaHeaders(name: string): string[] {
    return LISTA_METHODS[name]?.headers ?? [];
}
