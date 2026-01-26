/**
 * crypta.ts - Cryptography Implementation
 *
 * Native TypeScript implementation of the HAL cryptography interface.
 * Uses Node's crypto module for all operations.
 *
 * Verbs:
 *   - digere: hash (to digest)
 *   - authenfica: HMAC (to authenticate)
 *   - cela/revela: encrypt/decrypt (to hide/reveal)
 *   - signa/verifica: asymmetric signatures
 *   - genera/generaPar: key generation
 *   - derivabit: key derivation (async - intentionally slow)
 */

import * as crypto from 'node:crypto';

// =========================================================================
// TYPES
// =========================================================================

type HashAlgorithm = 'sha256' | 'sha384' | 'sha512' | 'md5' | 'blake2b512';
type HmacAlgorithm = 'hmac-sha256' | 'hmac-sha384' | 'hmac-sha512';
type CipherAlgorithm = 'aes-256-gcm' | 'aes-256-cbc';
type KdfAlgorithm = 'pbkdf2' | 'scrypt' | 'argon2id';
type KeyAlgorithm = 'aes-256';
type KeyPairAlgorithm = 'ed25519' | 'rsa-2048' | 'rsa-4096';

// =========================================================================
// PARCLAVIUM CLASS
// =========================================================================

export class ParClavium {
    private _publicKey: Uint8Array;
    private _privateKey: Uint8Array;
    private _algorithmus: string;

    constructor(publicKey: Uint8Array, privateKey: Uint8Array, algorithmus: string) {
        this._publicKey = publicKey;
        this._privateKey = privateKey;
        this._algorithmus = algorithmus;
    }

    publica(): Uint8Array {
        return this._publicKey;
    }

    privata(): Uint8Array {
        return this._privateKey;
    }

    algorithmus(): string {
        return this._algorithmus;
    }
}

// =========================================================================
// INTERNAL HELPERS
// =========================================================================

function normalizeHashAlgo(algo: string): string {
    if (algo === 'blake2b') return 'blake2b512';
    if (algo.startsWith('hmac-')) return algo.slice(5); // hmac-sha256 -> sha256
    return algo;
}

function getIvLength(algo: CipherAlgorithm): number {
    switch (algo) {
        case 'aes-256-gcm': return 12;
        case 'aes-256-cbc': return 16;
        default: throw new Error(`Unknown cipher algorithm: ${algo}`);
    }
}

const AUTH_TAG_LENGTH = 16;

// =========================================================================
// CRYPTA OBJECT
// =========================================================================

export const crypta = {
    // =========================================================================
    // HASHING
    // =========================================================================

    digere(alg: string, data: Uint8Array): Uint8Array {
        const hash = crypto.createHash(normalizeHashAlgo(alg));
        hash.update(data);
        return new Uint8Array(hash.digest());
    },

    // =========================================================================
    // MESSAGE AUTHENTICATION (HMAC)
    // =========================================================================

    authenfica(alg: string, clavis: Uint8Array, data: Uint8Array): Uint8Array {
        const hmac = crypto.createHmac(normalizeHashAlgo(alg), clavis);
        hmac.update(data);
        return new Uint8Array(hmac.digest());
    },

    // =========================================================================
    // SYMMETRIC ENCRYPTION
    // =========================================================================

    cela(alg: CipherAlgorithm, clavis: Uint8Array, data: Uint8Array): Uint8Array {
        const ivLength = getIvLength(alg);
        const iv = crypto.randomBytes(ivLength);

        if (alg === 'aes-256-gcm') {
            const cipher = crypto.createCipheriv('aes-256-gcm', clavis, iv);
            const encrypted = Buffer.concat([cipher.update(data), cipher.final()]);
            const authTag = cipher.getAuthTag();
            const result = new Uint8Array(iv.length + authTag.length + encrypted.length);
            result.set(iv, 0);
            result.set(authTag, iv.length);
            result.set(encrypted, iv.length + authTag.length);
            return result;
        }
        else {
            const cipher = crypto.createCipheriv('aes-256-cbc', clavis, iv);
            const encrypted = Buffer.concat([cipher.update(data), cipher.final()]);
            const result = new Uint8Array(iv.length + encrypted.length);
            result.set(iv, 0);
            result.set(encrypted, iv.length);
            return result;
        }
    },

    revela(alg: CipherAlgorithm, clavis: Uint8Array, data: Uint8Array): Uint8Array {
        const ivLength = getIvLength(alg);

        if (alg === 'aes-256-gcm') {
            const iv = data.slice(0, ivLength);
            const authTag = data.slice(ivLength, ivLength + AUTH_TAG_LENGTH);
            const ciphertext = data.slice(ivLength + AUTH_TAG_LENGTH);

            const decipher = crypto.createDecipheriv('aes-256-gcm', clavis, iv);
            decipher.setAuthTag(authTag);
            const decrypted = Buffer.concat([decipher.update(ciphertext), decipher.final()]);
            return new Uint8Array(decrypted);
        }
        else {
            const iv = data.slice(0, ivLength);
            const ciphertext = data.slice(ivLength);

            const decipher = crypto.createDecipheriv('aes-256-cbc', clavis, iv);
            const decrypted = Buffer.concat([decipher.update(ciphertext), decipher.final()]);
            return new Uint8Array(decrypted);
        }
    },

    // =========================================================================
    // ASYMMETRIC SIGNATURES
    // =========================================================================

    signa(clavisPrivata: Uint8Array, data: Uint8Array): Uint8Array {
        const keyObject = crypto.createPrivateKey({
            key: Buffer.from(clavisPrivata),
            format: 'der',
            type: 'pkcs8',
        });

        const signature = crypto.sign(null, data, keyObject);
        return new Uint8Array(signature);
    },

    verifica(clavisPublica: Uint8Array, data: Uint8Array, signatura: Uint8Array): boolean {
        const keyObject = crypto.createPublicKey({
            key: Buffer.from(clavisPublica),
            format: 'der',
            type: 'spki',
        });

        return crypto.verify(null, data, keyObject, signatura);
    },

    // =========================================================================
    // KEY GENERATION
    // =========================================================================

    genera(alg: KeyAlgorithm): Uint8Array {
        switch (alg) {
            case 'aes-256':
                return new Uint8Array(crypto.randomBytes(32));
            default:
                throw new Error(`Unknown key algorithm: ${alg}`);
        }
    },

    generaPar(alg: KeyPairAlgorithm): ParClavium {
        switch (alg) {
            case 'ed25519': {
                const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519', {
                    publicKeyEncoding: { type: 'spki', format: 'der' },
                    privateKeyEncoding: { type: 'pkcs8', format: 'der' },
                });
                return new ParClavium(new Uint8Array(publicKey), new Uint8Array(privateKey), 'ed25519');
            }

            case 'rsa-2048': {
                const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
                    modulusLength: 2048,
                    publicKeyEncoding: { type: 'spki', format: 'der' },
                    privateKeyEncoding: { type: 'pkcs8', format: 'der' },
                });
                return new ParClavium(new Uint8Array(publicKey), new Uint8Array(privateKey), 'rsa-2048');
            }

            case 'rsa-4096': {
                const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
                    modulusLength: 4096,
                    publicKeyEncoding: { type: 'spki', format: 'der' },
                    privateKeyEncoding: { type: 'pkcs8', format: 'der' },
                });
                return new ParClavium(new Uint8Array(publicKey), new Uint8Array(privateKey), 'rsa-4096');
            }

            default:
                throw new Error(`Unknown key pair algorithm: ${alg}`);
        }
    },

    // =========================================================================
    // KEY DERIVATION (async - intentionally slow)
    // =========================================================================

    async derivabit(alg: KdfAlgorithm, password: string, sal: Uint8Array, longitudo: number): Promise<Uint8Array> {
        switch (alg) {
            case 'pbkdf2':
                return new Promise((resolve, reject) => {
                    crypto.pbkdf2(password, sal, 100000, longitudo, 'sha256', (err, key) => {
                        if (err) reject(err);
                        else resolve(new Uint8Array(key));
                    });
                });

            case 'scrypt':
                return new Promise((resolve, reject) => {
                    crypto.scrypt(password, sal, longitudo, (err, key) => {
                        if (err) reject(err);
                        else resolve(new Uint8Array(key));
                    });
                });

            case 'argon2id':
                throw new Error('argon2id not yet supported in JS runtime');

            default:
                throw new Error(`Unknown KDF algorithm: ${alg}`);
        }
    },
};
