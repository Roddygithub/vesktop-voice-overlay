import { definePluginSettings } from "@api/Settings";
import definePlugin, { OptionType, PluginNative } from "@utils/types";
import {
    normalizeCoordinate,
    serializeClear,
    serializeSnapshot,
    speakingEventState,
    speakingEventUserId,
} from "./protocol";
import { getChannelSnapshot, setSpeaking, clearSpeaking, getCurrentVoiceChannelId, isInVoiceChannel } from "./voiceState";

type NativeModule = PluginNative<typeof import("./native")>;
let Native: NativeModule;
let socketClient: { send: (data: string) => void; disconnect: () => void } | null = null;
let currentChannelId: string | null = null;
let lifecycleGeneration = 0;
let clearSent = false;

function sendSettings() {
    if (!socketClient) return;

    socketClient.send(JSON.stringify({
        type: "settings",
        settings: {
            enabled: settings.store.enabled,
            position: settings.store.position,
            custom_x: normalizeCoordinate(settings.store.customX),
            custom_y: normalizeCoordinate(settings.store.customY),
            user_display: settings.store.userDisplay,
            name_display: settings.store.nameDisplay,
            avatar_size_mode: settings.store.avatarSize,
        },
    }));
}

const settings = definePluginSettings({
    enabled: {
        type: OptionType.BOOLEAN,
        description: "Show the Voice Widget in games",
        default: true,
        onChange: sendSettings,
    },
    userDisplay: {
        type: OptionType.SELECT,
        description: "Choose which voice users are visible",
        options: [
            { label: "Speaking only", value: "speaking_only", default: true },
            { label: "Always", value: "always" },
        ],
        onChange: sendSettings,
    },
    nameDisplay: {
        type: OptionType.SELECT,
        description: "Choose when display names are visible",
        options: [
            { label: "Speaking only", value: "speaking_only", default: true },
            { label: "Always", value: "always" },
            { label: "Never", value: "never" },
        ],
        onChange: sendSettings,
    },
    avatarSize: {
        type: OptionType.SELECT,
        description: "Choose the avatar size",
        options: [
            { label: "Small", value: "small", default: true },
            { label: "Large", value: "large" },
        ],
        onChange: sendSettings,
    },
    position: {
        type: OptionType.SELECT,
        description: "Choose the Voice Widget position",
        options: [
            { label: "Top right", value: "top-right", default: true },
            { label: "Top left", value: "top-left" },
            { label: "Bottom right", value: "bottom-right" },
            { label: "Bottom left", value: "bottom-left" },
            { label: "Center", value: "center" },
            { label: "Custom coordinates", value: "custom" },
        ],
        onChange: sendSettings,
    },
    customX: {
        type: OptionType.NUMBER,
        description: "Custom horizontal offset from the left edge",
        default: 20,
        onChange: sendSettings,
    },
    customY: {
        type: OptionType.NUMBER,
        description: "Custom vertical offset from the top edge",
        default: 20,
        onChange: sendSettings,
    },
});

function sendSnapshot() {
    if (!socketClient) return;
    if (!isInVoiceChannel()) {
        if (!clearSent) socketClient.send(serializeClear());
        clearSent = true;
        return;
    }

    const snapshot = getChannelSnapshot();
    if (snapshot) {
        clearSent = false;
        socketClient.send(serializeSnapshot(snapshot));
    } else if (!clearSent) {
        socketClient.send(serializeClear());
        clearSent = true;
    }
}

function cleanupSpeakingForChannel() {
    const newChannelId = getCurrentVoiceChannelId();
    if (newChannelId !== currentChannelId) {
        clearSpeaking();
        currentChannelId = newChannelId;
    }
}

export default definePlugin({
    name: "VesktopVoiceOverlay",
    description: "Wayland-native voice activity overlay for Vesktop",
    authors: [{ name: "Roddy", id: 0n }],
    tags: ["Voice"],
    settings,

    start() {
        const generation = ++lifecycleGeneration;
        clearSent = false;
        Native = VencordNative.pluginHelpers
            .VesktopVoiceOverlay as NativeModule;

        void (async () => {
            const socketPath = await Native.getSocketPath();
            if (generation !== lifecycleGeneration) return;
            await Native.startSocket(socketPath);
            if (generation !== lifecycleGeneration) return;
            socketClient = {
                send: data => void Native.send(data),
                disconnect: () => void Native.disconnect(),
            };

            sendSettings();
            currentChannelId = getCurrentVoiceChannelId();
            sendSnapshot();
        })().catch(error => {
            if (generation !== lifecycleGeneration) return;
            console.error(
                "[Vesktop Voice Overlay] Failed to start socket client:",
                error instanceof Error ? error.message : String(error),
            );
        });

        currentChannelId = getCurrentVoiceChannelId();
    },

    stop() {
        lifecycleGeneration++;
        void Native?.disconnect();
        socketClient = null;
        clearSpeaking();
        currentChannelId = null;
    },

    flux: {
        VOICE_STATE_UPDATES() {
            cleanupSpeakingForChannel();
            sendSnapshot();
        },

        SPEAKING(event: any) {
            const userId = speakingEventUserId(event);
            if (userId && setSpeaking(userId, speakingEventState(event))) {
                sendSnapshot();
            }
        },

        STOP_SPEAKING(event: any) {
            const userId = speakingEventUserId(event);
            if (userId && setSpeaking(userId, false)) {
                sendSnapshot();
            }
        },
    },
});
