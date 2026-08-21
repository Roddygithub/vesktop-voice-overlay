import { definePluginSettings } from "@api/Settings";
import definePlugin, { OptionType, PluginNative } from "@utils/types";
import { serializeSnapshot } from "./protocol";
import { getChannelSnapshot, setSpeaking, clearSpeaking, getCurrentVoiceChannelId, isInVoiceChannel } from "./voiceState";

type NativeModule = PluginNative<typeof import("./native")>;
let Native: NativeModule;
let socketClient: { send: (data: string) => void; disconnect: () => void } | null = null;
let currentChannelId: string | null = null;

function sendSettings() {
    if (!socketClient) return;

    socketClient.send(JSON.stringify({
        type: "settings",
        settings: {
            enabled: settings.store.enabled,
            position: settings.store.position,
            custom_x: settings.store.customX,
            custom_y: settings.store.customY,
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
    if (!isInVoiceChannel()) return;

    const snapshot = getChannelSnapshot();
    if (snapshot) {
        socketClient.send(serializeSnapshot(snapshot));
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
        Native = VencordNative.pluginHelpers
            .VesktopVoiceOverlay as NativeModule;

        void (async () => {
            const socketPath = await Native.getSocketPath();
            await Native.startSocket(socketPath);
            socketClient = {
                send: data => void Native.send(data),
                disconnect: () => void Native.disconnect(),
            };

            sendSettings();
            currentChannelId = getCurrentVoiceChannelId();
            sendSnapshot();
        })();

        currentChannelId = getCurrentVoiceChannelId();
    },

    stop() {
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
            if (event?.userId) {
                const speaking = event.speakingFlags !== undefined
                    ? event.speakingFlags !== 0
                    : event.speaking !== false && event.speaking !== 0;
                setSpeaking(event.userId, speaking);
                sendSnapshot();
            }
        },

        STOP_SPEAKING(event: any) {
            if (event?.userId) {
                setSpeaking(event.userId, false);
                sendSnapshot();
            }
        },
    },
});
