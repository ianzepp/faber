/**
 * SQLite schema and database utilities for test harness.
 */

import { Database } from 'bun:sqlite';
import { existsSync, mkdirSync } from 'fs';
import { dirname } from 'path';

export interface TestRun {
    id: number;
    timestamp: string;
    git_sha: string;
    git_branch: string;
}

export type Compiler = 'faber' | 'rivus';

export interface TestResult {
    id: number;
    run_id: number;
    compiler: Compiler;
    feature: string;
    target: string;
    file: string;
    test_name: string;
    source: string;
    codegen: string | null;
    codegen_ok: boolean;
    verify_ok: boolean | null; // null if verification not attempted
    passed: boolean;
    error_msg: string | null;
}

const SCHEMA = `
CREATE TABLE IF NOT EXISTS test_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    git_sha TEXT NOT NULL,
    git_branch TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS test_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    compiler TEXT NOT NULL,
    feature TEXT NOT NULL,
    target TEXT NOT NULL,
    file TEXT NOT NULL,
    test_name TEXT NOT NULL,
    source TEXT NOT NULL,
    codegen TEXT,
    codegen_ok INTEGER NOT NULL,
    verify_ok INTEGER,
    passed INTEGER NOT NULL,
    error_msg TEXT,
    FOREIGN KEY (run_id) REFERENCES test_runs(id)
);

CREATE INDEX IF NOT EXISTS idx_results_run ON test_results(run_id);
CREATE INDEX IF NOT EXISTS idx_results_compiler ON test_results(compiler);
CREATE INDEX IF NOT EXISTS idx_results_feature ON test_results(feature);
CREATE INDEX IF NOT EXISTS idx_results_target ON test_results(target);
CREATE INDEX IF NOT EXISTS idx_results_passed ON test_results(passed);
`;

export function openDatabase(dbPath: string): Database {
    const dir = dirname(dbPath);
    if (!existsSync(dir)) {
        mkdirSync(dir, { recursive: true });
    }

    const db = new Database(dbPath);
    db.exec(SCHEMA);
    return db;
}

export function getGitInfo(): { sha: string; branch: string } {
    try {
        const sha = Bun.spawnSync(['git', 'rev-parse', 'HEAD']).stdout.toString().trim();
        const branch = Bun.spawnSync(['git', 'rev-parse', '--abbrev-ref', 'HEAD']).stdout.toString().trim();
        return { sha, branch };
    }
    catch {
        return { sha: 'unknown', branch: 'unknown' };
    }
}

export function createTestRun(db: Database): number {
    const { sha, branch } = getGitInfo();
    const timestamp = new Date().toISOString();

    const stmt = db.prepare(`
        INSERT INTO test_runs (timestamp, git_sha, git_branch)
        VALUES (?, ?, ?)
    `);

    stmt.run(timestamp, sha, branch);
    return db.query('SELECT last_insert_rowid() as id').get() as { id: number } | null
        ? (db.query('SELECT last_insert_rowid() as id').get() as { id: number }).id
        : 0;
}

export function insertResult(
    db: Database,
    runId: number,
    result: Omit<TestResult, 'id' | 'run_id'>
): void {
    const stmt = db.prepare(`
        INSERT INTO test_results (run_id, compiler, feature, target, file, test_name, source, codegen, codegen_ok, verify_ok, passed, error_msg)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);

    stmt.run(
        runId,
        result.compiler,
        result.feature,
        result.target,
        result.file,
        result.test_name,
        result.source,
        result.codegen,
        result.codegen_ok ? 1 : 0,
        result.verify_ok === null ? null : result.verify_ok ? 1 : 0,
        result.passed ? 1 : 0,
        result.error_msg
    );
}

export function getTestRun(db: Database, runId: number): TestRun | null {
    return db.query('SELECT * FROM test_runs WHERE id = ?').get(runId) as TestRun | null;
}

export function getLatestTestRun(db: Database): TestRun | null {
    return db.query('SELECT * FROM test_runs ORDER BY id DESC LIMIT 1').get() as TestRun | null;
}

export function getResultsForRun(db: Database, runId: number): TestResult[] {
    return db.query('SELECT * FROM test_results WHERE run_id = ?').all(runId) as TestResult[];
}

export interface FeatureSummary {
    feature: string;
    passed: number;
    total: number;
}

export function getFeatureSummary(db: Database, runId: number): FeatureSummary[] {
    const query = `
        SELECT
            feature,
            SUM(CASE WHEN passed = 1 THEN 1 ELSE 0 END) as passed,
            COUNT(*) as total
        FROM test_results
        WHERE run_id = ?
        GROUP BY feature
        ORDER BY feature
    `;

    return db.query(query).all(runId) as FeatureSummary[];
}
