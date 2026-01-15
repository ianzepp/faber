/**
 * processus.ts - Process Device Implementation (Bun/Node)
 *
 * Native TypeScript implementation of the HAL process interface.
 */

export const processus = {
    // EXECUTION
    exsequi(cmd: string): string {
        const result = Bun.spawnSync(cmd.split(' '));
        return result.stdout.toString();
    },

    exsequiCodem(cmd: string): number {
        const result = Bun.spawnSync(cmd.split(' '));
        return result.exitCode;
    },

    genera(cmd: string, args: string[]): string {
        const result = Bun.spawnSync([cmd, ...args]);
        return result.stdout.toString();
    },

    generaCodem(cmd: string, args: string[]): number {
        const result = Bun.spawnSync([cmd, ...args]);
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
