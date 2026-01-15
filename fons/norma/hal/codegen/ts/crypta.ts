/**
 * crypta.ts - Cryptography Implementation
 *
 * Native TypeScript implementation of the HAL cryptography interface.
 * Uses Node's crypto module for all operations.
 */

import * as crypto from 'node:crypto';

// =========================================================================
// TYPES
// =========================================================================

type HashAlgorithm = 'sha256' | 'sha384' | 'sha512' | 'md5' | 'blake2b512';
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
    // Node crypto uses 'blake2b512' for BLAKE2b
    if (algo === 'blake2b') return 'blake2b512';
    return algo;
}

function getIvLength(algo: CipherAlgorithm): number {
    switch (algo) {
        case 'aes-256-gcm':
            return 12; // GCM recommends 12 bytes
        case 'aes-256-cbc':
            return 16; // CBC uses 16 bytes
        default:
            throw new Error(`Unknown cipher algorithm: ${algo}`);
    }
}

const AUTH_TAG_LENGTH = 16; // GCM auth tag is 16 bytes

// =========================================================================
// CRYPTA OBJECT
// =========================================================================

export const crypta = {
    // =========================================================================
    // HASHING
    // =========================================================================

    async digere(algorithmus: string, data: Uint8Array): Promise<Uint8Array> {
        const hash = crypto.createHash(normalizeHashAlgo(algorithmus));
        hash.update(data);
        return new Uint8Array(hash.digest());
    },

    async digereTextum(algorithmus: string, data: string): Promise<Uint8Array> {
        const hash = crypto.createHash(normalizeHashAlgo(algorithmus));
        hash.update(data, 'utf8');
        return new Uint8Array(hash.digest());
    },

    async digereHex(algorithmus: string, data: Uint8Array): Promise<string> {
        const hash = crypto.createHash(normalizeHashAlgo(algorithmus));
        hash.update(data);
        return hash.digest('hex');
    },

    // =========================================================================
    // HMAC
    // =========================================================================

    async hmac(algorithmus: string, clavis: Uint8Array, data: Uint8Array): Promise<Uint8Array> {
        const hmac = crypto.createHmac(normalizeHashAlgo(algorithmus), clavis);
        hmac.update(data);
        return new Uint8Array(hmac.digest());
    },

    async hmacHex(algorithmus: string, clavis: Uint8Array, data: Uint8Array): Promise<string> {
        const hmac = crypto.createHmac(normalizeHashAlgo(algorithmus), clavis);
        hmac.update(data);
        return hmac.digest('hex');
    },

    // =========================================================================
    // SYMMETRIC ENCRYPTION
    // =========================================================================

    async encripta(algorithmus: CipherAlgorithm, clavis: Uint8Array, data: Uint8Array): Promise<Uint8Array> {
        const ivLength = getIvLength(algorithmus);
        const iv = crypto.randomBytes(ivLength);

        if (algorithmus === 'aes-256-gcm') {
            const cipher = crypto.createCipheriv('aes-256-gcm', clavis, iv);
            const encrypted = Buffer.concat([cipher.update(data), cipher.final()]);
            const authTag = cipher.getAuthTag();
            // Format: IV + authTag + ciphertext
            const result = new Uint8Array(iv.length + authTag.length + encrypted.length);
            result.set(iv, 0);
            result.set(authTag, iv.length);
            result.set(encrypted, iv.length + authTag.length);
            return result;
        }
        else {
            // aes-256-cbc
            const cipher = crypto.createCipheriv('aes-256-cbc', clavis, iv);
            const encrypted = Buffer.concat([cipher.update(data), cipher.final()]);
            // Format: IV + ciphertext
            const result = new Uint8Array(iv.length + encrypted.length);
            result.set(iv, 0);
            result.set(encrypted, iv.length);
            return result;
        }
    },

    async decripta(algorithmus: CipherAlgorithm, clavis: Uint8Array, data: Uint8Array): Promise<Uint8Array> {
        const ivLength = getIvLength(algorithmus);

        if (algorithmus === 'aes-256-gcm') {
            const iv = data.slice(0, ivLength);
            const authTag = data.slice(ivLength, ivLength + AUTH_TAG_LENGTH);
            const ciphertext = data.slice(ivLength + AUTH_TAG_LENGTH);

            const decipher = crypto.createDecipheriv('aes-256-gcm', clavis, iv);
            decipher.setAuthTag(authTag);
            const decrypted = Buffer.concat([decipher.update(ciphertext), decipher.final()]);
            return new Uint8Array(decrypted);
        }
        else {
            // aes-256-cbc
            const iv = data.slice(0, ivLength);
            const ciphertext = data.slice(ivLength);

            const decipher = crypto.createDecipheriv('aes-256-cbc', clavis, iv);
            const decrypted = Buffer.concat([decipher.update(ciphertext), decipher.final()]);
            return new Uint8Array(decrypted);
        }
    },

    // =========================================================================
    // KEY DERIVATION
    // =========================================================================

    async derivaClavem(
        algorithmus: KdfAlgorithm,
        password: string,
        sal: Uint8Array,
        longitudo: number,
    ): Promise<Uint8Array> {
        switch (algorithmus) {
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
                throw new Error('argon2id not supported: requires argon2 package');

            default:
                throw new Error(`Unknown KDF algorithm: ${algorithmus}`);
        }
    },

    // =========================================================================
    // KEY GENERATION
    // =========================================================================

    async generaClavem(algorithmus: KeyAlgorithm): Promise<Uint8Array> {
        switch (algorithmus) {
            case 'aes-256':
                return new Uint8Array(crypto.randomBytes(32));

            default:
                throw new Error(`Unknown key algorithm: ${algorithmus}`);
        }
    },

    async generaParClavium(algorithmus: KeyPairAlgorithm): Promise<ParClavium> {
        switch (algorithmus) {
            case 'ed25519': {
                const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519', {
                    publicKeyEncoding: { type: 'spki', format: 'der' },
                    privateKeyEncoding: { type: 'pkcs8', format: 'der' },
                });
                return new ParClavium(
                    new Uint8Array(publicKey),
                    new Uint8Array(privateKey),
                    'ed25519',
                );
            }

            case 'rsa-2048': {
                const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
                    modulusLength: 2048,
                    publicKeyEncoding: { type: 'spki', format: 'der' },
                    privateKeyEncoding: { type: 'pkcs8', format: 'der' },
                });
                return new ParClavium(
                    new Uint8Array(publicKey),
                    new Uint8Array(privateKey),
                    'rsa-2048',
                );
            }

            case 'rsa-4096': {
                const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
                    modulusLength: 4096,
                    publicKeyEncoding: { type: 'spki', format: 'der' },
                    privateKeyEncoding: { type: 'pkcs8', format: 'der' },
                });
                return new ParClavium(
                    new Uint8Array(publicKey),
                    new Uint8Array(privateKey),
                    'rsa-4096',
                );
            }

            default:
                throw new Error(`Unknown key pair algorithm: ${algorithmus}`);
        }
    },

    // =========================================================================
    // ASYMMETRIC SIGNATURES
    // =========================================================================

    async signa(clavisPrivata: Uint8Array, data: Uint8Array): Promise<Uint8Array> {
        // Create key object from DER-encoded private key
        const keyObject = crypto.createPrivateKey({
            key: Buffer.from(clavisPrivata),
            format: 'der',
            type: 'pkcs8',
        });

        const signature = crypto.sign(null, data, keyObject);
        return new Uint8Array(signature);
    },

    async verifica(clavisPublica: Uint8Array, data: Uint8Array, signatura: Uint8Array): Promise<boolean> {
        // Create key object from DER-encoded public key
        const keyObject = crypto.createPublicKey({
            key: Buffer.from(clavisPublica),
            format: 'der',
            type: 'spki',
        });

        return crypto.verify(null, data, keyObject, signatura);
    },

    // =========================================================================
    // ENCODING UTILITIES
    // =========================================================================

    hexCodifica(data: Uint8Array): string {
        return Buffer.from(data).toString('hex');
    },

    hexDecodifica(hex: string): Uint8Array {
        return new Uint8Array(Buffer.from(hex, 'hex'));
    },

    base64Codifica(data: Uint8Array): string {
        return Buffer.from(data).toString('base64');
    },

    base64Decodifica(b64: string): Uint8Array {
        return new Uint8Array(Buffer.from(b64, 'base64'));
    },
};
