#!/usr/bin/env bun
/**
 * Mass-update loop syntax from `ex`/`de` to `itera ex`/`itera de`
 *
 * This script updates all .fab files in fons/rivus/ to use the new loop syntax
 * per issue #305.
 *
 * Usage: bun scripta/update-loop-syntax.ts [--dry-run]
 */

import { readdir, readFile, writeFile } from "fs/promises";
import { join, relative } from "path";

const DRY_RUN = process.argv.includes("--dry-run");
const ROOT = join(import.meta.dir, "..", "fons", "rivus");

interface Change {
  file: string;
  line: number;
  before: string;
  after: string;
}

async function* walkFab(dir: string): AsyncGenerator<string> {
  const entries = await readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walkFab(path);
    } else if (entry.name.endsWith(".fab")) {
      yield path;
    }
  }
}

function updateLine(line: string): string | null {
  // Match lines that start with optional whitespace, then 'ex ' or 'de '
  // but NOT preceded by 'importa ' or 'cede ' (imports and async iteration already handled)
  // Pattern: start of line, whitespace, then 'ex ' or 'de ' as a statement start

  // For 'ex' - avoid 'importa ex', 'cede ex', and already-updated 'itera ex'
  const exMatch = line.match(/^(\s*)ex\s+/);
  if (exMatch) {
    // Check it's not part of import (importa ex "...")
    // by looking at previous non-whitespace content - but we only have this line
    // So check if it's followed by a string (import) vs expression (loop)
    const afterEx = line.slice(exMatch[0].length);

    // If followed by quote, it's an import
    if (afterEx.startsWith('"') || afterEx.startsWith("'")) {
      return null;
    }

    // Otherwise it's a loop - prepend 'itera '
    return exMatch[1] + "itera ex " + afterEx;
  }

  // For 'de' - avoid 'cede de' and already-updated 'itera de'
  const deMatch = line.match(/^(\s*)de\s+/);
  if (deMatch) {
    const afterDe = line.slice(deMatch[0].length);
    return deMatch[1] + "itera de " + afterDe;
  }

  return null;
}

async function processFile(filePath: string): Promise<Change[]> {
  const content = await readFile(filePath, "utf-8");
  const lines = content.split("\n");
  const changes: Change[] = [];
  let modified = false;

  for (let i = 0; i < lines.length; i++) {
    const original = lines[i];
    const updated = updateLine(original);

    if (updated !== null) {
      changes.push({
        file: relative(ROOT, filePath),
        line: i + 1,
        before: original.trim(),
        after: updated.trim(),
      });
      lines[i] = updated;
      modified = true;
    }
  }

  if (modified && !DRY_RUN) {
    await writeFile(filePath, lines.join("\n"));
  }

  return changes;
}

async function main() {
  console.log(DRY_RUN ? "DRY RUN - no files will be modified\n" : "");

  let totalChanges = 0;
  let filesModified = 0;

  for await (const filePath of walkFab(ROOT)) {
    const changes = await processFile(filePath);

    if (changes.length > 0) {
      filesModified++;
      totalChanges += changes.length;

      console.log(`\n${relative(ROOT, filePath)}:`);
      for (const change of changes) {
        console.log(`  L${change.line}: ${change.before}`);
        console.log(`      -> ${change.after}`);
      }
    }
  }

  console.log(`\n${"=".repeat(60)}`);
  console.log(`Total: ${totalChanges} changes in ${filesModified} files`);

  if (DRY_RUN) {
    console.log("\nRun without --dry-run to apply changes");
  }
}

main().catch(console.error);
