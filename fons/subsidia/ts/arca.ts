// arca.ts - Database adapter library for TypeScript target
//
// Provides connection pooling and query execution for PostgreSQL, MySQL, SQLite.
// Driver is inferred from URL scheme.

// =============================================================================
// PROTOCOLS
// =============================================================================

export interface Connexio {
    lege(sql: string, params: unknown[]): AsyncIterable<Record<string, unknown>>;
    muta(sql: string, params: unknown[]): Promise<number>;
    incipe(): Promise<Transactio>;
    claude(): void;
}

export interface Transactio {
    lege(sql: string, params: unknown[]): AsyncIterable<Record<string, unknown>>;
    muta(sql: string, params: unknown[]): Promise<number>;
    committe(): Promise<void>;
    reverte(): Promise<void>;
}

// =============================================================================
// POSTGRESQL DRIVER
// =============================================================================
//
// NOTE: PostgreSQL support requires the 'pg' package to be installed.
// Install with: bun add pg @types/pg
//
// Commented out until pg is added as a dependency.

// import pg from "pg";
//
// class PgConnexio implements Connexio {
//     private pool: pg.Pool;
//
//     constructor(url: string) {
//         this.pool = new pg.Pool({ connectionString: url });
//     }
//
//     async *lege(sql: string, params: unknown[]): AsyncIterable<Record<string, unknown>> {
//         const client = await this.pool.connect();
//         try {
//             const result = await client.query(sql, params);
//             for (const row of result.rows) {
//                 yield row;
//             }
//         }
//         finally {
//             client.release();
//         }
//     }
//
//     async muta(sql: string, params: unknown[]): Promise<number> {
//         const result = await this.pool.query(sql, params);
//         return result.rowCount ?? 0;
//     }
//
//     async incipe(): Promise<Transactio> {
//         const client = await this.pool.connect();
//         await client.query("BEGIN");
//         return new PgTransactio(client);
//     }
//
//     claude(): void {
//         this.pool.end();
//     }
// }
//
// class PgTransactio implements Transactio {
//     constructor(private client: pg.PoolClient) {}
//
//     async *lege(sql: string, params: unknown[]): AsyncIterable<Record<string, unknown>> {
//         const result = await this.client.query(sql, params);
//         for (const row of result.rows) {
//             yield row;
//         }
//     }
//
//     async muta(sql: string, params: unknown[]): Promise<number> {
//         const result = await this.client.query(sql, params);
//         return result.rowCount ?? 0;
//     }
//
//     async committe(): Promise<void> {
//         try {
//             await this.client.query("COMMIT");
//         }
//         finally {
//             this.client.release();
//         }
//     }
//
//     async reverte(): Promise<void> {
//         try {
//             await this.client.query("ROLLBACK");
//         }
//         finally {
//             this.client.release();
//         }
//     }
// }

import { Database } from "bun:sqlite";
import type { SQLQueryBindings } from "bun:sqlite";

// =============================================================================
// SQLITE DRIVER (Bun native)
// =============================================================================

class SqliteConnexio implements Connexio {
    private db: Database;

    constructor(path: string) {
        this.db = new Database(path);
    }

    async *lege(sql: string, params: unknown[]): AsyncIterable<Record<string, unknown>> {
        const stmt = this.db.prepare(sql);
        // WHY: Bun.SQLite expects SQLQueryBindings[], but our interface uses unknown[] to match .fab definition
        const bindings = params as SQLQueryBindings[];
        for (const row of stmt.all(...bindings)) {
            yield row as Record<string, unknown>;
        }
    }

    async muta(sql: string, params: unknown[]): Promise<number> {
        const stmt = this.db.prepare(sql);
        // WHY: Bun.SQLite expects SQLQueryBindings[], but our interface uses unknown[] to match .fab definition
        const bindings = params as SQLQueryBindings[];
        const result = stmt.run(...bindings);
        return result.changes;
    }

    async incipe(): Promise<Transactio> {
        this.db.run("BEGIN");
        return new SqliteTransactio(this.db);
    }

    claude(): void {
        this.db.close();
    }
}

class SqliteTransactio implements Transactio {
    constructor(private db: Database) {}

    async *lege(sql: string, params: unknown[]): AsyncIterable<Record<string, unknown>> {
        const stmt = this.db.prepare(sql);
        // WHY: Bun.SQLite expects SQLQueryBindings[], but our interface uses unknown[] to match .fab definition
        const bindings = params as SQLQueryBindings[];
        for (const row of stmt.all(...bindings)) {
            yield row as Record<string, unknown>;
        }
    }

    async muta(sql: string, params: unknown[]): Promise<number> {
        const stmt = this.db.prepare(sql);
        // WHY: Bun.SQLite expects SQLQueryBindings[], but our interface uses unknown[] to match .fab definition
        const bindings = params as SQLQueryBindings[];
        const result = stmt.run(...bindings);
        return result.changes;
    }

    async committe(): Promise<void> {
        this.db.run("COMMIT");
    }

    async reverte(): Promise<void> {
        this.db.run("ROLLBACK");
    }
}

// =============================================================================
// FACTORY
// =============================================================================

export async function connecta(url: string): Promise<Connexio> {
    const scheme = url.split(":")[0];

    switch (scheme) {
        case "postgres":
        case "postgresql":
            throw new Error("PostgreSQL driver not yet implemented - install 'pg' package and uncomment PgConnexio");

        case "mysql":
            throw new Error("MySQL driver not yet implemented");

        case "sqlite":
            // sqlite:path or sqlite::memory:
            return new SqliteConnexio(url.replace("sqlite:", ""));

        case "file":
            // file:path
            return new SqliteConnexio(url.replace("file:", ""));

        default:
            throw new Error(`Unknown database scheme: ${scheme}`);
    }
}
