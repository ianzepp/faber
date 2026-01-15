/**
 * arca.ts - Database Device Implementation
 *
 * Native TypeScript implementation of the HAL arca (database) interface.
 * Uses Bun's built-in SQLite for local database operations.
 */

import { Database, type SQLQueryBindings } from 'bun:sqlite';

type Row = Record<string, unknown>;
type Params = SQLQueryBindings[];

/**
 * Database connection wrapper
 */
export class Connexio {
    private db: Database;
    private isOpen: boolean = true;

    constructor(db: Database) {
        this.db = db;
    }

    // =========================================================================
    // QUERIES
    // =========================================================================

    async *quaere(sql: string, params: Params = []): AsyncGenerator<Row> {
        this.assertOpen();
        const stmt = this.db.prepare(sql);
        const rows = stmt.all(...params) as Row[];
        for (const row of rows) {
            yield row;
        }
    }

    async quaereOmnes(sql: string, params: Params = []): Promise<Row[]> {
        this.assertOpen();
        const stmt = this.db.prepare(sql);
        return stmt.all(...params) as Row[];
    }

    async quaerePrimum(sql: string, params: Params = []): Promise<Row | null> {
        this.assertOpen();
        const stmt = this.db.prepare(sql);
        const row = stmt.get(...params) as Row | null;
        return row ?? null;
    }

    async quaereValorem(sql: string, params: Params = []): Promise<unknown> {
        this.assertOpen();
        const row = await this.quaerePrimum(sql, params);
        if (row === null) {
            return null;
        }
        const values = Object.values(row);
        return values.length > 0 ? values[0] : null;
    }

    // =========================================================================
    // MUTATIONS
    // =========================================================================

    async exsequi(sql: string, params: Params = []): Promise<number> {
        this.assertOpen();
        const stmt = this.db.prepare(sql);
        const result = stmt.run(...params);
        return result.changes;
    }

    async insere(sql: string, params: Params = []): Promise<number> {
        this.assertOpen();
        const stmt = this.db.prepare(sql);
        const result = stmt.run(...params);
        return Number(result.lastInsertRowid);
    }

    // =========================================================================
    // TRANSACTIONS
    // =========================================================================

    async incipe(): Promise<Transactio> {
        this.assertOpen();
        this.db.run('BEGIN TRANSACTION');
        return new Transactio(this.db);
    }

    // =========================================================================
    // LIFECYCLE
    // =========================================================================

    claude(): void {
        if (this.isOpen) {
            this.db.close();
            this.isOpen = false;
        }
    }

    aperta(): boolean {
        return this.isOpen;
    }

    private assertOpen(): void {
        if (!this.isOpen) {
            throw new Error('Connection is closed');
        }
    }
}

/**
 * Database transaction wrapper
 */
export class Transactio {
    private db: Database;
    private finished: boolean = false;

    constructor(db: Database) {
        this.db = db;
    }

    // =========================================================================
    // QUERIES
    // =========================================================================

    async *quaere(sql: string, params: Params = []): AsyncGenerator<Row> {
        this.assertActive();
        const stmt = this.db.prepare(sql);
        const rows = stmt.all(...params) as Row[];
        for (const row of rows) {
            yield row;
        }
    }

    async quaereOmnes(sql: string, params: Params = []): Promise<Row[]> {
        this.assertActive();
        const stmt = this.db.prepare(sql);
        return stmt.all(...params) as Row[];
    }

    // =========================================================================
    // MUTATIONS
    // =========================================================================

    async exsequi(sql: string, params: Params = []): Promise<number> {
        this.assertActive();
        const stmt = this.db.prepare(sql);
        const result = stmt.run(...params);
        return result.changes;
    }

    // =========================================================================
    // TRANSACTION CONTROL
    // =========================================================================

    async committe(): Promise<void> {
        this.assertActive();
        this.db.run('COMMIT');
        this.finished = true;
    }

    async reverte(): Promise<void> {
        this.assertActive();
        this.db.run('ROLLBACK');
        this.finished = true;
    }

    private assertActive(): void {
        if (this.finished) {
            throw new Error('Transaction is already finished');
        }
    }
}

/**
 * arca module - database connection functions
 */
export const arca = {
    // =========================================================================
    // CONNECTION
    // =========================================================================

    async connecta(url: string): Promise<Connexio> {
        // Handle sqlite://:memory: specially since URL parser chokes on it
        if (url === 'sqlite://:memory:' || url === 'sqlite::memory:') {
            return arca.memoria();
        }

        // Extract protocol manually for URLs that may not parse
        const colonIdx = url.indexOf(':');
        if (colonIdx === -1) {
            throw new Error('Invalid database URL: no protocol');
        }
        const protocol = url.slice(0, colonIdx + 1);

        if (protocol === 'sqlite:') {
            // sqlite:///path/to/db
            const path = url.slice('sqlite://'.length);
            return arca.sqlite(path);
        }

        if (protocol === 'postgres:' || protocol === 'postgresql:') {
            throw new Error('PostgreSQL driver not supported');
        }

        if (protocol === 'mysql:') {
            throw new Error('MySQL driver not supported');
        }

        throw new Error(`Unknown database protocol: ${protocol}`);
    },

    async sqlite(path: string): Promise<Connexio> {
        const db = new Database(path);
        return new Connexio(db);
    },

    async memoria(): Promise<Connexio> {
        const db = new Database(':memory:');
        return new Connexio(db);
    },
};
