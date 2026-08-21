import { IpcMainInvokeEvent } from "electron";
import net from "node:net";

const MAX_RECONNECT_ATTEMPTS = 5;
const BASE_RECONNECT_DELAY = 1000;
const MAX_RECONNECT_DELAY = 30000;

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

        if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
            reconnectTimer = setTimeout(() => {
                reconnectTimer = null;
                reconnectAttempts = 0;
                connect();
            }, MAX_RECONNECT_DELAY);
        } else {
            const delay = BASE_RECONNECT_DELAY * Math.pow(2, reconnectAttempts);
            reconnectTimer = setTimeout(() => {
                reconnectTimer = null;
                reconnectAttempts++;
                connect();
            }, delay);
        }
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
