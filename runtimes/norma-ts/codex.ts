/**
 * codex.ts - Encoding/Decoding Implementation
 *
 * Native TypeScript implementation of encoding operations.
 */

export const codex = {
    // =========================================================================
    // BASE64
    // =========================================================================

    base64(data: Uint8Array): string {
        return Buffer.from(data).toString('base64');
    },

    deBase64(data: string): Uint8Array {
        return new Uint8Array(Buffer.from(data, 'base64'));
    },

    temptaBase64(data: string): Uint8Array | null {
        try {
            // Check for valid base64 characters
            if (!/^[A-Za-z0-9+/]*={0,2}$/.test(data)) {
                return null;
            }
            return new Uint8Array(Buffer.from(data, 'base64'));
        } catch {
            return null;
        }
    },

    // =========================================================================
    // HEX
    // =========================================================================

    hex(data: Uint8Array): string {
        return Buffer.from(data).toString('hex');
    },

    deHex(data: string): Uint8Array {
        if (data.length % 2 !== 0) {
            throw new Error('Invalid hex string: odd length');
        }
        if (!/^[0-9a-fA-F]*$/.test(data)) {
            throw new Error('Invalid hex string: non-hex characters');
        }
        return new Uint8Array(Buffer.from(data, 'hex'));
    },

    temptaHex(data: string): Uint8Array | null {
        try {
            if (data.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(data)) {
                return null;
            }
            return new Uint8Array(Buffer.from(data, 'hex'));
        } catch {
            return null;
        }
    },

    // =========================================================================
    // URL ENCODING
    // =========================================================================

    url(data: string): string {
        return encodeURI(data);
    },

    deUrl(data: string): string {
        return decodeURI(data);
    },

    urlComponentum(data: string): string {
        return encodeURIComponent(data);
    },

    deUrlComponentum(data: string): string {
        return decodeURIComponent(data);
    },
};
