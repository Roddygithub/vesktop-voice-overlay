import { findStoreLazy, findByPropsLazy } from "@webpack";
import { Snapshot, ParticipantSelf, Participant } from "./protocol";

const VoiceStateStore = findStoreLazy("VoiceStateStore");
const UserStore = findByPropsLazy("getUser", "getCurrentUser");

function getUser(userId: string) {
    try {
        return UserStore.getUser(userId);
    } catch {
        return null;
    }
}

function getAvatarUrl(userId: string, avatar: string | null): string {
    if (avatar) {
        const ext = avatar.startsWith("a_") ? "gif" : "png";
        return `https://cdn.discordapp.com/avatars/${userId}/${avatar}.${ext}?size=128`;
    }
    return "";
}

const speakingUsers = new Set<string>();

export function setSpeaking(userId: string, speaking: boolean) {
    if (speaking) {
        speakingUsers.add(userId);
    } else {
        speakingUsers.delete(userId);
    }
    console.info("[VVO] speaking set updated", {
        speaking,
        tracked: speakingUsers.has(userId),
    });
}

export function clearSpeaking() {
    speakingUsers.clear();
}

export function getCurrentUser() {
    try {
        return UserStore.getCurrentUser();
    } catch {
        return null;
    }
}

export function isInVoiceChannel(): boolean {
    try {
        return VoiceStateStore.isCurrentClientInVoiceChannel();
    } catch {
        return false;
    }
}

export function getCurrentVoiceChannelId(): string | null {
    try {
        const currentUser = getCurrentUser();
        if (!currentUser) return null;
        return VoiceStateStore.getVoiceStateForUser(currentUser.id)?.channelId ?? null;
    } catch {
        return null;
    }
}

export function getChannelSnapshot(): Snapshot | null {
    const currentUser = getCurrentUser();
    if (!currentUser) return null;

    const channelId = getCurrentVoiceChannelId();
    if (!channelId) return null;

    const voiceStates: Record<string, any> = VoiceStateStore.getVoiceStatesForChannel(channelId);
    if (!voiceStates || Object.keys(voiceStates).length === 0) return null;

    const selfVoiceState = voiceStates[currentUser.id];
    if (!selfVoiceState) return null;

    const participants: Participant[] = [];

    for (const userId of Object.keys(voiceStates)) {
        if (userId === currentUser.id) continue;

        const userVoiceState = voiceStates[userId];
        const user = getUser(userId);
        const username = user?.globalName ?? user?.username ?? "Unknown";
        const avatar = user?.avatar ?? null;

        participants.push({
            userId,
            username,
            avatarUrl: getAvatarUrl(userId, avatar),
            mute: userVoiceState.selfMute || userVoiceState.mute,
            deaf: userVoiceState.selfDeaf || userVoiceState.deaf,
            speaking: speakingUsers.has(userId),
        });
    }

    const self: ParticipantSelf = {
        userId: currentUser.id,
        username: currentUser.globalName ?? currentUser.username,
        avatarUrl: getAvatarUrl(currentUser.id, currentUser.avatar),
        mute: selfVoiceState.selfMute || selfVoiceState.mute,
        deaf: selfVoiceState.selfDeaf || selfVoiceState.deaf,
        speaking: speakingUsers.has(currentUser.id),
    };

    return {
        version: 1,
        timestamp: Date.now(),
        self,
        participants,
    };
}
