/**
 * Test harness for cross-target codegen tests.
 *
 * Components:
 *   - schema.ts  - SQLite database schema and utilities
 *   - verify.ts  - Target language verification
 *   - runner.ts  - Main test runner (CLI)
 *   - report.ts  - Feature matrix generator (CLI)
 *
 * Usage:
 *   bun run fons/proba/harness/runner.ts      Run tests, save to SQLite
 *   bun run fons/proba/harness/report.ts      Generate feature matrix
 */

export * from './schema';
export * from './verify';
