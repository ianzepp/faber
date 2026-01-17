/**
 * processus.ts - Process Device Implementation (Bun/Node)
 *
 * Native TypeScript implementation of the HAL process interface.
 */

export const processus = {
    // SPAWN - Portable array-based process spawning
    genera(args: string[], cwd?: string, env?: Record<string, string>): number {
        const proc = Bun.spawn(args, {
            cwd: cwd ?? process.cwd(),
            env: env ? { ...process.env, ...env } : process.env,
            stdout: "ignore",
            stderr: "ignore",
            stdin: "ignore",
        });
        proc.unref();
        return proc.pid;
    },

    // SPAWN - With options, returns process handle
    spawn(cmd: string, args: string[], options: Record<string, unknown>): { pid: number; exited: Promise<number> } {
        const proc = Bun.spawn([cmd, ...args], {
            stdout: (options.stdout as "inherit" | "pipe" | "ignore") ?? "inherit",
            stderr: (options.stderr as "inherit" | "pipe" | "ignore") ?? "inherit",
        });
        return { pid: proc.pid, exited: proc.exited };
    },

    // SHELL EXECUTION - For commands needing shell features (&&, |, >, etc)
    exsequi(cmd: string): string {
        const result = Bun.spawnSync(["sh", "-c", cmd]);
        return result.stdout.toString();
    },

    exsequiCodem(cmd: string): number {
        const result = Bun.spawnSync(["sh", "-c", cmd]);
        return result.exitCode;
    },

    // ENVIRONMENT
    env(nomen: string): string | null {
        return Bun.env[nomen] ?? null;
    },

    envVel(nomen: string, defVal: string): string {
        return Bun.env[nomen] ?? defVal;
    },

    poneEnv(nomen: string, valor: string): void {
        Bun.env[nomen] = valor;
    },

    // PROCESS INFO
    cwd(): string {
        return process.cwd();
    },

    chdir(via: string): void {
        process.chdir(via);
    },

    pid(): number {
        return process.pid;
    },

    argv(): string[] {
        return Bun.argv.slice(2);
    },

    // EXIT
    exi(code: number): never {
        process.exit(code);
    },
};
