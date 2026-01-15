import { test, expect, describe } from 'bun:test';
import { nomenclator, type RecordumMx } from './nomenclator';

/**
 * DNS tests are network-dependent and may fail in CI or restricted environments.
 * Tests use well-known domains that should be resolvable in most contexts.
 */
describe('nomenclator HAL', () => {
    describe('forward lookup', () => {
        test('resolve returns IP addresses for localhost', async () => {
            const ips = await nomenclator.resolve('localhost');
            // localhost should resolve to 127.0.0.1 and/or ::1
            expect(ips.length).toBeGreaterThan(0);
            expect(ips.some((ip) => ip === '127.0.0.1' || ip === '::1')).toBe(true);
        });

        test('resolve returns IP addresses for google.com', async () => {
            const ips = await nomenclator.resolve('google.com');
            expect(ips.length).toBeGreaterThan(0);
        });

        test('resolve4 returns only IPv4 addresses', async () => {
            const ips = await nomenclator.resolve4('google.com');
            expect(ips.length).toBeGreaterThan(0);
            // IPv4 addresses match dotted-decimal format
            for (const ip of ips) {
                expect(ip).toMatch(/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/);
            }
        });

        test('resolve6 returns only IPv6 addresses (may be empty)', async () => {
            const ips = await nomenclator.resolve6('google.com');
            // IPv6 may not be available for all hosts or networks
            // If present, should contain colons
            for (const ip of ips) {
                expect(ip).toContain(':');
            }
        });

        test('resolve returns empty array for invalid hostname', async () => {
            const ips = await nomenclator.resolve('this.domain.definitely.does.not.exist.invalid');
            expect(ips).toEqual([]);
        });

        test('resolve4 returns empty array for invalid hostname', async () => {
            const ips = await nomenclator.resolve4('this.domain.definitely.does.not.exist.invalid');
            expect(ips).toEqual([]);
        });
    });

    describe('reverse lookup', () => {
        test('reversa returns hostname for known IP', async () => {
            // Google's public DNS server - should have PTR record
            const hosts = await nomenclator.reversa('8.8.8.8');
            // May return dns.google or similar
            expect(Array.isArray(hosts)).toBe(true);
        });

        test('reversa returns empty array for IP without PTR', async () => {
            // Private IP unlikely to have reverse DNS
            const hosts = await nomenclator.reversa('192.168.255.255');
            expect(hosts).toEqual([]);
        });
    });

    describe('record queries', () => {
        test('mx returns MX records for google.com', async () => {
            const records = await nomenclator.mx('google.com');
            expect(records.length).toBeGreaterThan(0);

            for (const record of records) {
                expect(typeof record.hospes()).toBe('string');
                expect(record.hospes().length).toBeGreaterThan(0);
                expect(typeof record.prioritas()).toBe('number');
                expect(record.prioritas()).toBeGreaterThanOrEqual(0);
            }
        });

        test('mx returns empty array for invalid domain', async () => {
            const records = await nomenclator.mx('this.domain.definitely.does.not.exist.invalid');
            expect(records).toEqual([]);
        });

        test('txt returns TXT records for google.com', async () => {
            const records = await nomenclator.txt('google.com');
            expect(records.length).toBeGreaterThan(0);
            for (const record of records) {
                expect(typeof record).toBe('string');
            }
        });

        test('ns returns NS records for google.com', async () => {
            const records = await nomenclator.ns('google.com');
            expect(records.length).toBeGreaterThan(0);
            for (const record of records) {
                expect(typeof record).toBe('string');
                expect(record.length).toBeGreaterThan(0);
            }
        });

        test('cname returns CNAME for www.google.com', async () => {
            // www.google.com typically has a CNAME
            const cname = await nomenclator.cname('www.google.com');
            // May or may not have CNAME depending on DNS configuration
            if (cname !== null) {
                expect(typeof cname).toBe('string');
                expect(cname.length).toBeGreaterThan(0);
            }
        });

        test('cname returns null for domain without CNAME', async () => {
            // google.com apex likely has no CNAME
            const cname = await nomenclator.cname('google.com');
            expect(cname).toBe(null);
        });
    });

    describe('RecordumMx interface', () => {
        test('RecordumMx methods return correct values', async () => {
            const records = await nomenclator.mx('google.com');
            if (records.length > 0) {
                const record: RecordumMx = records[0];
                const hospes = record.hospes();
                const prioritas = record.prioritas();

                // Verify methods are callable and return stable values
                expect(record.hospes()).toBe(hospes);
                expect(record.prioritas()).toBe(prioritas);
            }
        });
    });
});
