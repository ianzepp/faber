// Systema - File System Intrinsics for Module Resolution
//
// Provides file system operations that the Faber-compiled module resolution
// code needs. This file is manually maintained since extern declarations
// in Faber only generate TypeScript `declare` statements.

import { readFileSync, existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

// Read file contents as UTF-8 string
export function _readFileSync(via: string): string {
    return readFileSync(via, 'utf-8');
}

// Check if file exists
export function _existsSync(via: string): boolean {
    return existsSync(via);
}

// Get parent directory of a path
export function _dirname(via: string): string {
    return dirname(via);
}

// Resolve relative path against base
export function _resolve(basis: string, relativum: string): string {
    return resolve(basis, relativum);
}
