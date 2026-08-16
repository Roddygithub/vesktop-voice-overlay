import { describe, it, expect } from 'vitest';
import { serializeSnapshot, deserializeSnapshot, Snapshot, PROTOCOL_HEADER } from './protocol';

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

  it('round-trip: serialize -> deserialize preserves data', () => {
    const json = serializeSnapshot(sampleSnapshot);
    const parsed = deserializeSnapshot(json);
    
    expect(parsed).not.toBeNull();
    expect(parsed!.self.userId).toBe(sampleSnapshot.self.userId);
    expect(parsed!.self.speaking).toBe(sampleSnapshot.self.speaking);
    expect(parsed!.participants[0].volume).toBe(80);
  });
});
