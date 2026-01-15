/**
 * thesaurus.ts - In-Memory Cache and Pub/Sub Implementation
 *
 * Native TypeScript implementation of the HAL thesaurus interface.
 * Uses in-memory storage for dev/single-node use.
 */

interface CacheEntry {
    value: string;
    expiresAt: number | null; // null = no expiry
}

interface PendingMessage {
    topic: string;
    body: string;
    timestamp: number;
}

// Internal storage
const cache = new Map<string, CacheEntry>();
const subscriptions = new Set<SubscriptioImpl>();

// Cleanup interval for expired entries (runs every 60 seconds)
let cleanupInterval: ReturnType<typeof setInterval> | null = null;

function startCleanup(): void {
    if (cleanupInterval) return;
    cleanupInterval = setInterval(() => {
        const now = Date.now();
        for (const [key, entry] of cache) {
            if (entry.expiresAt !== null && entry.expiresAt <= now) {
                cache.delete(key);
            }
        }
    }, 60_000);
}

function isExpired(entry: CacheEntry): boolean {
    return entry.expiresAt !== null && entry.expiresAt <= Date.now();
}

function getValidEntry(key: string): CacheEntry | null {
    const entry = cache.get(key);
    if (!entry) return null;
    if (isExpired(entry)) {
        cache.delete(key);
        return null;
    }
    return entry;
}

/**
 * Convert glob pattern to regex.
 * * matches any chars, ? matches single char
 */
function globToRegex(pattern: string): RegExp {
    const escaped = pattern
        .replace(/[.+^${}()|[\]\\]/g, '\\$&')
        .replace(/\*/g, '.*')
        .replace(/\?/g, '.');
    return new RegExp(`^${escaped}$`);
}

/**
 * Check if topic matches subscription pattern.
 * * matches one segment, ** matches multiple segments
 */
function matchesPattern(topic: string, pattern: string): boolean {
    const topicParts = topic.split('/');
    const patternParts = pattern.split('/');

    let ti = 0;
    let pi = 0;

    while (ti < topicParts.length && pi < patternParts.length) {
        if (patternParts[pi] === '**') {
            // ** matches zero or more segments
            if (pi === patternParts.length - 1) {
                return true; // ** at end matches everything
            }
            // Try matching ** against varying numbers of segments
            for (let skip = 0; skip <= topicParts.length - ti; skip++) {
                if (matchesPattern(
                    topicParts.slice(ti + skip).join('/'),
                    patternParts.slice(pi + 1).join('/')
                )) {
                    return true;
                }
            }
            return false;
        }
        else if (patternParts[pi] === '*') {
            // * matches exactly one segment
            ti++;
            pi++;
        }
        else if (patternParts[pi] === topicParts[ti]) {
            ti++;
            pi++;
        }
        else {
            return false;
        }
    }

    // Handle trailing **
    if (pi < patternParts.length && patternParts[pi] === '**') {
        pi++;
    }

    return ti === topicParts.length && pi === patternParts.length;
}

/**
 * Pub/Sub message
 */
export class Nuntius {
    private _thema: string;
    private _corpus: string;
    private _tempus: number;

    constructor(topic: string, body: string, timestamp: number) {
        this._thema = topic;
        this._corpus = body;
        this._tempus = timestamp;
    }

    thema(): string {
        return this._thema;
    }

    corpus(): string {
        return this._corpus;
    }

    tempus(): number {
        return this._tempus;
    }
}

/**
 * Pub/Sub subscription
 */
export class Subscriptio {
    private patterns: string[];
    private queue: PendingMessage[] = [];
    private closed = false;
    private waiting: ((value: IteratorResult<Nuntius>) => void) | null = null;

    constructor(patterns: string[]) {
        this.patterns = patterns;
    }

    /** Internal: deliver message to this subscription */
    _deliver(topic: string, body: string, timestamp: number): boolean {
        if (this.closed) return false;

        for (const pattern of this.patterns) {
            if (matchesPattern(topic, pattern)) {
                const msg = { topic, body, timestamp };
                if (this.waiting) {
                    const resolve = this.waiting;
                    this.waiting = null;
                    resolve({ value: new Nuntius(msg.topic, msg.body, msg.timestamp), done: false });
                }
                else {
                    this.queue.push(msg);
                }
                return true;
            }
        }
        return false;
    }

    async *nuntii(): AsyncGenerator<Nuntius> {
        while (!this.closed) {
            if (this.queue.length > 0) {
                const msg = this.queue.shift()!;
                yield new Nuntius(msg.topic, msg.body, msg.timestamp);
            }
            else {
                const result = await new Promise<IteratorResult<Nuntius>>((resolve) => {
                    this.waiting = resolve;
                });
                if (result.done) break;
                yield result.value;
            }
        }
    }

    claude(): void {
        this.closed = true;
        subscriptions.delete(this as unknown as SubscriptioImpl);
        if (this.waiting) {
            this.waiting({ value: undefined as unknown as Nuntius, done: true });
            this.waiting = null;
        }
    }
}

// Type alias for internal use
type SubscriptioImpl = Subscriptio;

export const thesaurus = {
    // =========================================================================
    // KEY-VALUE CACHE
    // =========================================================================

    async cape(clavis: string): Promise<string | null> {
        const entry = getValidEntry(clavis);
        return entry ? entry.value : null;
    },

    async pone(clavis: string, valor: string, ttl?: number): Promise<void> {
        startCleanup();
        const expiresAt = ttl && ttl > 0 ? Date.now() + ttl * 1000 : null;
        cache.set(clavis, { value: valor, expiresAt });
    },

    async poneNovum(clavis: string, valor: string): Promise<boolean> {
        startCleanup();
        const existing = getValidEntry(clavis);
        if (existing) return false;
        cache.set(clavis, { value: valor, expiresAt: null });
        return true;
    },

    async dele(claves: string[]): Promise<number> {
        let count = 0;
        for (const key of claves) {
            if (cache.delete(key)) {
                count++;
            }
        }
        return count;
    },

    async exstat(clavis: string): Promise<boolean> {
        return getValidEntry(clavis) !== null;
    },

    async ttl(clavis: string): Promise<number> {
        const entry = cache.get(clavis);
        if (!entry) return -2;
        if (isExpired(entry)) {
            cache.delete(clavis);
            return -2;
        }
        if (entry.expiresAt === null) return -1;
        return Math.max(0, Math.ceil((entry.expiresAt - Date.now()) / 1000));
    },

    async expira(clavis: string, secundae: number): Promise<boolean> {
        const entry = getValidEntry(clavis);
        if (!entry) return false;
        entry.expiresAt = Date.now() + secundae * 1000;
        return true;
    },

    // =========================================================================
    // NUMERIC OPERATIONS
    // =========================================================================

    async incr(clavis: string): Promise<number> {
        return this.incrPer(clavis, 1);
    },

    async incrPer(clavis: string, quantum: number): Promise<number> {
        startCleanup();
        const entry = getValidEntry(clavis);
        let current = 0;
        let expiresAt: number | null = null;

        if (entry) {
            current = parseInt(entry.value, 10) || 0;
            expiresAt = entry.expiresAt;
        }

        const newValue = current + quantum;
        cache.set(clavis, { value: String(newValue), expiresAt });
        return newValue;
    },

    async decr(clavis: string): Promise<number> {
        return this.incrPer(clavis, -1);
    },

    // =========================================================================
    // KEY QUERIES
    // =========================================================================

    async claves(exemplar: string): Promise<string[]> {
        const regex = globToRegex(exemplar);
        const result: string[] = [];
        const now = Date.now();

        for (const [key, entry] of cache) {
            if (entry.expiresAt !== null && entry.expiresAt <= now) {
                cache.delete(key);
                continue;
            }
            if (regex.test(key)) {
                result.push(key);
            }
        }

        return result;
    },

    // =========================================================================
    // PUB/SUB
    // =========================================================================

    async publica(thema: string, nuntius: string): Promise<number> {
        const timestamp = Date.now();
        let count = 0;

        for (const sub of subscriptions) {
            if (sub._deliver(thema, nuntius, timestamp)) {
                count++;
            }
        }

        return count;
    },

    async subscribe(exemplaria: string[]): Promise<Subscriptio> {
        const sub = new Subscriptio(exemplaria);
        subscriptions.add(sub as unknown as SubscriptioImpl);
        return sub;
    },

    // =========================================================================
    // TEST HELPERS (not part of HAL spec)
    // =========================================================================

    /** Clear all cache entries and subscriptions - for testing only */
    _reset(): void {
        cache.clear();
        for (const sub of subscriptions) {
            sub.claude();
        }
        subscriptions.clear();
        if (cleanupInterval) {
            clearInterval(cleanupInterval);
            cleanupInterval = null;
        }
    },
};
