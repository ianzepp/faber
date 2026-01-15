import { test, expect, describe, beforeAll, afterAll } from 'bun:test';
import { caelum, Replicatio, Rogatio, Servitor } from './caelum';

describe('caelum HAL', () => {
    let server: Servitor;
    let baseUrl: string;

    // Track received requests for verification
    let lastRequest: {
        method: string;
        path: string;
        body: string;
        headers: Record<string, string>;
        params: Record<string, string | null>;
    } | null = null;

    beforeAll(async () => {
        server = await caelum.exspecta(0, (rogatio: Rogatio) => {
            // Store request info for test verification
            lastRequest = {
                method: rogatio.modus(),
                path: rogatio.via(),
                body: rogatio.corpus(),
                headers: rogatio.capita(),
                params: {
                    foo: rogatio.param('foo'),
                    bar: rogatio.param('bar'),
                },
            };

            // Route based on path
            const path = rogatio.via();

            if (path === '/echo') {
                return caelum.replicatioJson(200, {
                    method: rogatio.modus(),
                    body: rogatio.corpus(),
                });
            }

            if (path === '/json') {
                return caelum.replicatioJson(200, { message: 'hello', count: 42 });
            }

            if (path === '/text') {
                return caelum.replicatio(200, { 'Content-Type': 'text/plain' }, 'Hello, World!');
            }

            if (path === '/headers') {
                return caelum.replicatio(
                    200,
                    {
                        'X-Custom-Header': 'custom-value',
                        'Content-Type': 'text/plain',
                    },
                    'check headers'
                );
            }

            if (path === '/status/201') {
                return caelum.replicatio(201, {}, 'created');
            }

            if (path === '/status/404') {
                return caelum.replicatio(404, {}, 'not found');
            }

            if (path === '/status/500') {
                return caelum.replicatio(500, {}, 'server error');
            }

            if (path === '/redirect') {
                return caelum.redirectio('/target');
            }

            return caelum.replicatio(404, {}, 'Not Found');
        });

        baseUrl = `http://localhost:${server.portus()}`;
    });

    afterAll(() => {
        server.siste();
    });

    describe('Servitor', () => {
        test('portus returns the port number', () => {
            expect(server.portus()).toBeGreaterThan(0);
        });
    });

    describe('HTTP methods', () => {
        test('pete performs GET request', async () => {
            const response = await caelum.pete(`${baseUrl}/echo`);
            expect(response.status()).toBe(200);
            expect(lastRequest?.method).toBe('GET');
        });

        test('mitte performs POST request', async () => {
            const response = await caelum.mitte(`${baseUrl}/echo`, 'post body');
            expect(response.status()).toBe(200);
            expect(lastRequest?.method).toBe('POST');
            expect(lastRequest?.body).toBe('post body');
        });

        test('pone performs PUT request', async () => {
            const response = await caelum.pone(`${baseUrl}/echo`, 'put body');
            expect(response.status()).toBe(200);
            expect(lastRequest?.method).toBe('PUT');
            expect(lastRequest?.body).toBe('put body');
        });

        test('dele performs DELETE request', async () => {
            const response = await caelum.dele(`${baseUrl}/echo`);
            expect(response.status()).toBe(200);
            expect(lastRequest?.method).toBe('DELETE');
        });

        test('muta performs PATCH request', async () => {
            const response = await caelum.muta(`${baseUrl}/echo`, 'patch body');
            expect(response.status()).toBe(200);
            expect(lastRequest?.method).toBe('PATCH');
            expect(lastRequest?.body).toBe('patch body');
        });
    });

    describe('roga (generic request)', () => {
        test('sends custom method and headers', async () => {
            const response = await caelum.roga(
                'POST',
                `${baseUrl}/echo`,
                { 'X-Test-Header': 'test-value', 'Content-Type': 'application/json' },
                '{"test": true}'
            );
            expect(response.status()).toBe(200);
            expect(lastRequest?.method).toBe('POST');
            expect(lastRequest?.body).toBe('{"test": true}');
            expect(lastRequest?.headers['x-test-header']).toBe('test-value');
        });
    });

    describe('Replicatio', () => {
        test('status returns HTTP status code', async () => {
            const r201 = await caelum.pete(`${baseUrl}/status/201`);
            expect(r201.status()).toBe(201);

            const r404 = await caelum.pete(`${baseUrl}/status/404`);
            expect(r404.status()).toBe(404);

            const r500 = await caelum.pete(`${baseUrl}/status/500`);
            expect(r500.status()).toBe(500);
        });

        test('corpus returns body as text', async () => {
            const response = await caelum.pete(`${baseUrl}/text`);
            expect(response.corpus()).toBe('Hello, World!');
        });

        test('corpusJson parses JSON body', async () => {
            const response = await caelum.pete(`${baseUrl}/json`);
            const json = response.corpusJson() as { message: string; count: number };
            expect(json.message).toBe('hello');
            expect(json.count).toBe(42);
        });

        test('capita returns all headers', async () => {
            const response = await caelum.pete(`${baseUrl}/headers`);
            const headers = response.capita();
            expect(headers).toBeDefined();
            // Headers object should exist
            expect(typeof headers).toBe('object');
        });

        test('caput returns specific header (case-insensitive)', async () => {
            const response = await caelum.pete(`${baseUrl}/headers`);
            expect(response.caput('x-custom-header')).toBe('custom-value');
            expect(response.caput('X-Custom-Header')).toBe('custom-value');
            expect(response.caput('X-CUSTOM-HEADER')).toBe('custom-value');
            expect(response.caput('x-nonexistent')).toBe(null);
        });

        test('bene returns true for 2xx status codes', async () => {
            const r200 = await caelum.pete(`${baseUrl}/text`);
            expect(r200.bene()).toBe(true);

            const r201 = await caelum.pete(`${baseUrl}/status/201`);
            expect(r201.bene()).toBe(true);

            const r404 = await caelum.pete(`${baseUrl}/status/404`);
            expect(r404.bene()).toBe(false);

            const r500 = await caelum.pete(`${baseUrl}/status/500`);
            expect(r500.bene()).toBe(false);
        });
    });

    describe('Rogatio (server receives)', () => {
        test('modus returns HTTP method', async () => {
            await caelum.mitte(`${baseUrl}/echo`, 'test');
            expect(lastRequest?.method).toBe('POST');
        });

        test('via returns pathname', async () => {
            await caelum.pete(`${baseUrl}/echo`);
            expect(lastRequest?.path).toBe('/echo');
        });

        test('corpus returns request body', async () => {
            await caelum.mitte(`${baseUrl}/echo`, 'request body content');
            expect(lastRequest?.body).toBe('request body content');
        });

        test('param returns query parameters', async () => {
            await caelum.pete(`${baseUrl}/echo?foo=value1&bar=value2`);
            expect(lastRequest?.params.foo).toBe('value1');
            expect(lastRequest?.params.bar).toBe('value2');
        });

        test('param returns null for missing parameter', async () => {
            await caelum.pete(`${baseUrl}/echo?foo=value1`);
            expect(lastRequest?.params.foo).toBe('value1');
            expect(lastRequest?.params.bar).toBe(null);
        });

        test('caput returns request header', async () => {
            await caelum.roga('GET', `${baseUrl}/echo`, { 'X-Request-Header': 'req-value' }, '');
            expect(lastRequest?.headers['x-request-header']).toBe('req-value');
        });
    });

    describe('response builders', () => {
        test('replicatio creates response with status, headers, body', () => {
            const response = caelum.replicatio(
                201,
                { 'X-Test': 'value' },
                'body content'
            );
            expect(response.status()).toBe(201);
            expect(response.corpus()).toBe('body content');
            expect(response.caput('x-test')).toBe('value');
        });

        test('replicatioJson creates JSON response with correct content-type', () => {
            const response = caelum.replicatioJson(200, { key: 'value' });
            expect(response.status()).toBe(200);
            expect(response.caput('content-type')).toBe('application/json');
            expect(response.corpusJson()).toEqual({ key: 'value' });
        });

        test('redirectio creates redirect response', () => {
            const response = caelum.redirectio('/new-location');
            expect(response.status()).toBe(302);
            expect(response.caput('location')).toBe('/new-location');
        });
    });

    describe('roundtrip', () => {
        test('JSON roundtrip through server', async () => {
            const original = { name: 'test', values: [1, 2, 3], nested: { a: true } };
            const response = await caelum.mitte(`${baseUrl}/echo`, JSON.stringify(original));
            const received = response.corpusJson() as { body: string };
            expect(JSON.parse(received.body)).toEqual(original);
        });
    });
});
