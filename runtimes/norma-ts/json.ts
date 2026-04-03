/**
 * json.ts - JSON Encoding/Decoding Implementation
 *
 * Native TypeScript implementation of the HAL JSON interface.
 *
 * Verb meanings:
 *   - pange (compose): serialize value to JSON string
 *   - solve (untangle): parse JSON string to value
 *   - tempta (try): attempt to parse, return null on error
 */

export const json = {
    // =========================================================================
    // SERIALIZATION
    // =========================================================================

    /** Serialize value to JSON string (indentum > 0 for pretty-print) */
    pange(valor: unknown, indentum?: number): string {
        if (indentum !== undefined && indentum > 0) {
            return JSON.stringify(valor, null, indentum);
        }
        return JSON.stringify(valor);
    },

    // =========================================================================
    // PARSING
    // =========================================================================

    /** Parse JSON string to value (throws on error) */
    solve(json: string): unknown {
        return JSON.parse(json);
    },

    /** Attempt to parse JSON string (returns null on error) */
    tempta(json: string): unknown | null {
        try {
            return JSON.parse(json);
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
