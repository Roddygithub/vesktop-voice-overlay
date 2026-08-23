import { IpcMainInvokeEvent } from "electron";
import net from "node:net";
import { reconnectDelayMs, ResendCache } from "./resendCache";

type SocketClient = {
    send: (data: string) => void;
    disconnect: () => void;
};

let activeClient: SocketClient | null = null;

export function getSocketPath(_: IpcMainInvokeEvent): string {
    const runtimeDir = process.env.XDG_RUNTIME_DIR || `/tmp/vesktop-voice-overlay-${process.getuid()}`;
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
    const sendQueue: string[] = [];
    const resendCache = new ResendCache();

    function connect() {
        if (stopped) return;

        const newSocket = net.createConnection(socketPath);
        socket = newSocket;

        newSocket.on("connect", () => {
            isConnected = true;
            reconnectAttempts = 0;

            let headerBuffer = "";
            const onData = (data: Buffer) => {
                headerBuffer += data.toString();
                if (headerBuffer.includes("\n")) {
                    const header = headerBuffer.trim();
                    if (header === "VESKTOP_VOICE_OVERLAY/1.0") {
                        newSocket.off("data", onData);
                        newSocket.on("data", () => {});
                        flushQueue();
                        // Restore the overlay's state immediately after any
                        // restart so no voice activity is needed to repopulate.
                        for (const line of resendCache.resendLines()) {
                            socket?.write(line + "\n");
                        }
                    } else {
                        console.error("[Vesktop Voice Overlay] Invalid protocol header:", header);
                        newSocket.destroy();
                    }
                }
            };
            newSocket.on("data", onData);
        });

        newSocket.on("error", (err: Error) => {
            console.error("[Vesktop Voice Overlay] Socket error:", err.message);
            scheduleReconnect();
        });

        newSocket.on("close", () => {
            isConnected = false;
            scheduleReconnect();
        });
    }

    function send(line: string) {
        resendCache.record(line);
        if (isConnected && socket?.writable) {
            socket.write(line + "\n");
        } else {
            sendQueue.push(line);
            if (sendQueue.length > 100) {
                sendQueue.shift();
            }
        }
    }

    function flushQueue() {
        while (sendQueue.length > 0 && isConnected && socket?.writable) {
            const line = sendQueue.shift()!;
            socket.write(line + "\n");
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
