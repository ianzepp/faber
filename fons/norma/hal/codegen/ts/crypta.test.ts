import { test, expect, describe } from 'bun:test';
import { crypta, ParClavium } from './crypta';

describe('crypta HAL', () => {
    describe('hashing', () => {
        test('digere produces expected SHA-256 hash', async () => {
            const data = new TextEncoder().encode('hello');
            const hash = await crypta.digere('sha256', data);
            const hex = crypta.hexCodifica(hash);
            // Known SHA-256 of "hello"
            expect(hex).toBe('2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824');
        });

        test('digereTextum hashes text directly', async () => {
            const hash = await crypta.digereTextum('sha256', 'hello');
            const hex = crypta.hexCodifica(hash);
            expect(hex).toBe('2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824');
        });

        test('digereHex returns hex string', async () => {
            const data = new TextEncoder().encode('hello');
            const hex = await crypta.digereHex('sha256', data);
            expect(hex).toBe('2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824');
        });

        test('digere supports multiple algorithms', async () => {
            const data = new TextEncoder().encode('test');

            // SHA-384
            const sha384 = await crypta.digereHex('sha384', data);
            expect(sha384.length).toBe(96); // 384 bits = 48 bytes = 96 hex chars

            // SHA-512
            const sha512 = await crypta.digereHex('sha512', data);
            expect(sha512.length).toBe(128); // 512 bits = 64 bytes = 128 hex chars

            // MD5
            const md5 = await crypta.digereHex('md5', data);
            expect(md5.length).toBe(32); // 128 bits = 16 bytes = 32 hex chars

            // BLAKE2b-512
            const blake2b = await crypta.digereHex('blake2b512', data);
            expect(blake2b.length).toBe(128); // 512 bits = 64 bytes = 128 hex chars
        });
    });

    describe('hmac', () => {
        test('hmac produces expected output', async () => {
            const key = new TextEncoder().encode('secret');
            const data = new TextEncoder().encode('hello');
            const mac = await crypta.hmac('sha256', key, data);
            const hex = crypta.hexCodifica(mac);
            // Known HMAC-SHA256 of "hello" with key "secret"
            expect(hex).toBe('88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b');
        });

        test('hmacHex returns hex string', async () => {
            const key = new TextEncoder().encode('secret');
            const data = new TextEncoder().encode('hello');
            const hex = await crypta.hmacHex('sha256', key, data);
            expect(hex).toBe('88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b');
        });

        test('different keys produce different MACs', async () => {
            const data = new TextEncoder().encode('hello');
            const key1 = new TextEncoder().encode('secret1');
            const key2 = new TextEncoder().encode('secret2');

            const mac1 = await crypta.hmacHex('sha256', key1, data);
            const mac2 = await crypta.hmacHex('sha256', key2, data);

            expect(mac1).not.toBe(mac2);
        });
    });

    describe('symmetric encryption', () => {
        test('encrypt/decrypt roundtrip with AES-256-GCM', async () => {
            const key = await crypta.generaClavem('aes-256');
            const plaintext = new TextEncoder().encode('Hello, World!');

            const ciphertext = await crypta.encripta('aes-256-gcm', key, plaintext);
            const decrypted = await crypta.decripta('aes-256-gcm', key, ciphertext);

            expect(new TextDecoder().decode(decrypted)).toBe('Hello, World!');
        });

        test('encrypt/decrypt roundtrip with AES-256-CBC', async () => {
            const key = await crypta.generaClavem('aes-256');
            const plaintext = new TextEncoder().encode('Hello, World!');

            const ciphertext = await crypta.encripta('aes-256-cbc', key, plaintext);
            const decrypted = await crypta.decripta('aes-256-cbc', key, ciphertext);

            expect(new TextDecoder().decode(decrypted)).toBe('Hello, World!');
        });

        test('encryption produces different ciphertext each time (random IV)', async () => {
            const key = await crypta.generaClavem('aes-256');
            const plaintext = new TextEncoder().encode('Hello');

            const ct1 = await crypta.encripta('aes-256-gcm', key, plaintext);
            const ct2 = await crypta.encripta('aes-256-gcm', key, plaintext);

            // Ciphertexts should differ due to random IV
            expect(crypta.hexCodifica(ct1)).not.toBe(crypta.hexCodifica(ct2));
        });

        test('decryption with wrong key fails', async () => {
            const key1 = await crypta.generaClavem('aes-256');
            const key2 = await crypta.generaClavem('aes-256');
            const plaintext = new TextEncoder().encode('Secret');

            const ciphertext = await crypta.encripta('aes-256-gcm', key1, plaintext);

            // GCM will throw due to auth tag mismatch
            await expect(crypta.decripta('aes-256-gcm', key2, ciphertext)).rejects.toThrow();
        });
    });

    describe('key derivation', () => {
        test('pbkdf2 produces consistent output', async () => {
            const salt = new TextEncoder().encode('salty');
            const key1 = await crypta.derivaClavem('pbkdf2', 'password', salt, 32);
            const key2 = await crypta.derivaClavem('pbkdf2', 'password', salt, 32);

            expect(crypta.hexCodifica(key1)).toBe(crypta.hexCodifica(key2));
        });

        test('pbkdf2 different passwords produce different keys', async () => {
            const salt = new TextEncoder().encode('salty');
            const key1 = await crypta.derivaClavem('pbkdf2', 'password1', salt, 32);
            const key2 = await crypta.derivaClavem('pbkdf2', 'password2', salt, 32);

            expect(crypta.hexCodifica(key1)).not.toBe(crypta.hexCodifica(key2));
        });

        test('scrypt produces consistent output', async () => {
            const salt = new TextEncoder().encode('salty');
            const key1 = await crypta.derivaClavem('scrypt', 'password', salt, 32);
            const key2 = await crypta.derivaClavem('scrypt', 'password', salt, 32);

            expect(crypta.hexCodifica(key1)).toBe(crypta.hexCodifica(key2));
        });

        test('argon2id throws not supported error', async () => {
            const salt = new TextEncoder().encode('salty');
            await expect(crypta.derivaClavem('argon2id', 'password', salt, 32)).rejects.toThrow(
                'argon2id not supported',
            );
        });
    });

    describe('key generation', () => {
        test('generaClavem produces 32-byte AES key', async () => {
            const key = await crypta.generaClavem('aes-256');
            expect(key.length).toBe(32);
        });

        test('generaClavem produces unique keys', async () => {
            const key1 = await crypta.generaClavem('aes-256');
            const key2 = await crypta.generaClavem('aes-256');
            expect(crypta.hexCodifica(key1)).not.toBe(crypta.hexCodifica(key2));
        });

        test('generaParClavium produces ed25519 key pair', async () => {
            const pair = await crypta.generaParClavium('ed25519');

            expect(pair).toBeInstanceOf(ParClavium);
            expect(pair.algorithmus()).toBe('ed25519');
            expect(pair.publica().length).toBeGreaterThan(0);
            expect(pair.privata().length).toBeGreaterThan(0);
        });

        test('generaParClavium produces rsa-2048 key pair', async () => {
            const pair = await crypta.generaParClavium('rsa-2048');

            expect(pair.algorithmus()).toBe('rsa-2048');
            expect(pair.publica().length).toBeGreaterThan(0);
            expect(pair.privata().length).toBeGreaterThan(0);
        });

        // RSA-4096 test skipped in CI as it's slow
        test('generaParClavium produces rsa-4096 key pair', async () => {
            const pair = await crypta.generaParClavium('rsa-4096');

            expect(pair.algorithmus()).toBe('rsa-4096');
            expect(pair.publica().length).toBeGreaterThan(0);
            expect(pair.privata().length).toBeGreaterThan(0);
        });
    });

    describe('asymmetric signatures', () => {
        test('sign/verify roundtrip with ed25519', async () => {
            const pair = await crypta.generaParClavium('ed25519');
            const data = new TextEncoder().encode('Sign this message');

            const signature = await crypta.signa(pair.privata(), data);
            const valid = await crypta.verifica(pair.publica(), data, signature);

            expect(valid).toBe(true);
        });

        test('verify fails with wrong data', async () => {
            const pair = await crypta.generaParClavium('ed25519');
            const data = new TextEncoder().encode('Original message');
            const wrongData = new TextEncoder().encode('Wrong message');

            const signature = await crypta.signa(pair.privata(), data);
            const valid = await crypta.verifica(pair.publica(), wrongData, signature);

            expect(valid).toBe(false);
        });

        test('verify fails with wrong public key', async () => {
            const pair1 = await crypta.generaParClavium('ed25519');
            const pair2 = await crypta.generaParClavium('ed25519');
            const data = new TextEncoder().encode('Test message');

            const signature = await crypta.signa(pair1.privata(), data);
            const valid = await crypta.verifica(pair2.publica(), data, signature);

            expect(valid).toBe(false);
        });

        test('sign/verify roundtrip with RSA', async () => {
            const pair = await crypta.generaParClavium('rsa-2048');
            const data = new TextEncoder().encode('RSA signature test');

            const signature = await crypta.signa(pair.privata(), data);
            const valid = await crypta.verifica(pair.publica(), data, signature);

            expect(valid).toBe(true);
        });
    });

    describe('encoding utilities', () => {
        test('hex encoding roundtrip', () => {
            const original = new Uint8Array([0, 1, 127, 128, 255]);
            const hex = crypta.hexCodifica(original);
            const decoded = crypta.hexDecodifica(hex);

            expect(hex).toBe('00017f80ff');
            expect(Array.from(decoded)).toEqual(Array.from(original));
        });

        test('base64 encoding roundtrip', () => {
            const original = new Uint8Array([0, 1, 127, 128, 255]);
            const b64 = crypta.base64Codifica(original);
            const decoded = crypta.base64Decodifica(b64);

            expect(b64).toBe('AAF/gP8=');
            expect(Array.from(decoded)).toEqual(Array.from(original));
        });

        test('hex encoding of empty array', () => {
            const empty = new Uint8Array(0);
            const hex = crypta.hexCodifica(empty);
            expect(hex).toBe('');
            expect(crypta.hexDecodifica(hex).length).toBe(0);
        });

        test('base64 encoding of empty array', () => {
            const empty = new Uint8Array(0);
            const b64 = crypta.base64Codifica(empty);
            expect(b64).toBe('');
            expect(crypta.base64Decodifica(b64).length).toBe(0);
        });

        test('base64 handles UTF-8 text bytes', () => {
            const text = 'Hello, World!';
            const bytes = new TextEncoder().encode(text);
            const b64 = crypta.base64Codifica(bytes);
            const decoded = crypta.base64Decodifica(b64);
            const result = new TextDecoder().decode(decoded);

            expect(result).toBe(text);
        });
    });

    describe('ParClavium class', () => {
        test('stores and retrieves key components', async () => {
            const pair = await crypta.generaParClavium('ed25519');

            expect(pair.publica()).toBeInstanceOf(Uint8Array);
            expect(pair.privata()).toBeInstanceOf(Uint8Array);
            expect(pair.algorithmus()).toBe('ed25519');
        });

        test('public and private keys are different', async () => {
            const pair = await crypta.generaParClavium('ed25519');

            expect(crypta.hexCodifica(pair.publica())).not.toBe(crypta.hexCodifica(pair.privata()));
        });
    });
});
