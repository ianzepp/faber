/**
 * toml.ts - TOML Encoding/Decoding Implementation
 *
 * Native TypeScript implementation of the HAL TOML interface.
 * Uses Bun's built-in TOML.parse and smol-toml for stringify.
 *
 * Note: TOML root must be a table (object), not array or primitive.
 */

import { stringify as smolStringify } from 'smol-toml';

// Bun has built-in TOML.parse
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const BunTOML = (globalThis as any).Bun?.TOML;

function parse(toml: string): unknown {
    if (BunTOML) {
        return BunTOML.parse(toml);
    }
    // Fallback: dynamic import smol-toml parse
    throw new Error('TOML parsing requires Bun runtime');
}

function stringify(value: unknown): string {
    // smol-toml expects a Record<string, unknown>
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        throw new Error('TOML root must be a table (object)');
    }
    return smolStringify(value as Record<string, unknown>);
}

export const toml = {
    // =========================================================================
    // SERIALIZATION
    // =========================================================================

    solve(valor: unknown): string {
        return stringify(valor);
    },

    solvePulchre(valor: unknown): string {
        // smol-toml already produces pretty output by default
        return stringify(valor);
    },

    // =========================================================================
    // DESERIALIZATION
    // =========================================================================

    pange(tomlStr: string): unknown {
        return parse(tomlStr);
    },

    pangeTuto(tomlStr: string): unknown | null {
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

    estTextus(valor: unknown): boolean {
        return typeof valor === 'string';
    },

    estInteger(valor: unknown): boolean {
        return typeof valor === 'number' && Number.isInteger(valor);
    },

    estFractus(valor: unknown): boolean {
        return typeof valor === 'number' && !Number.isInteger(valor);
    },

    estBivalens(valor: unknown): boolean {
        return typeof valor === 'boolean';
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
    // TABLE ACCESS
    // =========================================================================

    cape(valor: unknown, clavis: string): unknown {
        if (typeof valor !== 'object' || valor === null) {
            return null;
        }
        // Support nested keys with dot notation
        const parts = clavis.split('.');
        let current: unknown = valor;
        for (const part of parts) {
            if (typeof current !== 'object' || current === null) {
                return null;
            }
            current = (current as Record<string, unknown>)[part];
        }
        return current ?? null;
    },
};
