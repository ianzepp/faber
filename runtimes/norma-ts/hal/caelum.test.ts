import { test, expect, describe, afterEach } from 'bun:test';
import { caelum, Auscultator, Connexus } from './caelum';

// Track resources for cleanup
const listeners: Auscultator[] = [];
const connections: Connexus[] = [];

afterEach(() => {
    for (const conn of connections) {
        try { conn.claude(); } catch { /* ignore */ }
    }
    for (const listener of listeners) {
        try { listener.claude(); } catch { /* ignore */ }
    }
    connections.length = 0;
    listeners.length = 0;
});

describe('caelum HAL', () => {
    describe('TCP', () => {
        test('auscultabit creates listener on port', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            expect(listener.portus()).toBeGreaterThan(0);
        });

        test('connectet connects to listener', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            const port = listener.portus();
            const clientConn = await caelum.connectet('127.0.0.1', port);
            connections.push(clientConn);

            const serverConn = await listener.accipiet();
            connections.push(serverConn);

            expect(clientConn.hospesRemotus()).toBe('127.0.0.1');
            expect(clientConn.portusRemotus()).toBe(port);
            expect(serverConn.portusLocalis()).toBe(port);
        });

        test('send and receive data client to server', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            const clientConn = await caelum.connectet('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipiet();
            connections.push(serverConn);

            const message = new TextEncoder().encode('Hello, server!');
            await clientConn.fundet(message);

            const received = await serverConn.hauriet();
            expect(new TextDecoder().decode(received)).toBe('Hello, server!');
        });

        test('send and receive data server to client', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            const clientConn = await caelum.connectet('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipiet();
            connections.push(serverConn);

            const message = new TextEncoder().encode('Hello, client!');
            await serverConn.fundet(message);

            const received = await clientConn.hauriet();
            expect(new TextDecoder().decode(received)).toBe('Hello, client!');
        });

        test('bidirectional communication', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            const clientConn = await caelum.connectet('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipiet();
            connections.push(serverConn);

            // Client sends
            await clientConn.fundet(new TextEncoder().encode('ping'));
            const serverReceived = await serverConn.hauriet();
            expect(new TextDecoder().decode(serverReceived)).toBe('ping');

            // Server responds
            await serverConn.fundet(new TextEncoder().encode('pong'));
            const clientReceived = await clientConn.hauriet();
            expect(new TextDecoder().decode(clientReceived)).toBe('pong');
        });

        test('multiple clients can connect', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            const client1 = await caelum.connectet('127.0.0.1', listener.portus());
            connections.push(client1);
            const server1 = await listener.accipiet();
            connections.push(server1);

            const client2 = await caelum.connectet('127.0.0.1', listener.portus());
            connections.push(client2);
            const server2 = await listener.accipiet();
            connections.push(server2);

            // Send different messages
            await client1.fundet(new TextEncoder().encode('from client 1'));
            await client2.fundet(new TextEncoder().encode('from client 2'));

            const msg1 = await server1.hauriet();
            const msg2 = await server2.hauriet();

            expect(new TextDecoder().decode(msg1)).toBe('from client 1');
            expect(new TextDecoder().decode(msg2)).toBe('from client 2');
        });

        test('hauriet(n) reads exact number of bytes', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            const clientConn = await caelum.connectet('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipiet();
            connections.push(serverConn);

            // Send 10 bytes
            await clientConn.fundet(new TextEncoder().encode('0123456789'));

            // Read exactly 5 bytes
            const first5 = await serverConn.hauriet(5);
            expect(new TextDecoder().decode(first5)).toBe('01234');

            // Read remaining 5 bytes
            const next5 = await serverConn.hauriet(5);
            expect(new TextDecoder().decode(next5)).toBe('56789');
        });

        test('address getters return correct values', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            const clientConn = await caelum.connectet('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipiet();
            connections.push(serverConn);

            // Client sees server's port as remote
            expect(clientConn.portusRemotus()).toBe(listener.portus());
            expect(clientConn.hospesRemotus()).toBe('127.0.0.1');

            // Server sees client's port as remote
            expect(serverConn.portusRemotus()).toBe(clientConn.portusLocalis());
            expect(serverConn.hospesRemotus()).toBe('127.0.0.1');
        });

        test('connections close cleanly', async () => {
            const listener = await caelum.auscultabit(0);
            listeners.push(listener);

            const clientConn = await caelum.connectet('127.0.0.1', listener.portus());
            const serverConn = await listener.accipiet();

            // Close client
            clientConn.claude();

            // Server should get empty data on read (connection closed)
            const data = await serverConn.hauriet();
            expect(data.length).toBe(0);

            serverConn.claude();
            listener.claude();
        });

        test('sync methods throw not supported', () => {
            expect(() => caelum.ausculta(0)).toThrow('not supported');
            expect(() => caelum.connecte('127.0.0.1', 80)).toThrow('not supported');
        });
    });

    // TODO: UDP tests deferred
});
