import type { IpcMainInvokeEvent } from "electron";
import net from "node:net";
import { classifyMessage, reconnectDelayMs, ResendCache } from "./resendCache";

type SocketClient = {
    send: (data: string) => void;
    disconnect: () => void;
};

let activeClient: SocketClient | null = null;
const MAX_HEADER_BYTES = 128;
const MAX_PAYLOAD_BYTES = 64 * 1024;
const MAX_QUEUED_LINES = 100;
const CLEAR_LINE = '{"type":"clear"}';

export function getSocketPath(_: IpcMainInvokeEvent): string {
    const runtimeDir = process.env.XDG_RUNTIME_DIR;
    if (!runtimeDir) throw new Error("XDG_RUNTIME_DIR is required");
    return `${runtimeDir}/vesktop-voice-overlay.sock`;
}

export function startSocket(
    _: IpcMainInvokeEvent,
    socketPath: string
): void {
    activeClient?.disconnect();

    let socket: net.Socket | null = null;
    let isConnected = false;
    let reconnectAttempts = 0;
    let reconnectTimer: NodeJS.Timeout | null = null;
    let stopped = false;
    let handshakeComplete = false;
    let backpressured = false;
    const sendQueue: string[] = [];
    let queuedSettings: string | null = null;
    let queuedState: string | null = null;
    const resendCache = new ResendCache();

    function enqueue(line: string) {
        switch (classifyMessage(line)) {
            case "settings":
                queuedSettings = line;
                break;
            case "snapshot":
            case "clear":
                queuedState = line;
                break;
            case "other":
                sendQueue.push(line);
                break;
        }
        while (
            sendQueue.length
                + Number(queuedSettings !== null)
                + Number(queuedState !== null)
            > MAX_QUEUED_LINES
        ) {
            sendQueue.shift();
        }
    }

    function takeQueuedLine(): string | null {
        if (queuedSettings !== null) {
            const line = queuedSettings;
            queuedSettings = null;
            return line;
        }
        if (queuedState !== null) {
            const line = queuedState;
            queuedState = null;
            return line;
        }
        return sendQueue.shift() ?? null;
    }

    function connect() {
        if (stopped) return;

        const newSocket = net.createConnection(socketPath);
        socket = newSocket;
        handshakeComplete = false;
        backpressured = false;

        newSocket.on("connect", () => {
            if (socket !== newSocket || stopped) return;
            isConnected = true;

            let headerBuffer = "";
            const onData = (data: Buffer) => {
                if (socket !== newSocket || stopped) return;
                headerBuffer += data.toString();
                if (Buffer.byteLength(headerBuffer) > MAX_HEADER_BYTES) {
                    console.error("[Vesktop Voice Overlay] Protocol header is too large");
                    newSocket.destroy();
                    return;
                }

                const newline = headerBuffer.indexOf("\n");
                if (newline !== -1) {
                    const header = headerBuffer.slice(0, newline).replace(/\r$/, "");
                    if (header === "VESKTOP_VOICE_OVERLAY/1.0") {
                        newSocket.off("data", onData);
                        handshakeComplete = true;
                        reconnectAttempts = 0;
                        // Restore the overlay's state immediately after any
                        // restart so no voice activity is needed to repopulate.
                        for (const line of resendCache.resendLines()) {
                            enqueue(line);
                        }
                        flushQueue();
                    } else {
                        console.error("[Vesktop Voice Overlay] Invalid protocol header");
                        newSocket.destroy();
                    }
                }
            };
            newSocket.on("data", onData);
        });

        newSocket.on("drain", () => {
            if (socket !== newSocket || stopped) return;
            backpressured = false;
            flushQueue();
        });

        newSocket.on("error", (err: Error) => {
            if (socket !== newSocket || stopped) return;
            if (reconnectAttempts === 0 || reconnectAttempts % 15 === 0) {
                console.warn("[Vesktop Voice Overlay] Socket unavailable:", err.message);
            }
            scheduleReconnect();
        });

        newSocket.on("close", () => {
            if (socket !== newSocket) return;
            socket = null;
            isConnected = false;
            handshakeComplete = false;
            backpressured = false;
            scheduleReconnect();
        });
    }

    function send(line: string) {
        if (Buffer.byteLength(line) > MAX_PAYLOAD_BYTES) {
            console.error("[Vesktop Voice Overlay] Refusing oversized outgoing payload");
            if (classifyMessage(line) !== "snapshot") return;
            line = CLEAR_LINE;
        }
        resendCache.record(line);
        if (!isConnected || !handshakeComplete || !socket?.writable) {
            // Settings and voice state are authoritative snapshots. Replaying
            // only their latest cached values avoids a stale-event burst after
            // a long disconnect; preserve unknown future message kinds.
            if (classifyMessage(line) === "other") enqueue(line);
            return;
        }
        if (isConnected && handshakeComplete && !backpressured && socket?.writable) {
            backpressured = !socket.write(line + "\n");
        } else {
            enqueue(line);
        }
    }

    function flushQueue() {
        while (
            isConnected
            && handshakeComplete
            && !backpressured
            && socket?.writable
        ) {
            const line = takeQueuedLine();
            if (line === null) break;
            backpressured = !socket.write(line + "\n");
        }
    }

    function scheduleReconnect() {
        if (stopped || reconnectTimer) return;

        const delay = reconnectDelayMs(reconnectAttempts);
        reconnectTimer = setTimeout(() => {
            reconnectTimer = null;
            reconnectAttempts++;
            connect();
        }, delay);
    }

    function disconnect() {
        stopped = true;
        if (reconnectTimer) clearTimeout(reconnectTimer);
        reconnectTimer = null;
        socket?.destroy();
        socket = null;
        isConnected = false;
        handshakeComplete = false;
        backpressured = false;
    }

    connect();
    activeClient = { send, disconnect };
}

export function send(_: IpcMainInvokeEvent, data: string): void {
    activeClient?.send(data);
}

export function disconnect(_: IpcMainInvokeEvent): void {
    activeClient?.disconnect();
    activeClient = null;
}
