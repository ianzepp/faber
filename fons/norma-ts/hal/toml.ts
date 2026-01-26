/**
 * toml.ts - TOML Encoding/Decoding Implementation
 *
 * Native TypeScript implementation of the HAL TOML interface.
 * Uses Bun's built-in TOML.parse and smol-toml for stringify.
 *
 * Note: TOML root must be a table (object), not array or primitive.
 *
 * Verb meanings:
 *   - pange (compose): serialize table to TOML string
 *   - solve (untangle): parse TOML string to table
 *   - tempta (try): attempt to parse, return null on error
 */

import { stringify as smolStringify } from 'smol-toml';

// Bun has built-in TOML.parse
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const BunTOML = (globalThis as any).Bun?.TOML;

function parse(toml: string): unknown {
    if (BunTOML) {
        return BunTOML.parse(toml);
    }
    throw new Error('TOML parsing requires Bun runtime');
}

function stringify(value: unknown): string {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        throw new Error('TOML root must be a table (object)');
    }
    return smolStringify(value as Record<string, unknown>);
}

export const toml = {
    // =========================================================================
    // SERIALIZATION
    // =========================================================================

    /** Serialize table to TOML string */
    pange(valor: unknown): string {
        return stringify(valor);
    },

    // =========================================================================
    // PARSING
    // =========================================================================

    /** Parse TOML string to table (throws on error) */
    solve(tomlStr: string): unknown {
        return parse(tomlStr);
    },

    /** Attempt to parse TOML string (returns null on error) */
    tempta(tomlStr: string): unknown | null {
        try {
            return parse(tomlStr);
        }
        catch {
            return null;
        }
    },

    // =========================================================================
    // TYPE CHECKING
    // =========================================================================

    estNihil(valor: unknown): boolean {
        return valor === null || valor === undefined;
    },

    estBivalens(valor: unknown): boolean {
        return typeof valor === 'boolean';
    },

    estTextus(valor: unknown): boolean {
        return typeof valor === 'string';
    },

    estInteger(valor: unknown): boolean {
        return typeof valor === 'number' && Number.isInteger(valor);
    },

    estFractus(valor: unknown): boolean {
        return typeof valor === 'number' && !Number.isInteger(valor);
    },

    estTempus(valor: unknown): boolean {
        return valor instanceof Date;
    },

    estLista(valor: unknown): boolean {
        return Array.isArray(valor);
    },

    estTabula(valor: unknown): boolean {
        return typeof valor === 'object' && valor !== null && !Array.isArray(valor) && !(valor instanceof Date);
    },

    // =========================================================================
    // VALUE EXTRACTION
    // =========================================================================

    utTextus(valor: unknown, defVal: string): string {
        return typeof valor === 'string' ? valor : defVal;
    },

    utNumerus(valor: unknown, defVal: number): number {
        return typeof valor === 'number' ? valor : defVal;
    },

    utBivalens(valor: unknown, defVal: boolean): boolean {
        return typeof valor === 'boolean' ? valor : defVal;
    },

    // =========================================================================
    // VALUE ACCESS
    // =========================================================================

    /** Get value by key (returns null if missing) */
    cape(valor: unknown, clavis: string): unknown {
        if (typeof valor === 'object' && valor !== null && !Array.isArray(valor)) {
            return (valor as Record<string, unknown>)[clavis] ?? null;
        }
        return null;
    },

    /** Pluck value by array index (returns null if out of bounds) */
    carpe(valor: unknown, index: number): unknown {
        if (Array.isArray(valor) && index >= 0 && index < valor.length) {
            return valor[index];
        }
        return null;
    },

    /** Find value by dotted path (returns null if not found) */
    inveni(valor: unknown, via: string): unknown {
        const parts = via.split('.');
        let current: unknown = valor;

        for (const part of parts) {
            if (current === null || current === undefined) {
                return null;
            }
            if (typeof current === 'object' && !Array.isArray(current) && !(current instanceof Date)) {
                current = (current as Record<string, unknown>)[part];
            }
            else if (Array.isArray(current)) {
                const idx = parseInt(part, 10);
                if (isNaN(idx) || idx < 0 || idx >= current.length) {
                    return null;
                }
                current = current[idx];
            }
            else {
                return null;
            }
        }

        return current ?? null;
    },
};
