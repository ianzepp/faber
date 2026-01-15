/**
 * nomenclator.ts - DNS Resolution Implementation
 *
 * Etymology: "nomenclator" - name-caller. In Rome, a slave who announced
 *            visitors' names. DNS resolves names to addresses.
 *
 * Native TypeScript implementation of the HAL DNS interface.
 */

import dns from 'node:dns/promises';

/**
 * MX Record interface
 */
export interface RecordumMx {
    hospes(): string;
    prioritas(): number;
}

/**
 * Create an MX record from dns.resolveMx result
 */
function createRecordumMx(exchange: string, priority: number): RecordumMx {
    return {
        hospes: () => exchange,
        prioritas: () => priority,
    };
}

export const nomenclator = {
    // =========================================================================
    // FORWARD LOOKUP
    // =========================================================================

    /**
     * Resolve hostname to IP addresses (IPv4 and IPv6)
     */
    async resolve(hospes: string): Promise<string[]> {
        const results: string[] = [];

        // Try IPv4
        try {
            const ipv4 = await dns.resolve4(hospes);
            results.push(...ipv4);
        }
        catch {
            // IPv4 resolution failed, continue
        }

        // Try IPv6
        try {
            const ipv6 = await dns.resolve6(hospes);
            results.push(...ipv6);
        }
        catch {
            // IPv6 resolution failed, continue
        }

        return results;
    },

    /**
     * Resolve hostname to IPv4 addresses only
     */
    async resolve4(hospes: string): Promise<string[]> {
        try {
            return await dns.resolve4(hospes);
        }
        catch {
            return [];
        }
    },

    /**
     * Resolve hostname to IPv6 addresses only
     */
    async resolve6(hospes: string): Promise<string[]> {
        try {
            return await dns.resolve6(hospes);
        }
        catch {
            return [];
        }
    },

    // =========================================================================
    // REVERSE LOOKUP
    // =========================================================================

    /**
     * Reverse lookup: IP address to hostnames
     */
    async reversa(ip: string): Promise<string[]> {
        try {
            return await dns.reverse(ip);
        }
        catch {
            return [];
        }
    },

    // =========================================================================
    // RECORD QUERIES
    // =========================================================================

    /**
     * Query MX records (mail servers)
     */
    async mx(dominium: string): Promise<RecordumMx[]> {
        try {
            const records = await dns.resolveMx(dominium);
            return records.map((r) => createRecordumMx(r.exchange, r.priority));
        }
        catch {
            return [];
        }
    },

    /**
     * Query TXT records
     */
    async txt(dominium: string): Promise<string[]> {
        try {
            const records = await dns.resolveTxt(dominium);
            // resolveTxt returns string[][] (each TXT record can have multiple strings)
            return records.map((chunks) => chunks.join(''));
        }
        catch {
            return [];
        }
    },

    /**
     * Query NS records (name servers)
     */
    async ns(dominium: string): Promise<string[]> {
        try {
            return await dns.resolveNs(dominium);
        }
        catch {
            return [];
        }
    },

    /**
     * Query CNAME record
     */
    async cname(dominium: string): Promise<string | null> {
        try {
            const results = await dns.resolveCname(dominium);
            return results.length > 0 ? results[0] : null;
        }
        catch {
            return null;
        }
    },
};
