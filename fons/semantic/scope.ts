/**
 * Scope Management - Symbol Tables and Lexical Scoping
 *
 * COMPILER PHASE
 * ==============
 * semantic
 *
 * ARCHITECTURE
 * ============
 * This module implements lexical scoping with a chain of symbol tables. Each
 * scope contains bindings from names to their semantic information (type,
 * mutability, etc.). Scopes form a parent-child chain representing block nesting.
 *
 * The symbol table tracks:
 * - Variables (varia/fixum declarations)
 * - Functions (functio declarations)
 * - Parameters (within function scope)
 *
 * Lookup walks up the scope chain until a binding is found or we reach global scope.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  AST nodes during semantic analysis
 * OUTPUT: Symbol bindings with resolved types
 * ERRORS: Reported via SemanticError for undefined/duplicate symbols
 *
 * @module semantic/scope
 */

import type { SemanticType } from './types';
import type { Position } from '../tokenizer/types';

// =============================================================================
// SYMBOL DEFINITIONS
// =============================================================================

/**
 * A symbol binding in the symbol table.
 *
 * WHY: Extended to support type aliases and structured types.
 *      - 'type' for typus (type alias) declarations
 *      - 'enum' for ordo (enum) declarations
 *      - 'genus' for genus (class/struct) declarations
 *      - 'pactum' for pactum (interface) declarations
 */
export interface Symbol {
    name: string;
    type: SemanticType;
    kind: 'variable' | 'function' | 'parameter' | 'type' | 'enum' | 'genus' | 'pactum';
    mutable: boolean;
    position: Position;
}

/**
 * A lexical scope containing symbol bindings.
 */
export interface Scope {
    symbols: Map<string, Symbol>;
    parent: Scope | null;
    kind: 'global' | 'function' | 'block';
}

// =============================================================================
// SCOPE MANAGEMENT
// =============================================================================

/**
 * Create the global scope.
 */
export function createGlobalScope(): Scope {
    return {
        symbols: new Map(),
        parent: null,
        kind: 'global',
    };
}

/**
 * Create a child scope.
 */
export function createScope(parent: Scope, kind: Scope['kind'] = 'block'): Scope {
    return {
        symbols: new Map(),
        parent,
        kind,
    };
}

/**
 * Define a symbol in the current scope.
 *
 * @returns Error message if symbol already defined in this scope, null otherwise
 */
export function defineSymbol(scope: Scope, symbol: Symbol): string | null {
    if (scope.symbols.has(symbol.name)) {
        const existing = scope.symbols.get(symbol.name)!;

        return `'${symbol.name}' is already defined at line ${existing.position.line}`;
    }

    scope.symbols.set(symbol.name, symbol);

    return null;
}

/**
 * Look up a symbol by name, walking up the scope chain.
 *
 * @returns The symbol if found, null otherwise
 */
export function lookupSymbol(scope: Scope, name: string): Symbol | null {
    const symbol = scope.symbols.get(name);

    if (symbol) {
        return symbol;
    }

    if (scope.parent) {
        return lookupSymbol(scope.parent, name);
    }

    return null;
}

/**
 * Look up a symbol only in the current scope (no parent traversal).
 *
 * WHY: Used to check for redefinition in the same scope.
 */
export function lookupSymbolLocal(scope: Scope, name: string): Symbol | null {
    return scope.symbols.get(name) ?? null;
}

/**
 * Update an existing symbol's type in the current scope.
 *
 * WHY: Two-pass analysis predeclares symbols with placeholder types,
 *      then refines them with real types in a second pass.
 *
 * @returns true if symbol was updated, false if not found
 */
export function updateSymbolType(scope: Scope, name: string, type: SemanticType): boolean {
    const symbol = scope.symbols.get(name);

    if (symbol) {
        symbol.type = type;

        return true;
    }

    return false;
}

/**
 * Find the enclosing function scope.
 *
 * WHY: Needed for return type checking.
 */
export function findFunctionScope(scope: Scope): Scope | null {
    if (scope.kind === 'function') {
        return scope;
    }

    if (scope.parent) {
        return findFunctionScope(scope.parent);
    }

    return null;
}
