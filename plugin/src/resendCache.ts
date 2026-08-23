export type MessageKind = "settings" | "snapshot" | "other";

/** Reconnect backoff: gentle doubling capped well below the old 30s pause
 * phase, so recovery after an overlay restart lands under ~2 seconds. */
export const RECONNECT_BASE_MS = 500;
export const RECONNECT_MAX_MS = 2000;

export function reconnectDelayMs(attempt: number): number {
    const doubled = RECONNECT_BASE_MS * Math.pow(2, Math.max(0, attempt));
    return Math.min(doubled, RECONNECT_MAX_MS);
}

export function classifyMessage(line: string): MessageKind {
    try {
        const parsed: unknown = JSON.parse(line);
        if (parsed && typeof parsed === "object") {
            const obj = parsed as Record<string, unknown>;
            if (obj.type === "settings") return "settings";
            if (obj.version === 1) return "snapshot";
        }
        return "other";
    } catch {
        return "other";
    }
}

/** Remembers the latest settings and voice snapshot lines so both can be
 * replayed immediately after any (re)connect without waiting for new voice
 * activity in Discord. */
export class ResendCache {
    private settingsLine: string | null = null;
    private snapshotLine: string | null = null;

    record(line: string): void {
        switch (classifyMessage(line)) {
            case "settings":
                this.settingsLine = line;
                break;
            case "snapshot":
                this.snapshotLine = line;
                break;
            case "other":
                break;
        }
    }

    /** Latest state to replay after a (re)connect, oldest first. */
    resendLines(): string[] {
        return [this.settingsLine, this.snapshotLine].filter(
            (line): line is string => line !== null,
        );
    }
}
