/**
 * processus.ts - Process Device Implementation
 *
 * Native TypeScript implementation of the HAL process interface.
 * Uses Bun process APIs.
 *
 * Spawn semantics encoded via different verbs:
 *   - genera: spawn attached, caller manages lifecycle
 *   - dimitte: spawn detached, dismiss to run independently
 */

/** Spawned subprocess handle for attached processes */
export interface Subprocessus {
    pid: number;
    expiravit: Promise<number>;  // resolves to exit code
}

export const processus = {
    // =========================================================================
    // SPAWN - Attached
    // =========================================================================
    // Verb: genera from "generare" (to generate, beget)

    /** Spawn attached process - caller can wait for exit via handle.expiravit */
    genera(
        argumenta: string[],
        directorium?: string,
        ambitus?: Record<string, string>
    ): Subprocessus {
        const proc = Bun.spawn(argumenta, {
            cwd: directorium ?? process.cwd(),
            env: ambitus ? { ...process.env, ...ambitus } : process.env,
            stdout: "inherit",
            stderr: "inherit",
            stdin: "inherit",
        });
        return {
            pid: proc.pid,
            expiravit: proc.exited,
        };
    },

    // =========================================================================
    // SPAWN - Detached
    // =========================================================================
    // Verb: dimitte from "dimittere" (to send away, dismiss)

    /** Dismiss process to run independently - returns PID */
    dimitte(
        argumenta: string[],
        directorium?: string,
        ambitus?: Record<string, string>
    ): number {
        const proc = Bun.spawn(argumenta, {
            cwd: directorium ?? process.cwd(),
            env: ambitus ? { ...process.env, ...ambitus } : process.env,
            stdout: "ignore",
            stderr: "ignore",
            stdin: "ignore",
        });
        proc.unref();
        return proc.pid;
    },

    // =========================================================================
    // SHELL EXECUTION
    // =========================================================================
    // Verb: exsequi/exsequetur from "exsequi" (to execute, accomplish)

    /** Execute shell command, block until complete, return stdout (sync) */
    exsequi(imperium: string): string {
        const result = Bun.spawnSync(["sh", "-c", imperium]);
        return result.stdout.toString();
    },

    /** Execute shell command, return stdout when complete (async) */
    async exsequetur(imperium: string): Promise<string> {
        const proc = Bun.spawn(["sh", "-c", imperium], {
            stdout: "pipe",
            stderr: "pipe",
        });
        const output = await new Response(proc.stdout).text();
        await proc.exited;
        return output;
    },

    // =========================================================================
    // ENVIRONMENT - Read/Write
    // =========================================================================
    // Verbs: lege/scribe (read/write)

    /** Read environment variable (returns null if not set) */
    lege(nomen: string): string | null {
        return Bun.env[nomen] ?? null;
    },

    /** Write environment variable */
    scribe(nomen: string, valor: string): void {
        Bun.env[nomen] = valor;
    },

    // =========================================================================
    // PROCESS INFO
    // =========================================================================

    /** Get current working directory (where the process dwells) */
    sedes(): string {
        return process.cwd();
    },

    /** Change current working directory */
    muta(via: string): void {
        process.chdir(via);
    },

    /** Get process ID */
    identitas(): number {
        return process.pid;
    },

    /** Get command line arguments (excludes runtime and script path) */
    argumenta(): string[] {
        return Bun.argv.slice(2);
    },

    // =========================================================================
    // EXIT
    // =========================================================================
    // Verb: exi from "exire" (to exit, depart)

    /** Exit process with code (never returns) */
    exi(code: number): never {
        process.exit(code);
    },
};
