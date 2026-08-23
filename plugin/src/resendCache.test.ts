import { describe, it, expect } from 'vitest';
import {
    classifyMessage,
    reconnectDelayMs,
    ResendCache,
    RECONNECT_BASE_MS,
    RECONNECT_MAX_MS,
} from './resendCache';

const SETTINGS_LINE =
    '{"type":"settings","settings":{"enabled":true,"position":"top-right"}}';
const SNAPSHOT_LINE = '{"version":1,"timestamp":1,"self":{},"participants":[]}';

describe('classifyMessage', () => {
    it('classifies settings messages', () => {
        expect(classifyMessage(SETTINGS_LINE)).toBe('settings');
    });

    it('classifies v1 snapshots', () => {
        expect(classifyMessage(SNAPSHOT_LINE)).toBe('snapshot');
    });

    it('rejects invalid JSON and unknown payloads', () => {
        expect(classifyMessage('not json')).toBe('other');
        expect(classifyMessage('{"type":"unknown"}')).toBe('other');
        expect(classifyMessage('{"version":2}')).toBe('other');
    });
});

describe('reconnectDelayMs', () => {
    it('starts at the base delay and doubles', () => {
        expect(reconnectDelayMs(0)).toBe(RECONNECT_BASE_MS);
        expect(reconnectDelayMs(1)).toBe(RECONNECT_BASE_MS * 2);
        expect(reconnectDelayMs(2)).toBe(RECONNECT_BASE_MS * 4);
    });

    it('never exceeds the maximum delay', () => {
        expect(reconnectDelayMs(5)).toBe(RECONNECT_MAX_MS);
        expect(reconnectDelayMs(50)).toBe(RECONNECT_MAX_MS);
        expect(RECONNECT_MAX_MS).toBeLessThanOrEqual(2000);
    });
});

describe('ResendCache', () => {
    it('returns nothing before anything was recorded', () => {
        const cache = new ResendCache();
        expect(cache.resendLines()).toEqual([]);
    });

    it('keeps the latest settings and snapshot and replays settings first', () => {
        const cache = new ResendCache();
        cache.record(SNAPSHOT_LINE);
        cache.record(SETTINGS_LINE);

        const lines = cache.resendLines();
        expect(lines).toEqual([SETTINGS_LINE, SNAPSHOT_LINE]);
    });

    it('keeps only the newest line per kind', () => {
        const cache = new ResendCache();
        cache.record(SETTINGS_LINE);
        const newerSettings =
            '{"type":"settings","settings":{"enabled":false,"position":"custom"}}';
        cache.record(newerSettings);
        cache.record(SNAPSHOT_LINE);

        const lines = cache.resendLines();
        expect(lines).toHaveLength(2);
        expect(lines[0]).toBe(newerSettings);
        expect(lines[1]).toBe(SNAPSHOT_LINE);
    });
});
