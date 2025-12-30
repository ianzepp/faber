/**
 * Lista Method Registry - Zig translations for Latin array methods
 *
 * COMPILER PHASE
 * ==============
 * codegen (Zig target)
 *
 * ARCHITECTURE
 * ============
 * This module defines Zig translations for lista<T> (ArrayList) methods.
 * Zig's ArrayList requires an allocator for most operations. We use a
 * module-level arena allocator initialized in the preamble.
 *
 * ZIG SPECIFICS
 * =============
 * - lista<T> maps to std.ArrayList(T)
 * - Most mutating operations need allocator: list.append(alloc, x)
 * - Access via .items slice: list.items[0], list.items.len
 * - No functional methods (map, filter) - use ex...pro loops instead
 *
 * UNIMPLEMENTED METHODS
 * =====================
 * Functional methods (mappata, filtrata, reducta, etc.) are stubbed with
 * @compileError since Zig has no equivalent. Users should use explicit loops.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Latin method name from CallExpression
 * OUTPUT: Zig code string
 * ERRORS: Returns undefined if method name not recognized
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Generator function type for Zig collection methods.
 *
 * WHY: The curator parameter allows methods to use the correct allocator
 *      based on context (function parameter vs module-level arena).
 * WHY: The args parameter is a string[] (not a joined string) to preserve
 *      argument boundaries for multi-parameter lambdas.
 */
export type ZigGenerator = (obj: string, args: string[], curator: string) => string;

/**
 * Describes how to translate a Latin method to Zig.
 */
export interface ListaMethod {
    /** The Latin method name */
    latin: string;

    /** True if method mutates the array in place */
    mutates: boolean;

    /**
     * Zig translation.
     * - string: simple property/method access
     * - function: custom code generation with allocator context
     */
    zig: string | ZigGenerator;
}

// =============================================================================
// METHOD REGISTRY
// =============================================================================

/**
 * Registry of Latin array methods with Zig translations.
 *
 * WHY: Zig's ArrayList has different semantics than JS arrays:
 * - Requires allocator for growth operations
 * - Uses .items slice for element access
 * - No built-in functional methods
 */
export const LISTA_METHODS: Record<string, ListaMethod> = {
    // -------------------------------------------------------------------------
    // ADDING ELEMENTS
    // -------------------------------------------------------------------------

    /** Add element to end (mutates, needs allocator) */
    adde: {
        latin: 'adde',
        mutates: true,
        // WHY: ArrayList.append() can fail on OOM, we catch and panic
        zig: (obj, args, curator) => `${obj}.append(${curator}, ${args[0]}) catch @panic("OOM")`,
    },

    /** Add element to end (returns new list) - NOT IMPLEMENTED */
    addita: {
        latin: 'addita',
        mutates: false,
        zig: () => `@compileError("addita (immutable append) not implemented for Zig - use adde or explicit loop")`,
    },

    /** Add element to start (mutates) */
    praepone: {
        latin: 'praepone',
        mutates: true,
        // WHY: ArrayList.insert() at index 0 = prepend
        zig: (obj, args, curator) => `${obj}.insert(${curator}, 0, ${args[0]}) catch @panic("OOM")`,
    },

    /** Add element to start (returns new list) - NOT IMPLEMENTED */
    praeposita: {
        latin: 'praeposita',
        mutates: false,
        zig: () => `@compileError("praeposita (immutable prepend) not implemented for Zig - use praepone or explicit loop")`,
    },

    // -------------------------------------------------------------------------
    // REMOVING ELEMENTS
    // -------------------------------------------------------------------------

    /** Remove and return last element (mutates) */
    remove: {
        latin: 'remove',
        mutates: true,
        // WHY: pop() returns optional, we use .? to unwrap (panics if empty)
        zig: (obj, _args) => `${obj}.pop()`,
    },

    /** Remove last element (returns new list) - NOT IMPLEMENTED */
    remota: {
        latin: 'remota',
        mutates: false,
        zig: () => `@compileError("remota (immutable pop) not implemented for Zig - use remove or explicit loop")`,
    },

    /** Remove and return first element (mutates) */
    decapita: {
        latin: 'decapita',
        mutates: true,
        // WHY: orderedRemove(0) removes first element, preserving order
        zig: (obj, _args) => `${obj}.orderedRemove(0)`,
    },

    /** Remove first element (returns new list) - NOT IMPLEMENTED */
    decapitata: {
        latin: 'decapitata',
        mutates: false,
        zig: () => `@compileError("decapitata (immutable shift) not implemented for Zig - use decapita or explicit loop")`,
    },

    /** Clear all elements (mutates) */
    purga: {
        latin: 'purga',
        mutates: true,
        // WHY: clearRetainingCapacity keeps allocated memory for reuse
        zig: (obj, _args) => `${obj}.clearRetainingCapacity()`,
    },

    // -------------------------------------------------------------------------
    // ACCESSING ELEMENTS
    // -------------------------------------------------------------------------

    /** Get first element */
    primus: {
        latin: 'primus',
        mutates: false,
        // WHY: Direct slice access, returns optional-like behavior via bounds
        zig: (obj, _args) => `${obj}.items[0]`,
    },

    /** Get last element */
    ultimus: {
        latin: 'ultimus',
        mutates: false,
        // WHY: items.len - 1 gives last index
        zig: (obj, _args) => `${obj}.items[${obj}.items.len - 1]`,
    },

    /** Get element at index */
    accipe: {
        latin: 'accipe',
        mutates: false,
        zig: (obj, args) => `${obj}.items[${args[0]}]`,
    },

    // -------------------------------------------------------------------------
    // PROPERTIES
    // -------------------------------------------------------------------------

    /** Get length */
    longitudo: {
        latin: 'longitudo',
        mutates: false,
        zig: (obj, _args) => `${obj}.items.len`,
    },

    /** Check if empty */
    vacua: {
        latin: 'vacua',
        mutates: false,
        zig: (obj, _args) => `(${obj}.items.len == 0)`,
    },

    // -------------------------------------------------------------------------
    // SEARCHING
    // -------------------------------------------------------------------------

    /** Check if contains element - NOT FULLY IMPLEMENTED */
    continet: {
        latin: 'continet',
        mutates: false,
        // WHY: Zig has no built-in includes. Would need std.mem.indexOfScalar
        zig: (obj, args) => `(std.mem.indexOfScalar(@TypeOf(${obj}.items[0]), ${obj}.items, ${args[0]}) != null)`,
    },

    /** Find index of element */
    indiceDe: {
        latin: 'indiceDe',
        mutates: false,
        zig: (obj, args) => `std.mem.indexOfScalar(@TypeOf(${obj}.items[0]), ${obj}.items, ${args[0]})`,
    },

    // -------------------------------------------------------------------------
    // PREDICATE METHODS
    // -------------------------------------------------------------------------

    /** Check if all elements match predicate */
    omnes: {
        latin: 'omnes',
        mutates: false,
        // WHY: Inline loop since Zig has no .all() iterator method
        zig: (obj, args, _curator) => {
            return `blk: { for (${obj}.items) |v| { if (!${args[0]}(v)) break :blk false; } break :blk true; }`;
        },
    },

    /** Check if any element matches predicate */
    aliquis: {
        latin: 'aliquis',
        mutates: false,
        zig: (obj, args, _curator) => {
            return `blk: { for (${obj}.items) |v| { if (${args[0]}(v)) break :blk true; } break :blk false; }`;
        },
    },

    /** Find first element matching predicate */
    inveni: {
        latin: 'inveni',
        mutates: false,
        zig: (obj, args, _curator) => {
            return `blk: { for (${obj}.items) |v| { if (${args[0]}(v)) break :blk v; } break :blk null; }`;
        },
    },

    /** Find index of first element matching predicate */
    inveniIndicem: {
        latin: 'inveniIndicem',
        mutates: false,
        zig: (obj, args, _curator) => {
            return `blk: { for (${obj}.items, 0..) |v, i| { if (${args[0]}(v)) break :blk i; } break :blk null; }`;
        },
    },

    // -------------------------------------------------------------------------
    // FUNCTIONAL METHODS (allocating)
    // -------------------------------------------------------------------------

    /** Filter elements (returns new list) */
    filtrata: {
        latin: 'filtrata',
        mutates: false,
        zig: (obj, args, curator) => {
            // WHY: Create new ArrayList, iterate and append matching elements
            return `blk: { var result = std.ArrayList(@TypeOf(${obj}.items[0])).init(${curator}); for (${obj}.items) |v| { if (${args[0]}(v)) result.append(${curator}, v) catch @panic("OOM"); } break :blk result; }`;
        },
    },

    /** Map elements (returns new list) */
    mappata: {
        latin: 'mappata',
        mutates: false,
        zig: (obj, args, curator) => {
            // WHY: Create new ArrayList with transformed elements
            return `blk: { var result = std.ArrayList(@TypeOf(${args[0]}(${obj}.items[0]))).init(${curator}); for (${obj}.items) |v| { result.append(${curator}, ${args[0]}(v)) catch @panic("OOM"); } break :blk result; }`;
        },
    },

    /** Reduce to single value */
    reducta: {
        latin: 'reducta',
        mutates: false,
        zig: (obj, args, _curator) => {
            // args[0] = reducer fn, args[1] = initial value (optional)
            const init = args.length >= 2 ? args[1] : '0';
            return `blk: { var acc = ${init}; for (${obj}.items) |v| { acc = ${args[0]}(acc, v); } break :blk acc; }`;
        },
    },

    /** Reverse (returns new list) */
    inversa: {
        latin: 'inversa',
        mutates: false,
        zig: (obj, _args, curator) => {
            return `blk: { var result = std.ArrayList(@TypeOf(${obj}.items[0])).init(${curator}); var i: usize = ${obj}.items.len; while (i > 0) { i -= 1; result.append(${curator}, ${obj}.items[i]) catch @panic("OOM"); } break :blk result; }`;
        },
    },

    /** Sort (returns new list) */
    ordinata: {
        latin: 'ordinata',
        mutates: false,
        zig: (obj, args, curator) => {
            // Clone the items, sort them, return as new ArrayList
            const compareFn = args.length > 0 ? args[0] : 'std.sort.asc(@TypeOf(result.items[0]))';
            return `blk: { var result = ${obj}.clone() catch @panic("OOM"); std.mem.sort(@TypeOf(result.items[0]), result.items, {}, ${compareFn}); break :blk result; }`;
        },
    },

    /** Slice - take elements from start to end */
    sectio: {
        latin: 'sectio',
        mutates: false,
        zig: (obj, args, curator) => {
            if (args.length >= 2) {
                return `blk: { var result = std.ArrayList(@TypeOf(${obj}.items[0])).init(${curator}); for (${obj}.items[${args[0]}..${args[1]}]) |v| { result.append(${curator}, v) catch @panic("OOM"); } break :blk result; }`;
            }
            return `blk: { var result = std.ArrayList(@TypeOf(${obj}.items[0])).init(${curator}); for (${obj}.items[${args[0]}..]) |v| { result.append(${curator}, v) catch @panic("OOM"); } break :blk result; }`;
        },
    },

    /** Take first n elements */
    prima: {
        latin: 'prima',
        mutates: false,
        zig: (obj, args, curator) => {
            return `blk: { const n = @min(${args[0]}, ${obj}.items.len); var result = std.ArrayList(@TypeOf(${obj}.items[0])).init(${curator}); for (${obj}.items[0..n]) |v| { result.append(${curator}, v) catch @panic("OOM"); } break :blk result; }`;
        },
    },

    /** Take last n elements */
    ultima: {
        latin: 'ultima',
        mutates: false,
        zig: (obj, args, curator) => {
            return `blk: { const n = @min(${args[0]}, ${obj}.items.len); const start = ${obj}.items.len - n; var result = std.ArrayList(@TypeOf(${obj}.items[0])).init(${curator}); for (${obj}.items[start..]) |v| { result.append(${curator}, v) catch @panic("OOM"); } break :blk result; }`;
        },
    },

    /** Skip first n elements */
    omitte: {
        latin: 'omitte',
        mutates: false,
        zig: (obj, args, curator) => {
            return `blk: { const skip = @min(${args[0]}, ${obj}.items.len); var result = std.ArrayList(@TypeOf(${obj}.items[0])).init(${curator}); for (${obj}.items[skip..]) |v| { result.append(${curator}, v) catch @panic("OOM"); } break :blk result; }`;
        },
    },

    // -------------------------------------------------------------------------
    // MUTATING OPERATIONS
    // -------------------------------------------------------------------------

    /** Sort in place */
    ordina: {
        latin: 'ordina',
        mutates: true,
        zig: (obj, args, _curator) => {
            const compareFn = args.length > 0 ? args[0] : `std.sort.asc(@TypeOf(${obj}.items[0]))`;
            return `std.mem.sort(@TypeOf(${obj}.items[0]), ${obj}.items, {}, ${compareFn})`;
        },
    },

    /** Reverse in place */
    inverte: {
        latin: 'inverte',
        mutates: true,
        zig: (obj, _args, _curator) => `std.mem.reverse(@TypeOf(${obj}.items[0]), ${obj}.items)`,
    },

    // -------------------------------------------------------------------------
    // ITERATION
    // -------------------------------------------------------------------------

    /** Iterate with callback */
    perambula: {
        latin: 'perambula',
        mutates: false,
        zig: (obj, args, _curator) => {
            // WHY: Execute callback for each element using inline for loop
            return `for (${obj}.items) |v| { ${args[0]}(v); }`;
        },
    },

    /** Join elements to string */
    coniunge: {
        latin: 'coniunge',
        mutates: false,
        // WHY: Zig doesn't have a built-in join, this is complex - stub for now
        zig: () => `@compileError("coniunge not implemented for Zig - string joining requires allocator and format")`,
    },

    // -------------------------------------------------------------------------
    // AGGREGATION
    // -------------------------------------------------------------------------

    /** Sum of numeric elements */
    summa: {
        latin: 'summa',
        mutates: false,
        // WHY: Use a labeled block with inline loop since Zig has no reduce()
        zig: (obj, _args) => {
            return `blk: { var sum: i64 = 0; for (${obj}.items) |v| { sum += v; } break :blk sum; }`;
        },
    },

    /** Average of numeric elements */
    medium: {
        latin: 'medium',
        mutates: false,
        zig: (obj, _args) => {
            return `blk: { var sum: i64 = 0; for (${obj}.items) |v| { sum += v; } break :blk @as(f64, @floatFromInt(sum)) / @as(f64, @floatFromInt(${obj}.items.len)); }`;
        },
    },

    /** Minimum value */
    minimus: {
        latin: 'minimus',
        mutates: false,
        // WHY: std.mem.min returns ?T, we iterate to get the actual min
        zig: (obj, _args) => `std.mem.min(@TypeOf(${obj}.items[0]), ${obj}.items)`,
    },

    /** Maximum value */
    maximus: {
        latin: 'maximus',
        mutates: false,
        zig: (obj, _args) => `std.mem.max(@TypeOf(${obj}.items[0]), ${obj}.items)`,
    },

    /** Count elements (optionally matching predicate) */
    numera: {
        latin: 'numera',
        mutates: false,
        zig: (obj, args, _curator) => {
            if (args.length > 0) {
                // With predicate - count matching
                return `blk: { var count: usize = 0; for (${obj}.items) |v| { if (${args[0]}(v)) { count += 1; } } break :blk count; }`;
            }
            // No predicate - just length
            return `${obj}.items.len`;
        },
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
