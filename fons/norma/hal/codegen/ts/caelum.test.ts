import { test, expect, describe, afterEach } from 'bun:test';
import { caelum, Auscultator, Connexus, SocketumUdp } from './caelum';

// Track resources for cleanup
const listeners: Auscultator[] = [];
const connections: Connexus[] = [];
const udpSockets: SocketumUdp[] = [];

afterEach(() => {
    // Clean up all resources
    for (const conn of connections) {
        try { conn.claude(); } catch { /* ignore */ }
    }
    for (const listener of listeners) {
        try { listener.claude(); } catch { /* ignore */ }
    }
    for (const sock of udpSockets) {
        try { sock.claude(); } catch { /* ignore */ }
    }
    connections.length = 0;
    listeners.length = 0;
    udpSockets.length = 0;
});

describe('caelum HAL', () => {
    describe('TCP', () => {
        test('ausculta creates listener on port', async () => {
            const listener = await caelum.ausculta(0); // 0 = random available port
            listeners.push(listener);

            expect(listener.portus()).toBeGreaterThan(0);
        });

        test('connecta connects to listener', async () => {
            const listener = await caelum.ausculta(0);
            listeners.push(listener);

            const port = listener.portus();
            const clientConn = await caelum.connecta('127.0.0.1', port);
            connections.push(clientConn);

            const serverConn = await listener.accipe();
            connections.push(serverConn);

            expect(clientConn.hospesRemotus()).toBe('127.0.0.1');
            expect(clientConn.portusRemotus()).toBe(port);
            expect(serverConn.portusLocalis()).toBe(port);
        });

        test('send and receive data client to server', async () => {
            const listener = await caelum.ausculta(0);
            listeners.push(listener);

            const clientConn = await caelum.connecta('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipe();
            connections.push(serverConn);

            const message = new TextEncoder().encode('Hello, server!');
            await clientConn.scribe(message);

            const received = await serverConn.lege();
            expect(new TextDecoder().decode(received)).toBe('Hello, server!');
        });

        test('send and receive data server to client', async () => {
            const listener = await caelum.ausculta(0);
            listeners.push(listener);

            const clientConn = await caelum.connecta('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipe();
            connections.push(serverConn);

            const message = new TextEncoder().encode('Hello, client!');
            await serverConn.scribe(message);

            const received = await clientConn.lege();
            expect(new TextDecoder().decode(received)).toBe('Hello, client!');
        });

        test('bidirectional communication', async () => {
            const listener = await caelum.ausculta(0);
            listeners.push(listener);

            const clientConn = await caelum.connecta('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipe();
            connections.push(serverConn);

            // Client sends
            await clientConn.scribe(new TextEncoder().encode('ping'));
            const serverReceived = await serverConn.lege();
            expect(new TextDecoder().decode(serverReceived)).toBe('ping');

            // Server responds
            await serverConn.scribe(new TextEncoder().encode('pong'));
            const clientReceived = await clientConn.lege();
            expect(new TextDecoder().decode(clientReceived)).toBe('pong');
        });

        test('multiple clients can connect', async () => {
            const listener = await caelum.ausculta(0);
            listeners.push(listener);

            const client1 = await caelum.connecta('127.0.0.1', listener.portus());
            connections.push(client1);
            const server1 = await listener.accipe();
            connections.push(server1);

            const client2 = await caelum.connecta('127.0.0.1', listener.portus());
            connections.push(client2);
            const server2 = await listener.accipe();
            connections.push(server2);

            // Send different messages
            await client1.scribe(new TextEncoder().encode('from client 1'));
            await client2.scribe(new TextEncoder().encode('from client 2'));

            const msg1 = await server1.lege();
            const msg2 = await server2.lege();

            expect(new TextDecoder().decode(msg1)).toBe('from client 1');
            expect(new TextDecoder().decode(msg2)).toBe('from client 2');
        });

        test('legeUsque reads exact number of bytes', async () => {
            const listener = await caelum.ausculta(0);
            listeners.push(listener);

            const clientConn = await caelum.connecta('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipe();
            connections.push(serverConn);

            // Send 10 bytes
            await clientConn.scribe(new TextEncoder().encode('0123456789'));

            // Read exactly 5 bytes
            const first5 = await serverConn.legeUsque(5);
            expect(new TextDecoder().decode(first5)).toBe('01234');

            // Read remaining 5 bytes
            const next5 = await serverConn.legeUsque(5);
            expect(new TextDecoder().decode(next5)).toBe('56789');
        });

        test('address getters return correct values', async () => {
            const listener = await caelum.ausculta(0);
            listeners.push(listener);

            const clientConn = await caelum.connecta('127.0.0.1', listener.portus());
            connections.push(clientConn);

            const serverConn = await listener.accipe();
            connections.push(serverConn);

            // Client sees server's port as remote
            expect(clientConn.portusRemotus()).toBe(listener.portus());
            expect(clientConn.hospesRemotus()).toBe('127.0.0.1');

            // Server sees client's port as remote
            expect(serverConn.portusRemotus()).toBe(clientConn.portusLocalis());
            expect(serverConn.hospesRemotus()).toBe('127.0.0.1');
        });

        test('connections close cleanly', async () => {
            const listener = await caelum.ausculta(0);
            listeners.push(listener);

            const clientConn = await caelum.connecta('127.0.0.1', listener.portus());
            const serverConn = await listener.accipe();

            // Close client
            clientConn.claude();

            // Server should get empty data on read (connection closed)
            const data = await serverConn.lege();
            expect(data.length).toBe(0);

            serverConn.claude();
            listener.claude();
        });
    });

    describe('UDP', () => {
        test('bindUdp creates socket on port', async () => {
            const socket = await caelum.bindUdp(0);
            udpSockets.push(socket);

            expect(socket.portus()).toBeGreaterThan(0);
        });

        test('send and receive datagram', async () => {
            const receiver = await caelum.bindUdp(0);
            udpSockets.push(receiver);

            const sender = await caelum.bindUdp(0);
            udpSockets.push(sender);

            const message = new TextEncoder().encode('Hello, UDP!');
            await sender.mitte('127.0.0.1', receiver.portus(), message);

            const datum = await receiver.recipe();
            expect(new TextDecoder().decode(datum.data())).toBe('Hello, UDP!');
            expect(datum.hospes()).toBe('127.0.0.1');
            expect(datum.portus()).toBe(sender.portus());
        });

        test('mitteUdp sends without binding', async () => {
            const receiver = await caelum.bindUdp(0);
            udpSockets.push(receiver);

            const message = new TextEncoder().encode('One-shot UDP');
            await caelum.mitteUdp('127.0.0.1', receiver.portus(), message);

            const datum = await receiver.recipe();
            expect(new TextDecoder().decode(datum.data())).toBe('One-shot UDP');
        });

        test('DatumUdp returns correct sender info', async () => {
            const receiver = await caelum.bindUdp(0);
            udpSockets.push(receiver);

            const sender = await caelum.bindUdp(0);
            udpSockets.push(sender);

            await sender.mitte('127.0.0.1', receiver.portus(), new Uint8Array([1, 2, 3]));

            const datum = await receiver.recipe();
            expect(datum.data()).toEqual(new Uint8Array([1, 2, 3]));
            expect(datum.hospes()).toBe('127.0.0.1');
            expect(datum.portus()).toBe(sender.portus());
        });

        test('UDP socket closes cleanly', async () => {
            const socket = await caelum.bindUdp(0);

            // Should not throw
            socket.claude();
        });

        test('multiple datagrams queued', async () => {
            const receiver = await caelum.bindUdp(0);
            udpSockets.push(receiver);

            const sender = await caelum.bindUdp(0);
            udpSockets.push(sender);

            // Send multiple datagrams
            await sender.mitte('127.0.0.1', receiver.portus(), new TextEncoder().encode('msg1'));
            await sender.mitte('127.0.0.1', receiver.portus(), new TextEncoder().encode('msg2'));
            await sender.mitte('127.0.0.1', receiver.portus(), new TextEncoder().encode('msg3'));

            // Small delay to ensure all messages arrive
            await new Promise(resolve => setTimeout(resolve, 50));

            // Receive all
            const d1 = await receiver.recipe();
            const d2 = await receiver.recipe();
            const d3 = await receiver.recipe();

            const messages = [
                new TextDecoder().decode(d1.data()),
                new TextDecoder().decode(d2.data()),
                new TextDecoder().decode(d3.data()),
            ];

            // UDP doesn't guarantee order, but all messages should arrive
            expect(messages).toContain('msg1');
            expect(messages).toContain('msg2');
            expect(messages).toContain('msg3');
        });
    });
});
