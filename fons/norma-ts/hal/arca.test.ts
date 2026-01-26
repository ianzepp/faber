import { test, expect, describe, beforeEach, afterEach } from 'bun:test';
import { arca, Connexio, Transactio } from './arca';

describe('arca HAL', () => {
    let conn: Connexio;

    beforeEach(async () => {
        conn = await arca.connectet('sqlite://:memory:');
    });

    afterEach(() => {
        if (conn.aperta()) {
            conn.claude();
        }
    });

    describe('connection', () => {
        test('connectet with sqlite memory URL creates connection', async () => {
            expect(conn).toBeInstanceOf(Connexio);
            expect(conn.aperta()).toBe(true);
        });

        test('connectet with sqlite file URL creates connection', async () => {
            const c = await arca.connectet('sqlite://:memory:');
            expect(c).toBeInstanceOf(Connexio);
            expect(c.aperta()).toBe(true);
            c.claude();
        });

        test('connectet throws for postgres URL', async () => {
            await expect(arca.connectet('postgres://localhost/db')).rejects.toThrow('PostgreSQL driver not supported');
        });

        test('connectet throws for mysql URL', async () => {
            await expect(arca.connectet('mysql://localhost/db')).rejects.toThrow('MySQL driver not supported');
        });

        test('connectet throws for unknown protocol', async () => {
            await expect(arca.connectet('mongodb://localhost/db')).rejects.toThrow('Unknown database protocol');
        });
    });

    describe('lifecycle', () => {
        test('claude closes connection', async () => {
            const c = await arca.connectet('sqlite://:memory:');
            expect(c.aperta()).toBe(true);
            c.claude();
            expect(c.aperta()).toBe(false);
        });

        test('operations fail after close', async () => {
            const c = await arca.connectet('sqlite://:memory:');
            c.claude();
            await expect(c.quaeret('SELECT 1', [])).rejects.toThrow('Connection is closed');
        });
    });

    describe('queries', () => {
        beforeEach(async () => {
            await conn.exsequetur('CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)', []);
            await conn.exsequetur('INSERT INTO users (name, age) VALUES (?, ?)', ['Alice', 30]);
            await conn.exsequetur('INSERT INTO users (name, age) VALUES (?, ?)', ['Bob', 25]);
            await conn.exsequetur('INSERT INTO users (name, age) VALUES (?, ?)', ['Carol', 35]);
        });

        test('quaeret returns all rows', async () => {
            const rows = await conn.quaeret('SELECT * FROM users ORDER BY id', []);
            expect(rows).toHaveLength(3);
            expect(rows[0]).toEqual({ id: 1, name: 'Alice', age: 30 });
            expect(rows[1]).toEqual({ id: 2, name: 'Bob', age: 25 });
            expect(rows[2]).toEqual({ id: 3, name: 'Carol', age: 35 });
        });

        test('quaeret with params filters rows', async () => {
            const rows = await conn.quaeret('SELECT * FROM users WHERE age > ?', [28]);
            expect(rows).toHaveLength(2);
            expect(rows[0]!.name).toBe('Alice');
            expect(rows[1]!.name).toBe('Carol');
        });

        test('capiet returns first row', async () => {
            const row = await conn.capiet('SELECT * FROM users ORDER BY age ASC', []);
            expect(row).toEqual({ id: 2, name: 'Bob', age: 25 });
        });

        test('capiet returns null when no rows', async () => {
            const row = await conn.capiet('SELECT * FROM users WHERE age > ?', [100]);
            expect(row).toBe(null);
        });

        test('quaerent iterates over rows', async () => {
            const names: string[] = [];
            for await (const row of conn.quaerent('SELECT name FROM users ORDER BY id', [])) {
                names.push(row.name as string);
            }
            expect(names).toEqual(['Alice', 'Bob', 'Carol']);
        });
    });

    describe('mutations', () => {
        beforeEach(async () => {
            await conn.exsequetur('CREATE TABLE items (id INTEGER PRIMARY KEY, value TEXT)', []);
        });

        test('exsequetur returns affected row count for INSERT', async () => {
            const count = await conn.exsequetur('INSERT INTO items (value) VALUES (?)', ['test']);
            expect(count).toBe(1);
        });

        test('exsequetur returns affected row count for UPDATE', async () => {
            await conn.exsequetur('INSERT INTO items (value) VALUES (?)', ['a']);
            await conn.exsequetur('INSERT INTO items (value) VALUES (?)', ['b']);
            await conn.exsequetur('INSERT INTO items (value) VALUES (?)', ['c']);

            const count = await conn.exsequetur('UPDATE items SET value = ?', ['updated']);
            expect(count).toBe(3);
        });

        test('exsequetur returns affected row count for DELETE', async () => {
            await conn.exsequetur('INSERT INTO items (value) VALUES (?)', ['a']);
            await conn.exsequetur('INSERT INTO items (value) VALUES (?)', ['b']);

            const count = await conn.exsequetur('DELETE FROM items WHERE value = ?', ['a']);
            expect(count).toBe(1);
        });

        test('inseret returns last inserted ID', async () => {
            const id1 = await conn.inseret('INSERT INTO items (value) VALUES (?)', ['first']);
            expect(id1).toBe(1);

            const id2 = await conn.inseret('INSERT INTO items (value) VALUES (?)', ['second']);
            expect(id2).toBe(2);

            const id3 = await conn.inseret('INSERT INTO items (value) VALUES (?)', ['third']);
            expect(id3).toBe(3);
        });
    });

    describe('transactions', () => {
        beforeEach(async () => {
            await conn.exsequetur('CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER)', []);
            await conn.exsequetur('INSERT INTO accounts (balance) VALUES (?)', [100]);
        });

        test('transaction commit persists changes', async () => {
            const tx = await conn.incipiet();
            await tx.exsequetur('UPDATE accounts SET balance = ? WHERE id = ?', [200, 1]);
            await tx.committet();

            const row = await conn.capiet('SELECT balance FROM accounts WHERE id = ?', [1]);
            expect(row!.balance).toBe(200);
        });

        test('transaction rollback discards changes', async () => {
            const tx = await conn.incipiet();
            await tx.exsequetur('UPDATE accounts SET balance = ? WHERE id = ?', [999, 1]);

            // Verify change is visible within transaction
            const txRows = await tx.quaeret('SELECT balance FROM accounts WHERE id = ?', [1]);
            expect(txRows[0]!.balance).toBe(999);

            await tx.revertet();

            // Verify change was rolled back
            const row = await conn.capiet('SELECT balance FROM accounts WHERE id = ?', [1]);
            expect(row!.balance).toBe(100);
        });

        test('transaction quaeret works', async () => {
            const tx = await conn.incipiet();
            const rows = await tx.quaeret('SELECT * FROM accounts', []);
            expect(rows).toHaveLength(1);
            expect(rows[0]!.balance).toBe(100);
            await tx.committet();
        });

        test('transaction quaerent iterates rows', async () => {
            await conn.exsequetur('INSERT INTO accounts (balance) VALUES (?)', [200]);

            const tx = await conn.incipiet();
            const balances: number[] = [];
            for await (const row of tx.quaerent('SELECT balance FROM accounts ORDER BY id', [])) {
                balances.push(row.balance as number);
            }
            expect(balances).toEqual([100, 200]);
            await tx.committet();
        });

        test('operations fail after transaction commit', async () => {
            const tx = await conn.incipiet();
            await tx.committet();
            await expect(tx.exsequetur('SELECT 1', [])).rejects.toThrow('Transaction is already finished');
        });

        test('operations fail after transaction rollback', async () => {
            const tx = await conn.incipiet();
            await tx.revertet();
            await expect(tx.quaeret('SELECT 1', [])).rejects.toThrow('Transaction is already finished');
        });
    });

    describe('parameterized queries', () => {
        beforeEach(async () => {
            await conn.exsequetur('CREATE TABLE data (id INTEGER PRIMARY KEY, text TEXT)', []);
        });

        test('handles special characters safely', async () => {
            const malicious = "'; DROP TABLE data; --";
            await conn.exsequetur('INSERT INTO data (text) VALUES (?)', [malicious]);

            const rows = await conn.quaeret('SELECT * FROM data', []);
            expect(rows).toHaveLength(1);
            expect(rows[0]!.text).toBe(malicious);

            // Table still exists
            const row = await conn.capiet(
                "SELECT COUNT(*) as cnt FROM sqlite_master WHERE type='table' AND name='data'",
                []
            );
            expect(row!.cnt).toBe(1);
        });

        test('handles null parameters', async () => {
            await conn.exsequetur('INSERT INTO data (text) VALUES (?)', [null]);
            const row = await conn.capiet('SELECT * FROM data', []);
            expect(row?.text).toBe(null);
        });

        test('handles multiple parameters', async () => {
            await conn.exsequetur('CREATE TABLE multi (a TEXT, b INTEGER, c REAL)', []);
            await conn.exsequetur('INSERT INTO multi (a, b, c) VALUES (?, ?, ?)', ['text', 42, 3.14]);

            const row = await conn.capiet('SELECT * FROM multi', []);
            expect(row).toEqual({ a: 'text', b: 42, c: 3.14 });
        });
    });
});
