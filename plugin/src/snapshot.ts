import { Snapshot, ParticipantSelf, Participant } from "./protocol";

export interface VoiceChannelSnapshot {
    channelId: string;
    self: ParticipantSelf;
    participants: Participant[];
}

export function buildSnapshot(data: VoiceChannelSnapshot): Snapshot {
    return {
        version: 1,
        timestamp: Date.now(),
        self: data.self,
        participants: data.participants,
    };
}
