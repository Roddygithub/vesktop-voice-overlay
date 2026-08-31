import { describe, it, expect } from 'vitest';
import {
  serializeSnapshot,
  deserializeSnapshot,
  normalizeCoordinate,
  serializeClear,
  speakingEventState,
  speakingEventUserId,
  Snapshot,
  PROTOCOL_HEADER,
} from './protocol';

describe('Protocol v1', () => {
  const sampleSnapshot: Snapshot = {
    version: 1,
    timestamp: 1692000000000,
    self: {
      userId: '123456789012345678',
      username: 'Roddy',
      avatarUrl: 'https://cdn.discordapp.com/avatars/123/abc.png',
      mute: false,
      deaf: false,
      speaking: true,
    },
    participants: [
      {
        userId: '987654321098765432',
        username: 'Friend',
        avatarUrl: 'https://cdn.discordapp.com/avatars/987/def.png',
        speaking: false,
        volume: 80,
      },
    ],
  };

  it('serializes snapshot to JSON', () => {
    const json = serializeSnapshot(sampleSnapshot);
    expect(json).toContain('"version":1');
    expect(json).toContain('"username":"Roddy"');
    expect(json).toContain('"speaking":true');
  });

  it('deserializes valid JSON line', () => {
    const json = serializeSnapshot(sampleSnapshot);
    const parsed = deserializeSnapshot(json);
    expect(parsed).not.toBeNull();
    expect(parsed!.version).toBe(1);
    expect(parsed!.self.username).toBe('Roddy');
    expect(parsed!.participants).toHaveLength(1);
  });

  it('returns null for invalid JSON', () => {
    const parsed = deserializeSnapshot('not json');
    expect(parsed).toBeNull();
  });

  it('returns null for wrong version', () => {
    const json = '{"version":2,"timestamp":0,"self":{},"participants":[]}';
    const parsed = deserializeSnapshot(json);
    expect(parsed).toBeNull();
  });

  it('protocol header matches expected format', () => {
    expect(PROTOCOL_HEADER).toBe('VESKTOP_VOICE_OVERLAY/1.0\n');
  });

  it('serializes the authoritative clear message', () => {
    expect(serializeClear()).toBe('{"type":"clear"}');
  });

  it('normalizes custom coordinates to server-valid integers', () => {
    expect(normalizeCoordinate(12.9)).toBe(12);
    expect(normalizeCoordinate(50000)).toBe(32768);
    expect(normalizeCoordinate(-50000)).toBe(-32768);
    expect(normalizeCoordinate(Number.NaN)).toBe(0);
  });

  it('accepts camelCase and snake_case speaking event fields', () => {
    expect(speakingEventUserId({ userId: 'camel' })).toBe('camel');
    expect(speakingEventUserId({ user_id: 'snake' })).toBe('snake');
    expect(speakingEventUserId({ userId: 123 })).toBeNull();
    expect(speakingEventState({ speakingFlags: 0 })).toBe(false);
    expect(speakingEventState({ speaking_flags: 2 })).toBe(true);
    expect(speakingEventState({ speaking: false })).toBe(false);
  });

  it('round-trip: serialize -> deserialize preserves data', () => {
    const json = serializeSnapshot(sampleSnapshot);
    const parsed = deserializeSnapshot(json);
    
    expect(parsed).not.toBeNull();
    expect(parsed!.self.userId).toBe(sampleSnapshot.self.userId);
    expect(parsed!.self.speaking).toBe(sampleSnapshot.self.speaking);
    expect(parsed!.participants[0].volume).toBe(80);
  });
});
