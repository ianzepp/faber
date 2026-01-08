// caelum.ts - HTTP helper library for TypeScript target
//
// Implements the caelum stdlib functions using fetch API (client)
// and Node.js http module (server).

import * as http from 'http';

// =============================================================================
// TYPES
// =============================================================================

export interface Replicatio {
    status(): number;
    corpus(): string;
    capita(): Map<string, string>;
    bene(): boolean;
}

export interface Rogatio {
    modus(): string;
    via(): string;
    corpus(): string;
    capita(): Map<string, string>;
}

export interface Servitor {
    siste(): Promise<void>;
}

// =============================================================================
// INTERNAL HELPERS
// =============================================================================

class ReplicatioImpl implements Replicatio {
    constructor(
        private _status: number,
        private _capita: Map<string, string>,
        private _corpus: string
    ) {}

    status(): number {
        return this._status;
    }

    corpus(): string {
        return this._corpus;
    }

    capita(): Map<string, string> {
        return this._capita;
    }

    bene(): boolean {
        return this._status >= 200 && this._status < 300;
    }
}

class RogatioImpl implements Rogatio {
    constructor(
        private _modus: string,
        private _via: string,
        private _corpus: string,
        private _capita: Map<string, string>
    ) {}

    modus(): string {
        return this._modus;
    }

    via(): string {
        return this._via;
    }

    corpus(): string {
        return this._corpus;
    }

    capita(): Map<string, string> {
        return this._capita;
    }
}

class ServitorImpl implements Servitor {
    constructor(private server: http.Server) {}

    async siste(): Promise<void> {
        return new Promise((resolve, reject) => {
            this.server.close((err) => {
                if (err) reject(err);
                else resolve();
            });
        });
    }
}

async function headersToMap(headers: Headers): Promise<Map<string, string>> {
    const map = new Map<string, string>();
    headers.forEach((value, key) => map.set(key, value));
    return map;
}

// =============================================================================
// HTTP CLIENT
// =============================================================================

export async function pete(url: string): Promise<Replicatio> {
    return roga('GET', url, new Map(), '');
}

export async function mitte(url: string, corpus: string): Promise<Replicatio> {
    return roga('POST', url, new Map(), corpus);
}

export async function pone(url: string, corpus: string): Promise<Replicatio> {
    return roga('PUT', url, new Map(), corpus);
}

export async function dele(url: string): Promise<Replicatio> {
    return roga('DELETE', url, new Map(), '');
}

export async function muta(url: string, corpus: string): Promise<Replicatio> {
    return roga('PATCH', url, new Map(), corpus);
}

export async function roga(
    modus: string,
    url: string,
    capita: Map<string, string>,
    corpus: string
): Promise<Replicatio> {
    const headers: Record<string, string> = {};
    capita.forEach((value, key) => {
        headers[key] = value;
    });

    const response = await fetch(url, {
        method: modus,
        headers,
        body: corpus || undefined,
    });

    const text = await response.text();
    const responseHeaders = await headersToMap(response.headers);

    return new ReplicatioImpl(response.status, responseHeaders, text);
}

// =============================================================================
// HTTP SERVER
// =============================================================================

export async function exspecta(
    handler: (rogatio: Rogatio) => Replicatio,
    portus: number
): Promise<Servitor> {
    return new Promise((resolve) => {
        const server = http.createServer(async (req, res) => {
            // Collect request body
            const chunks: Buffer[] = [];
            for await (const chunk of req) {
                chunks.push(chunk);
            }
            const body = Buffer.concat(chunks).toString('utf-8');

            // Build Rogatio
            const capita = new Map<string, string>();
            for (const [key, value] of Object.entries(req.headers)) {
                if (typeof value === 'string') {
                    capita.set(key, value);
                }
                else if (Array.isArray(value)) {
                    capita.set(key, value.join(', '));
                }
            }

            const rogatio = new RogatioImpl(
                req.method || 'GET',
                req.url || '/',
                body,
                capita
            );

            // Call handler
            const replicatio = handler(rogatio);

            // Send response
            replicatio.capita().forEach((value, key) => {
                res.setHeader(key, value);
            });
            res.statusCode = replicatio.status();
            res.end(replicatio.corpus());
        });

        server.listen(portus, () => {
            resolve(new ServitorImpl(server));
        });
    });
}

// =============================================================================
// RESPONSE BUILDER
// =============================================================================

export function replicatio(
    status: number,
    capita: Map<string, string>,
    corpus: string
): Replicatio {
    return new ReplicatioImpl(status, capita, corpus);
}
