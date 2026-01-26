/**
 * caelum.ts - Network Socket Implementation (TCP/UDP)
 *
 * Native TypeScript implementation of the HAL network socket interface.
 * Uses Bun's native TCP APIs when available, falls back to Node's net module.
 * UDP uses Node's dgram module.
 */

import * as net from 'node:net';
import * as tls from 'node:tls';
import * as dgram from 'node:dgram';
import * as fs from 'node:fs';

// Strip IPv6-mapped IPv4 prefix (::ffff:) for consistency
function normalizeAddress(addr: string): string {
    if (addr.startsWith('::ffff:')) {
        return addr.slice(7);
    }
    return addr;
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

    async accipe(): Promise<Connexus> {
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
        // Reject any pending accipe calls
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
            // Resolve pending reads with empty data
            for (const resolver of this.dataResolvers) {
                resolver(new Uint8Array(0));
            }
            this.dataResolvers = [];
        });

        this.socket.on('error', () => {
            this.closed = true;
        });
    }

    async lege(): Promise<Uint8Array> {
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

    async legeUsque(n: number): Promise<Uint8Array> {
        // Collect up to n bytes
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
                    // Take partial chunk
                    result.push(...chunk.slice(0, needed));
                    this.dataBuffer[0] = chunk.slice(needed);
                }
            }
            else if (this.closed) {
                break;
            }
            else {
                // Wait for more data
                const data = await this.lege();
                if (data.length === 0) {
                    break; // Connection closed
                }
                this.dataBuffer.unshift(Buffer.from(data));
            }
        }

        return new Uint8Array(result);
    }

    async scribe(data: Uint8Array): Promise<void> {
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
// UDP DATAGRAM
// =============================================================================

export class DatumUdp {
    private _data: Uint8Array;
    private _hospes: string;
    private _portus: number;

    constructor(data: Uint8Array, hospes: string, portus: number) {
        this._data = data;
        this._hospes = hospes;
        this._portus = portus;
    }

    data(): Uint8Array {
        return this._data;
    }

    hospes(): string {
        return this._hospes;
    }

    portus(): number {
        return this._portus;
    }
}

// =============================================================================
// UDP SOCKET
// =============================================================================

export class SocketumUdp {
    private socket: dgram.Socket;
    private pendingMessages: DatumUdp[] = [];
    private pendingResolvers: Array<(msg: DatumUdp) => void> = [];
    private closed = false;
    private localPort: number;

    constructor(socket: dgram.Socket, port: number) {
        this.socket = socket;
        this.localPort = port;

        this.socket.on('message', (msg: Buffer, rinfo: dgram.RemoteInfo) => {
            const datum = new DatumUdp(new Uint8Array(msg), rinfo.address, rinfo.port);
            const resolver = this.pendingResolvers.shift();
            if (resolver) {
                resolver(datum);
            }
            else {
                this.pendingMessages.push(datum);
            }
        });
    }

    async recipe(): Promise<DatumUdp> {
        if (this.closed) {
            throw new Error('Socket is closed');
        }

        const pending = this.pendingMessages.shift();
        if (pending) {
            return pending;
        }

        return new Promise((resolve) => {
            this.pendingResolvers.push(resolve);
        });
    }

    async mitte(hospes: string, portus: number, data: Uint8Array): Promise<void> {
        return new Promise((resolve, reject) => {
            this.socket.send(Buffer.from(data), portus, hospes, (err) => {
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
        this.socket.close();
    }

    portus(): number {
        return this.localPort;
    }
}

// =============================================================================
// MAIN MODULE EXPORT
// =============================================================================

export const caelum = {
    // =========================================================================
    // TCP SERVER
    // =========================================================================

    async ausculta(portus: number): Promise<Auscultator> {
        return new Promise((resolve, reject) => {
            const server = net.createServer();

            server.on('error', reject);

            server.listen(portus, () => {
                const addr = server.address() as net.AddressInfo;
                resolve(new Auscultator(server, addr.port));
            });
        });
    },

    async auscultaTls(portus: number, certPath: string, keyPath: string): Promise<Auscultator> {
        return new Promise((resolve, reject) => {
            const options: tls.TlsOptions = {
                cert: fs.readFileSync(certPath),
                key: fs.readFileSync(keyPath),
            };

            const server = tls.createServer(options);

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

    async connecta(hospes: string, portus: number): Promise<Connexus> {
        return new Promise((resolve, reject) => {
            const socket = net.createConnection({ host: hospes, port: portus }, () => {
                resolve(new Connexus(socket));
            });

            socket.on('error', reject);
        });
    },

    async connectaTls(hospes: string, portus: number): Promise<Connexus> {
        return new Promise((resolve, reject) => {
            const socket = tls.connect({ host: hospes, port: portus }, () => {
                resolve(new Connexus(socket));
            });

            socket.on('error', reject);
        });
    },

    // =========================================================================
    // UDP
    // =========================================================================

    async bindUdp(portus: number): Promise<SocketumUdp> {
        return new Promise((resolve, reject) => {
            const socket = dgram.createSocket('udp4');

            socket.on('error', reject);

            socket.bind(portus, () => {
                const addr = socket.address();
                resolve(new SocketumUdp(socket, addr.port));
            });
        });
    },

    async mitteUdp(hospes: string, portus: number, data: Uint8Array): Promise<void> {
        return new Promise((resolve, reject) => {
            const socket = dgram.createSocket('udp4');

            socket.send(Buffer.from(data), portus, hospes, (err) => {
                socket.close();
                if (err) {
                    reject(err);
                }
                else {
                    resolve();
                }
            });
        });
    },
};
