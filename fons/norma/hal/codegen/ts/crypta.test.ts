import { test, expect, describe } from 'bun:test';
import { crypta, ParClavium } from './crypta';

// Helper for hex conversion in tests (encoding utilities removed from crypta)
function toHex(data: Uint8Array): string {
    return Buffer.from(data).toString('hex');
}

describe('crypta HAL', () => {
    describe('hashing (digere)', () => {
        test('digere produces expected SHA-256 hash', () => {
            const data = new TextEncoder().encode('hello');
            const hash = crypta.digere('sha256', data);
            expect(toHex(hash)).toBe('2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824');
        });

        test('digere supports multiple algorithms', () => {
            const data = new TextEncoder().encode('test');

            const sha384 = crypta.digere('sha384', data);
            expect(toHex(sha384).length).toBe(96);

            const sha512 = crypta.digere('sha512', data);
            expect(toHex(sha512).length).toBe(128);

            const md5 = crypta.digere('md5', data);
            expect(toHex(md5).length).toBe(32);

            const blake2b = crypta.digere('blake2b512', data);
            expect(toHex(blake2b).length).toBe(128);
        });
    });

    describe('message authentication (authenfica)', () => {
        test('authenfica produces expected HMAC output', () => {
            const key = new TextEncoder().encode('secret');
            const data = new TextEncoder().encode('hello');
            const mac = crypta.authenfica('hmac-sha256', key, data);
            expect(toHex(mac)).toBe('88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b');
        });

        test('different keys produce different MACs', () => {
            const data = new TextEncoder().encode('hello');
            const key1 = new TextEncoder().encode('secret1');
            const key2 = new TextEncoder().encode('secret2');

            const mac1 = crypta.authenfica('hmac-sha256', key1, data);
            const mac2 = crypta.authenfica('hmac-sha256', key2, data);

            expect(toHex(mac1)).not.toBe(toHex(mac2));
        });
    });

    describe('symmetric encryption (cela/revela)', () => {
        test('cela/revela roundtrip with AES-256-GCM', () => {
            const key = crypta.genera('aes-256');
            const plaintext = new TextEncoder().encode('Hello, World!');

            const ciphertext = crypta.cela('aes-256-gcm', key, plaintext);
            const decrypted = crypta.revela('aes-256-gcm', key, ciphertext);

            expect(new TextDecoder().decode(decrypted)).toBe('Hello, World!');
        });

        test('cela/revela roundtrip with AES-256-CBC', () => {
            const key = crypta.genera('aes-256');
            const plaintext = new TextEncoder().encode('Hello, World!');

            const ciphertext = crypta.cela('aes-256-cbc', key, plaintext);
            const decrypted = crypta.revela('aes-256-cbc', key, ciphertext);

            expect(new TextDecoder().decode(decrypted)).toBe('Hello, World!');
        });

        test('cela produces different ciphertext each time (random IV)', () => {
            const key = crypta.genera('aes-256');
            const plaintext = new TextEncoder().encode('Hello');

            const ct1 = crypta.cela('aes-256-gcm', key, plaintext);
            const ct2 = crypta.cela('aes-256-gcm', key, plaintext);

            expect(toHex(ct1)).not.toBe(toHex(ct2));
        });

        test('revela with wrong key fails', () => {
            const key1 = crypta.genera('aes-256');
            const key2 = crypta.genera('aes-256');
            const plaintext = new TextEncoder().encode('Secret');

            const ciphertext = crypta.cela('aes-256-gcm', key1, plaintext);

            expect(() => crypta.revela('aes-256-gcm', key2, ciphertext)).toThrow();
        });
    });

    describe('asymmetric signatures (signa/verifica)', () => {
        test('signa/verifica roundtrip with ed25519', () => {
            const pair = crypta.generaPar('ed25519');
            const data = new TextEncoder().encode('Sign this message');

            const signature = crypta.signa(pair.privata(), data);
            const valid = crypta.verifica(pair.publica(), data, signature);

            expect(valid).toBe(true);
        });

        test('verifica fails with wrong data', () => {
            const pair = crypta.generaPar('ed25519');
            const data = new TextEncoder().encode('Original message');
            const wrongData = new TextEncoder().encode('Wrong message');

            const signature = crypta.signa(pair.privata(), data);
            const valid = crypta.verifica(pair.publica(), wrongData, signature);

            expect(valid).toBe(false);
        });

        test('verifica fails with wrong public key', () => {
            const pair1 = crypta.generaPar('ed25519');
            const pair2 = crypta.generaPar('ed25519');
            const data = new TextEncoder().encode('Test message');

            const signature = crypta.signa(pair1.privata(), data);
            const valid = crypta.verifica(pair2.publica(), data, signature);

            expect(valid).toBe(false);
        });

        test('signa/verifica roundtrip with RSA', () => {
            const pair = crypta.generaPar('rsa-2048');
            const data = new TextEncoder().encode('RSA signature test');

            const signature = crypta.signa(pair.privata(), data);
            const valid = crypta.verifica(pair.publica(), data, signature);

            expect(valid).toBe(true);
        });
    });

    describe('key generation (genera/generaPar)', () => {
        test('genera produces 32-byte AES key', () => {
            const key = crypta.genera('aes-256');
            expect(key.length).toBe(32);
        });

        test('genera produces unique keys', () => {
            const key1 = crypta.genera('aes-256');
            const key2 = crypta.genera('aes-256');
            expect(toHex(key1)).not.toBe(toHex(key2));
        });

        test('generaPar produces ed25519 key pair', () => {
            const pair = crypta.generaPar('ed25519');

            expect(pair).toBeInstanceOf(ParClavium);
            expect(pair.algorithmus()).toBe('ed25519');
            expect(pair.publica().length).toBeGreaterThan(0);
            expect(pair.privata().length).toBeGreaterThan(0);
        });

        test('generaPar produces rsa-2048 key pair', () => {
            const pair = crypta.generaPar('rsa-2048');

            expect(pair.algorithmus()).toBe('rsa-2048');
            expect(pair.publica().length).toBeGreaterThan(0);
            expect(pair.privata().length).toBeGreaterThan(0);
        });

        test('generaPar produces rsa-4096 key pair', () => {
            const pair = crypta.generaPar('rsa-4096');

            expect(pair.algorithmus()).toBe('rsa-4096');
            expect(pair.publica().length).toBeGreaterThan(0);
            expect(pair.privata().length).toBeGreaterThan(0);
        });
    });

    describe('key derivation (derivabit)', () => {
        test('pbkdf2 produces consistent output', async () => {
            const salt = new TextEncoder().encode('salty');
            const key1 = await crypta.derivabit('pbkdf2', 'password', salt, 32);
            const key2 = await crypta.derivabit('pbkdf2', 'password', salt, 32);

            expect(toHex(key1)).toBe(toHex(key2));
        });

        test('pbkdf2 different passwords produce different keys', async () => {
            const salt = new TextEncoder().encode('salty');
            const key1 = await crypta.derivabit('pbkdf2', 'password1', salt, 32);
            const key2 = await crypta.derivabit('pbkdf2', 'password2', salt, 32);

            expect(toHex(key1)).not.toBe(toHex(key2));
        });

        test('scrypt produces consistent output', async () => {
            const salt = new TextEncoder().encode('salty');
            const key1 = await crypta.derivabit('scrypt', 'password', salt, 32);
            const key2 = await crypta.derivabit('scrypt', 'password', salt, 32);

            expect(toHex(key1)).toBe(toHex(key2));
        });

        test('argon2id throws not supported error', async () => {
            const salt = new TextEncoder().encode('salty');
            await expect(crypta.derivabit('argon2id', 'password', salt, 32)).rejects.toThrow('not yet supported');
        });
    });

    describe('ParClavium class', () => {
        test('stores and retrieves key components', () => {
            const pair = crypta.generaPar('ed25519');

            expect(pair.publica()).toBeInstanceOf(Uint8Array);
            expect(pair.privata()).toBeInstanceOf(Uint8Array);
            expect(pair.algorithmus()).toBe('ed25519');
        });

        test('public and private keys are different', () => {
            const pair = crypta.generaPar('ed25519');
            expect(toHex(pair.publica())).not.toBe(toHex(pair.privata()));
        });
    });
});
