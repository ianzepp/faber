/**
 * http.ts - HTTP Client/Server Implementation
 *
 * Native TypeScript implementation of the HAL HTTP interface.
 * Uses Bun's built-in fetch and Bun.serve().
 */

// =============================================================================
// RESPONSE CLASS
// =============================================================================

export class Replicatio {
    private _status: number;
    private _body: string | Uint8Array;
    private _headers: Record<string, string>;

    constructor(status: number, headers: Record<string, string>, body: string | Uint8Array) {
        this._status = status;
        this._headers = headers;
        this._body = body;
    }

    status(): number {
        return this._status;
    }

    corpus(): string {
        if (typeof this._body === 'string') {
            return this._body;
        }
        return new TextDecoder().decode(this._body);
    }

    corpusOcteti(): Uint8Array {
        if (this._body instanceof Uint8Array) {
            return this._body;
        }
        return new TextEncoder().encode(this._body);
    }

    corpusJson(): unknown {
        return JSON.parse(this.corpus());
    }

    capita(): Record<string, string> {
        return { ...this._headers };
    }

    caput(nomen: string): string | null {
        // Headers are case-insensitive, normalize to lowercase for lookup
        const lower = nomen.toLowerCase();
        for (const [key, value] of Object.entries(this._headers)) {
            if (key.toLowerCase() === lower) {
                return value;
            }
        }
        return null;
    }

    bene(): boolean {
        return this._status >= 200 && this._status < 300;
    }

    // Internal: convert to Bun Response for server use
    _toResponse(): Response {
        return new Response(this._body, {
            status: this._status,
            headers: this._headers,
        });
    }
}

// =============================================================================
// REQUEST CLASS (for server handlers)
// =============================================================================

export class Rogatio {
    private _method: string;
    private _url: URL;
    private _body: string;
    private _headers: Record<string, string>;

    constructor(method: string, url: URL, headers: Record<string, string>, body: string) {
        this._method = method;
        this._url = url;
        this._headers = headers;
        this._body = body;
    }

    modus(): string {
        return this._method;
    }

    via(): string {
        return this._url.pathname;
    }

    corpus(): string {
        return this._body;
    }

    corpusJson(): unknown {
        return JSON.parse(this._body);
    }

    capita(): Record<string, string> {
        return { ...this._headers };
    }

    caput(nomen: string): string | null {
        const lower = nomen.toLowerCase();
        for (const [key, value] of Object.entries(this._headers)) {
            if (key.toLowerCase() === lower) {
                return value;
            }
        }
        return null;
    }

    param(nomen: string): string | null {
        return this._url.searchParams.get(nomen);
    }

    // Internal: create from Bun Request
    static async _fromRequest(request: Request): Promise<Rogatio> {
        const headers: Record<string, string> = {};
        request.headers.forEach((value, key) => {
            headers[key] = value;
        });

        const body = await request.text();
        return new Rogatio(request.method, new URL(request.url), headers, body);
    }
}

// =============================================================================
// SERVER CLASS
// =============================================================================

export class Servitor {
    private _server: ReturnType<typeof Bun.serve>;

    constructor(server: ReturnType<typeof Bun.serve>) {
        this._server = server;
    }

    siste(): void {
        this._server.stop();
    }

    portus(): number {
        return this._server.port;
    }
}

// =============================================================================
// MAIN MODULE
// =============================================================================

type Handler = (rogatio: Rogatio) => Replicatio | Promise<Replicatio>;

async function responseFromFetch(response: Response): Promise<Replicatio> {
    const headers: Record<string, string> = {};
    response.headers.forEach((value, key) => {
        headers[key] = value;
    });
    const body = await response.text();
    return new Replicatio(response.status, headers, body);
}

export const http = {
    // =========================================================================
    // HTTP CLIENT - Simple Methods
    // =========================================================================

    async petet(url: string): Promise<Replicatio> {
        const response = await fetch(url, { method: 'GET' });
        return responseFromFetch(response);
    },

    async mittet(url: string, corpus: string): Promise<Replicatio> {
        const response = await fetch(url, {
            method: 'POST',
            body: corpus,
        });
        return responseFromFetch(response);
    },

    async ponet(url: string, corpus: string): Promise<Replicatio> {
        const response = await fetch(url, {
            method: 'PUT',
            body: corpus,
        });
        return responseFromFetch(response);
    },

    async delet(url: string): Promise<Replicatio> {
        const response = await fetch(url, { method: 'DELETE' });
        return responseFromFetch(response);
    },

    async mutabit(url: string, corpus: string): Promise<Replicatio> {
        const response = await fetch(url, {
            method: 'PATCH',
            body: corpus,
        });
        return responseFromFetch(response);
    },

    // =========================================================================
    // HTTP CLIENT - Advanced
    // =========================================================================

    async rogabit(
        modus: string,
        url: string,
        capita: Record<string, string>,
        corpus: string
    ): Promise<Replicatio> {
        const response = await fetch(url, {
            method: modus,
            headers: capita,
            body: corpus || undefined,
        });
        return responseFromFetch(response);
    },

    // =========================================================================
    // HTTP SERVER
    // =========================================================================

    async exspectabit(portus: number, handler: Handler): Promise<Servitor> {
        const server = Bun.serve({
            port: portus,
            async fetch(request: Request): Promise<Response> {
                const rogatio = await Rogatio._fromRequest(request);
                const replicatio = await handler(rogatio);
                return replicatio._toResponse();
            },
        });
        return new Servitor(server);
    },

    // =========================================================================
    // RESPONSE BUILDERS
    // =========================================================================

    replica(status: number, capita: Record<string, string>, corpus: string): Replicatio {
        return new Replicatio(status, capita, corpus);
    },

    scribe(status: number, corpus: string): Replicatio {
        return new Replicatio(status, { 'Content-Type': 'text/plain' }, corpus);
    },

    funde(status: number, data: Uint8Array): Replicatio {
        return new Replicatio(status, { 'Content-Type': 'application/octet-stream' }, data);
    },

    json(status: number, data: unknown): Replicatio {
        return new Replicatio(
            status,
            { 'Content-Type': 'application/json' },
            JSON.stringify(data)
        );
    },

    redirige(url: string): Replicatio {
        return new Replicatio(302, { 'Location': url }, '');
    },
};
