/**
 * yaml.ts - YAML Encoding/Decoding Implementation
 *
 * Native TypeScript implementation of the HAL YAML interface.
 * Uses Bun's built-in YAML support (globalThis.Bun.YAML or js-yaml fallback).
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

    solve(valor: unknown): string {
        return YAML.stringify(valor);
    },

    solveMulti(documenta: unknown[]): string {
        return documenta.map(doc => YAML.stringify(doc)).join('---\n');
    },

    // =========================================================================
    // DESERIALIZATION
    // =========================================================================

    pange(yaml: string): unknown {
        return YAML.parse(yaml);
    },

    pangeTuto(yaml: string): unknown | null {
        try {
            return YAML.parse(yaml);
        }
        catch {
            return null;
        }
    },

    pangeMulti(yaml: string): unknown[] {
        // Split on document separators and parse each
        const docs = yaml.split(/^---\s*$/m).filter(s => s.trim());
        return docs.map(doc => YAML.parse(doc));
    },

    // =========================================================================
    // TYPE CHECKING (same as JSON)
    // =========================================================================

    estNihil(valor: unknown): boolean {
        return valor === null || valor === undefined;
    },

    estTextus(valor: unknown): boolean {
        return typeof valor === 'string';
    },

    estNumerus(valor: unknown): boolean {
        return typeof valor === 'number';
    },

    estLista(valor: unknown): boolean {
        return Array.isArray(valor);
    },

    estTabula(valor: unknown): boolean {
        return typeof valor === 'object' && valor !== null && !Array.isArray(valor);
    },
};
