import { test, expect, describe, beforeEach, afterEach } from 'bun:test';
import { arca, Connexio, Transactio } from './arca';

describe('arca HAL', () => {
    let conn: Connexio;

    beforeEach(async () => {
        conn = await arca.memoria();
    });

    afterEach(() => {
        if (conn.aperta()) {
            conn.claude();
        }
    });

    describe('connection', () => {
        test('memoria creates working in-memory database', async () => {
            expect(conn).toBeInstanceOf(Connexio);
            expect(conn.aperta()).toBe(true);
        });

        test('connecta with sqlite URL creates connection', async () => {
            const c = await arca.connecta('sqlite://:memory:');
            expect(c).toBeInstanceOf(Connexio);
            expect(c.aperta()).toBe(true);
            c.claude();
        });

        test('connecta throws for postgres URL', async () => {
            await expect(arca.connecta('postgres://localhost/db')).rejects.toThrow('PostgreSQL driver not supported');
        });

        test('connecta throws for mysql URL', async () => {
            await expect(arca.connecta('mysql://localhost/db')).rejects.toThrow('MySQL driver not supported');
        });

        test('connecta throws for unknown protocol', async () => {
            await expect(arca.connecta('mongodb://localhost/db')).rejects.toThrow('Unknown database protocol');
        });
    });

    describe('lifecycle', () => {
        test('claude closes connection', async () => {
            const c = await arca.memoria();
            expect(c.aperta()).toBe(true);
            c.claude();
            expect(c.aperta()).toBe(false);
        });

        test('operations fail after close', async () => {
            const c = await arca.memoria();
            c.claude();
            await expect(c.quaereOmnes('SELECT 1')).rejects.toThrow('Connection is closed');
        });
    });

    describe('queries', () => {
        beforeEach(async () => {
            await conn.exsequi('CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)');
            await conn.exsequi('INSERT INTO users (name, age) VALUES (?, ?)', ['Alice', 30]);
            await conn.exsequi('INSERT INTO users (name, age) VALUES (?, ?)', ['Bob', 25]);
            await conn.exsequi('INSERT INTO users (name, age) VALUES (?, ?)', ['Carol', 35]);
        });

        test('quaereOmnes returns all rows', async () => {
            const rows = await conn.quaereOmnes('SELECT * FROM users ORDER BY id');
            expect(rows).toHaveLength(3);
            expect(rows[0]).toEqual({ id: 1, name: 'Alice', age: 30 });
            expect(rows[1]).toEqual({ id: 2, name: 'Bob', age: 25 });
            expect(rows[2]).toEqual({ id: 3, name: 'Carol', age: 35 });
        });

        test('quaereOmnes with params filters rows', async () => {
            const rows = await conn.quaereOmnes('SELECT * FROM users WHERE age > ?', [28]);
            expect(rows).toHaveLength(2);
            expect(rows[0]!.name).toBe('Alice');
            expect(rows[1]!.name).toBe('Carol');
        });

        test('quaerePrimum returns first row', async () => {
            const row = await conn.quaerePrimum('SELECT * FROM users ORDER BY age ASC');
            expect(row).toEqual({ id: 2, name: 'Bob', age: 25 });
        });

        test('quaerePrimum returns null when no rows', async () => {
            const row = await conn.quaerePrimum('SELECT * FROM users WHERE age > ?', [100]);
            expect(row).toBe(null);
        });

        test('quaereValorem extracts single value', async () => {
            const count = await conn.quaereValorem('SELECT COUNT(*) FROM users');
            expect(count).toBe(3);
        });

        test('quaereValorem returns null for empty result', async () => {
            const value = await conn.quaereValorem('SELECT name FROM users WHERE id = ?', [999]);
            expect(value).toBe(null);
        });

        test('quaere iterates over rows', async () => {
            const names: string[] = [];
            for await (const row of conn.quaere('SELECT name FROM users ORDER BY id')) {
                names.push(row.name as string);
            }
            expect(names).toEqual(['Alice', 'Bob', 'Carol']);
        });
    });

    describe('mutations', () => {
        beforeEach(async () => {
            await conn.exsequi('CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT)');
        });

        test('exsequi returns affected row count for INSERT', async () => {
            const count = await conn.exsequi('INSERT INTO items (value) VALUES (?)', ['test']);
            expect(count).toBe(1);
        });

        test('exsequi returns affected row count for UPDATE', async () => {
            await conn.exsequi('INSERT INTO items (value) VALUES (?)', ['a']);
            await conn.exsequi('INSERT INTO items (value) VALUES (?)', ['b']);
            await conn.exsequi('INSERT INTO items (value) VALUES (?)', ['c']);

            const count = await conn.exsequi('UPDATE items SET value = ?', ['updated']);
            expect(count).toBe(3);
        });

        test('exsequi returns affected row count for DELETE', async () => {
            await conn.exsequi('INSERT INTO items (value) VALUES (?)', ['a']);
            await conn.exsequi('INSERT INTO items (value) VALUES (?)', ['b']);

            const count = await conn.exsequi('DELETE FROM items WHERE value = ?', ['a']);
            expect(count).toBe(1);
        });

        test('insere returns last inserted ID', async () => {
            const id1 = await conn.insere('INSERT INTO items (value) VALUES (?)', ['first']);
            expect(id1).toBe(1);

            const id2 = await conn.insere('INSERT INTO items (value) VALUES (?)', ['second']);
            expect(id2).toBe(2);

            const id3 = await conn.insere('INSERT INTO items (value) VALUES (?)', ['third']);
            expect(id3).toBe(3);
        });
    });

    describe('transactions', () => {
        beforeEach(async () => {
            await conn.exsequi('CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER)');
            await conn.exsequi('INSERT INTO accounts (balance) VALUES (?)', [100]);
        });

        test('transaction commit persists changes', async () => {
            const tx = await conn.incipe();
            await tx.exsequi('UPDATE accounts SET balance = ? WHERE id = ?', [200, 1]);
            await tx.committe();

            const balance = await conn.quaereValorem('SELECT balance FROM accounts WHERE id = ?', [1]);
            expect(balance).toBe(200);
        });

        test('transaction rollback discards changes', async () => {
            const tx = await conn.incipe();
            await tx.exsequi('UPDATE accounts SET balance = ? WHERE id = ?', [999, 1]);

            // Verify change is visible within transaction
            const txRows = await tx.quaereOmnes('SELECT balance FROM accounts WHERE id = ?', [1]);
            expect(txRows[0]!.balance).toBe(999);

            await tx.reverte();

            // Verify change was rolled back
            const balance = await conn.quaereValorem('SELECT balance FROM accounts WHERE id = ?', [1]);
            expect(balance).toBe(100);
        });

        test('transaction quaereOmnes works', async () => {
            const tx = await conn.incipe();
            const rows = await tx.quaereOmnes('SELECT * FROM accounts');
            expect(rows).toHaveLength(1);
            expect(rows[0]!.balance).toBe(100);
            await tx.committe();
        });

        test('transaction quaere iterates rows', async () => {
            await conn.exsequi('INSERT INTO accounts (balance) VALUES (?)', [200]);

            const tx = await conn.incipe();
            const balances: number[] = [];
            for await (const row of tx.quaere('SELECT balance FROM accounts ORDER BY id')) {
                balances.push(row.balance as number);
            }
            expect(balances).toEqual([100, 200]);
            await tx.committe();
        });

        test('operations fail after transaction commit', async () => {
            const tx = await conn.incipe();
            await tx.committe();
            await expect(tx.exsequi('SELECT 1')).rejects.toThrow('Transaction is already finished');
        });

        test('operations fail after transaction rollback', async () => {
            const tx = await conn.incipe();
            await tx.reverte();
            await expect(tx.quaereOmnes('SELECT 1')).rejects.toThrow('Transaction is already finished');
        });
    });

    describe('parameterized queries', () => {
        beforeEach(async () => {
            await conn.exsequi('CREATE TABLE data (id INTEGER PRIMARY KEY, text TEXT)');
        });

        test('handles special characters safely', async () => {
            const malicious = "'; DROP TABLE data; --";
            await conn.exsequi('INSERT INTO data (text) VALUES (?)', [malicious]);

            const rows = await conn.quaereOmnes('SELECT * FROM data');
            expect(rows).toHaveLength(1);
            expect(rows[0]!.text).toBe(malicious);

            // Table still exists
            const tableExists = await conn.quaereValorem(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='data'"
            );
            expect(tableExists).toBe(1);
        });

        test('handles null parameters', async () => {
            await conn.exsequi('INSERT INTO data (text) VALUES (?)', [null]);
            const row = await conn.quaerePrimum('SELECT * FROM data');
            expect(row?.text).toBe(null);
        });

        test('handles multiple parameters', async () => {
            await conn.exsequi('CREATE TABLE multi (a TEXT, b INTEGER, c REAL)');
            await conn.exsequi('INSERT INTO multi (a, b, c) VALUES (?, ?, ?)', ['text', 42, 3.14]);

            const row = await conn.quaerePrimum('SELECT * FROM multi');
            expect(row).toEqual({ a: 'text', b: 42, c: 3.14 });
        });
    });
});
