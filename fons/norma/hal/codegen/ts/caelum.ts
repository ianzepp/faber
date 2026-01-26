/**
 * caelum.ts - Network Socket Implementation (TCP)
 *
 * Native TypeScript implementation of the HAL network socket interface.
 * Uses Node's net/tls modules.
 *
 * Verb conjugation:
 *   - Imperative (ausculta, connecte, hauri, funde): sync (throws in JS runtime)
 *   - Future (auscultabit, connectet, hauriet, fundet): async
 */

import * as net from 'node:net';
import * as tls from 'node:tls';
import * as fs from 'node:fs';

// Strip IPv6-mapped IPv4 prefix (::ffff:) for consistency
function normalizeAddress(addr: string): string {
    if (addr.startsWith('::ffff:')) {
        return addr.slice(7);
    }
    return addr;
}

function syncNotSupported(): never {
    throw new Error('Synchronous socket I/O is not supported in JavaScript runtime');
}

// =============================================================================
// TCP LISTENER
// =============================================================================

export class Auscultator {
    private server: net.Server;
    private pendingConnections: Connexus[] = [];
    private pendingResolvers: Array<(conn: Connexus) => void> = [];
    private closed = false;
    private localPort: number;

    constructor(server: net.Server, port: number) {
        this.server = server;
        this.localPort = port;

        this.server.on('connection', (socket: net.Socket) => {
            const conn = new Connexus(socket);
            const resolver = this.pendingResolvers.shift();
            if (resolver) {
                resolver(conn);
            }
            else {
                this.pendingConnections.push(conn);
            }
        });
    }

    // Sync accept - not supported in JS
    accipe(): Connexus {
        return syncNotSupported();
    }

    // Async accept
    async accipiet(): Promise<Connexus> {
        if (this.closed) {
            throw new Error('Listener is closed');
        }

        const pending = this.pendingConnections.shift();
        if (pending) {
            return pending;
        }

        return new Promise((resolve) => {
            this.pendingResolvers.push(resolve);
        });
    }

    claude(): void {
        this.closed = true;
        this.server.close();
        this.pendingResolvers = [];
    }

    portus(): number {
        return this.localPort;
    }
}

// =============================================================================
// TCP CONNECTION
// =============================================================================

export class Connexus {
    private socket: net.Socket;
    private dataBuffer: Buffer[] = [];
    private dataResolvers: Array<(data: Uint8Array) => void> = [];
    private closed = false;

    constructor(socket: net.Socket) {
        this.socket = socket;

        this.socket.on('data', (chunk: Buffer) => {
            const resolver = this.dataResolvers.shift();
            if (resolver) {
                resolver(new Uint8Array(chunk));
            }
            else {
                this.dataBuffer.push(chunk);
            }
        });

        this.socket.on('close', () => {
            this.closed = true;
            for (const resolver of this.dataResolvers) {
                resolver(new Uint8Array(0));
            }
            this.dataResolvers = [];
        });

        this.socket.on('error', () => {
            this.closed = true;
        });
    }

    // Sync draw bytes - not supported in JS
    hauri(_n?: number): Uint8Array {
        return syncNotSupported();
    }

    // Async draw bytes (up to n if provided, otherwise whatever is available)
    async hauriet(n?: number): Promise<Uint8Array> {
        if (n !== undefined) {
            return this.haurietUsque(n);
        }

        if (this.dataBuffer.length > 0) {
            const chunk = this.dataBuffer.shift()!;
            return new Uint8Array(chunk);
        }

        if (this.closed) {
            return new Uint8Array(0);
        }

        return new Promise((resolve) => {
            this.dataResolvers.push(resolve);
        });
    }

    // Internal: read up to n bytes
    private async haurietUsque(n: number): Promise<Uint8Array> {
        const result: number[] = [];

        while (result.length < n) {
            if (this.dataBuffer.length > 0) {
                const chunk = this.dataBuffer[0];
                const needed = n - result.length;

                if (chunk.length <= needed) {
                    this.dataBuffer.shift();
                    result.push(...chunk);
                }
                else {
                    result.push(...chunk.slice(0, needed));
                    this.dataBuffer[0] = chunk.slice(needed);
                }
            }
            else if (this.closed) {
                break;
            }
            else {
                const data = await this.hauriet();
                if (data.length === 0) {
                    break;
                }
                this.dataBuffer.unshift(Buffer.from(data));
            }
        }

        return new Uint8Array(result);
    }

    // Sync pour bytes - not supported in JS
    funde(_data: Uint8Array): void {
        syncNotSupported();
    }

    // Async pour bytes
    async fundet(data: Uint8Array): Promise<void> {
        return new Promise((resolve, reject) => {
            this.socket.write(Buffer.from(data), (err) => {
                if (err) {
                    reject(err);
                }
                else {
                    resolve();
                }
            });
        });
    }

    claude(): void {
        this.closed = true;
        this.socket.destroy();
    }

    hospesRemotus(): string {
        return normalizeAddress(this.socket.remoteAddress ?? '');
    }

    portusRemotus(): number {
        return this.socket.remotePort ?? 0;
    }

    hospesLocalis(): string {
        return normalizeAddress(this.socket.localAddress ?? '');
    }

    portusLocalis(): number {
        return this.socket.localPort ?? 0;
    }
}

// =============================================================================
// MAIN MODULE EXPORT
// =============================================================================

export const caelum = {
    // =========================================================================
    // TCP SERVER
    // =========================================================================

    // Sync listen - not supported in JS
    ausculta(_portus: number, _cert?: string, _key?: string): Auscultator {
        return syncNotSupported();
    },

    // Async listen (TLS when cert/key provided)
    async auscultabit(portus: number, cert?: string, key?: string): Promise<Auscultator> {
        const useTls = cert !== undefined && key !== undefined;

        return new Promise((resolve, reject) => {
            let server: net.Server;

            if (useTls) {
                const options: tls.TlsOptions = {
                    cert: fs.readFileSync(cert),
                    key: fs.readFileSync(key),
                };
                server = tls.createServer(options);
            }
            else {
                server = net.createServer();
            }

            server.on('error', reject);

            server.listen(portus, () => {
                const addr = server.address() as net.AddressInfo;
                resolve(new Auscultator(server, addr.port));
            });
        });
    },

    // =========================================================================
    // TCP CLIENT
    // =========================================================================

    // Sync connect - not supported in JS
    connecte(_hospes: string, _portus: number, _tute?: boolean): Connexus {
        return syncNotSupported();
    },

    // Async connect (TLS when tute=true)
    async connectet(hospes: string, portus: number, tute: boolean = false): Promise<Connexus> {
        return new Promise((resolve, reject) => {
            const socket = tute
                ? tls.connect({ host: hospes, port: portus }, () => resolve(new Connexus(socket)))
                : net.createConnection({ host: hospes, port: portus }, () => resolve(new Connexus(socket)));

            socket.on('error', reject);
        });
    },
};
