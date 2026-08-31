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
  mute?: boolean;
  deaf?: boolean;
  speaking: boolean;
  volume?: number;
}

export type Snapshot = SnapshotV1;

export interface ClearMessage {
  type: "clear";
}

export const PROTOCOL_VERSION = 1;
export const PROTOCOL_HEADER = `VESKTOP_VOICE_OVERLAY/${PROTOCOL_VERSION}.0\n`;

export function serializeSnapshot(snapshot: Snapshot): string {
  return JSON.stringify(snapshot);
}

export function serializeClear(): string {
  return JSON.stringify({ type: "clear" } satisfies ClearMessage);
}

export function normalizeCoordinate(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(-32768, Math.min(32768, Math.trunc(value)));
}

export function positionForCustomCoordinateChange(position: string): string {
  return position === "custom" ? position : "custom";
}

export function speakingEventUserId(event: unknown): string | null {
  if (!event || typeof event !== "object") return null;
  const value = event as Record<string, unknown>;
  const userId = value.userId ?? value.user_id;
  return typeof userId === "string" && userId.length > 0 ? userId : null;
}

export function speakingEventState(event: unknown): boolean {
  if (!event || typeof event !== "object") return true;
  const value = event as Record<string, unknown>;
  const flags = value.speakingFlags ?? value.speaking_flags;
  if (typeof flags === "number") return flags !== 0;
  return value.speaking !== false && value.speaking !== 0;
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
