/**
 * yaml.ts - YAML Encoding/Decoding Implementation
 *
 * Native TypeScript implementation of the HAL YAML interface.
 * Uses Bun's built-in YAML support (globalThis.Bun.YAML or js-yaml fallback).
 *
 * Verb meanings:
 *   - pange (compose): serialize value to YAML string
 *   - necto (bind): bind multiple documents into multi-doc YAML
 *   - solve (untangle): parse YAML string to value
 *   - tempta (try): attempt to parse, return null on error
 *   - collige (gather): gather all documents from multi-doc YAML
 */

// Bun has built-in YAML support
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const YAML = (globalThis as any).Bun?.YAML ?? await import('js-yaml').then(m => ({
    parse: m.load,
    stringify: m.dump,
}));

export const yaml = {
    // =========================================================================
    // SERIALIZATION
    // =========================================================================

    /** Serialize value to YAML string */
    pange(valor: unknown): string {
        return YAML.stringify(valor);
    },

    /** Bind multiple documents into multi-doc YAML string */
    necto(documenta: unknown[]): string {
        return documenta.map(doc => YAML.stringify(doc)).join('---\n');
    },

    // =========================================================================
    // PARSING
    // =========================================================================

    /** Parse YAML string to value (throws on error) */
    solve(yaml: string): unknown {
        return YAML.parse(yaml);
    },

    /** Attempt to parse YAML string (returns null on error) */
    tempta(yaml: string): unknown | null {
        try {
            return YAML.parse(yaml);
        }
        catch {
            return null;
        }
    },

    /** Gather all documents from multi-doc YAML string */
    collige(yaml: string): unknown[] {
        const docs = yaml.split(/^---\s*$/m).filter(s => s.trim());
        return docs.map(doc => YAML.parse(doc));
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

    estNumerus(valor: unknown): boolean {
        return typeof valor === 'number';
    },

    estTextus(valor: unknown): boolean {
        return typeof valor === 'string';
    },

    estLista(valor: unknown): boolean {
        return Array.isArray(valor);
    },

    estTabula(valor: unknown): boolean {
        return typeof valor === 'object' && valor !== null && !Array.isArray(valor);
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
            if (typeof current === 'object' && !Array.isArray(current)) {
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
