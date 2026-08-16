export interface SnapshotV1 {
  version: 1;
  timestamp: number;
  self: ParticipantSelf;
  participants: Participant[];
}

export interface ParticipantSelf {
  userId: string;
  username: string;
  avatarUrl: string;
  mute: boolean;
  deaf: boolean;
  speaking: boolean;
}

export interface Participant {
  userId: string;
  username: string;
  avatarUrl: string;
  speaking: boolean;
  volume?: number;
}

export type Snapshot = SnapshotV1;

export const PROTOCOL_VERSION = 1;
export const PROTOCOL_HEADER = `VESKTOP_VOICE_OVERLAY/${PROTOCOL_VERSION}.0\n`;

export function serializeSnapshot(snapshot: Snapshot): string {
  return JSON.stringify(snapshot);
}

export function deserializeSnapshot(line: string): Snapshot | null {
  try {
    const parsed = JSON.parse(line);
    if (parsed.version === 1) {
      return parsed as SnapshotV1;
    }
    return null;
  } catch {
    return null;
  }
}

export function getSocketPath(): string {
  const runtimeDir = process.env.XDG_RUNTIME_DIR || `/tmp/vesktop-voice-overlay-${(process as any).getuid?.() || 1000}`;
  return `${runtimeDir}/vesktop-voice-overlay.sock`;
}
