import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import net from "node:net";
import { afterEach, describe, expect, it } from "vitest";
import { disconnect, getSocketPath, send, startSocket } from "./native";

const SETTINGS = '{"type":"settings","settings":{"enabled":true}}';
const SNAPSHOT = '{"version":1,"timestamp":1,"self":{},"participants":[]}';
const CLEAR = '{"type":"clear"}';

describe("native socket client", () => {
    let directory: string | null = null;
    let server: net.Server | null = null;
    const sockets = new Set<net.Socket>();

    afterEach(async () => {
        disconnect(undefined as never);
        for (const socket of sockets) socket.destroy();
        sockets.clear();
        if (server?.listening) {
            await new Promise<void>(resolve => server!.close(() => resolve()));
        }
        server = null;
        if (directory) await rm(directory, { recursive: true, force: true });
        directory = null;
    });

    async function listen(onConnection: (socket: net.Socket) => void): Promise<string> {
        directory = await mkdtemp(join(tmpdir(), "vvo-native-test-"));
        const socketPath = join(directory, "overlay.sock");
        server = net.createServer(socket => {
            sockets.add(socket);
            socket.on("close", () => sockets.delete(socket));
            onConnection(socket);
        });
        await new Promise<void>((resolve, reject) => {
            server!.once("error", reject);
            server!.listen(socketPath, resolve);
        });
        return socketPath;
    }

    it("fails closed without a private runtime directory", () => {
        const runtimeDir = process.env.XDG_RUNTIME_DIR;
        try {
            delete process.env.XDG_RUNTIME_DIR;
            expect(() => getSocketPath(undefined as never)).toThrow(
                "XDG_RUNTIME_DIR is required",
            );
        } finally {
            if (runtimeDir === undefined) delete process.env.XDG_RUNTIME_DIR;
            else process.env.XDG_RUNTIME_DIR = runtimeDir;
        }
    });

    it("waits for the header and replays only the latest authoritative state", async () => {
        const received = new Promise<string[]>((resolve, reject) => {
            void listen(socket => {
                let buffer = "";
                socket.on("data", data => {
                    buffer += data.toString();
                    const lines = buffer.trim().split("\n");
                    if (lines.length >= 2) {
                        resolve(lines);
                    }
                });
                socket.write("VESKTOP_VOICE_");
                setTimeout(() => socket.write("OVERLAY/1.0\n"), 10);
            }).then(socketPath => {
                startSocket(undefined as never, socketPath);
                send(undefined as never, SETTINGS);
                send(undefined as never, SNAPSHOT);
                send(undefined as never, CLEAR);
            }, reject);
        });

        await expect(received).resolves.toEqual([SETTINGS, CLEAR]);
    });

    it("preserves latest settings and state while the socket is backpressured", async () => {
        const probe = '{"type":"probe"}';
        const latestSnapshot = JSON.stringify({
            version: 1,
            timestamp: 100,
            self: {},
            participants: [],
        });
        let resumeSocket: net.Socket | null = null;
        let markPaused!: () => void;
        let finish!: (lines: string[]) => void;
        const paused = new Promise<void>(resolve => {
            markPaused = resolve;
        });
        const received = new Promise<string[]>(resolve => {
            finish = resolve;
        });
        const lines: string[] = [];
        let buffer = "";
        const socketPath = await listen(socket => {
            resumeSocket = socket;
            socket.on("data", data => {
                buffer += data.toString();
                let newline: number;
                while ((newline = buffer.indexOf("\n")) !== -1) {
                    const line = buffer.slice(0, newline);
                    buffer = buffer.slice(newline + 1);
                    lines.push(line);
                    if (line === probe) {
                        socket.pause();
                        markPaused();
                    }
                    if (line === latestSnapshot) finish([...lines]);
                }
            });
            socket.write("VESKTOP_VOICE_OVERLAY/1.0\n");
        });

        startSocket(undefined as never, socketPath);
        send(undefined as never, probe);
        await paused;

        const filler = JSON.stringify({ type: "other", padding: "x".repeat(60_000) });
        for (let index = 0; index < 8; index++) send(undefined as never, filler);
        send(undefined as never, SETTINGS);
        for (let timestamp = 0; timestamp <= 100; timestamp++) {
            send(undefined as never, JSON.stringify({
                version: 1,
                timestamp,
                self: {},
                participants: [],
            }));
        }
        resumeSocket!.resume();

        const flushed = await received;
        expect(flushed).toContain(SETTINGS);
        expect(flushed).toContain(latestSnapshot);
    });

    it("replaces an oversized authoritative snapshot with clear", async () => {
        let resolveReceived!: (line: string) => void;
        const received = new Promise<string>(resolve => {
            resolveReceived = resolve;
        });
        const socketPath = await listen(socket => {
            let buffer = "";
            socket.on("data", data => {
                buffer += data.toString();
                const newline = buffer.indexOf("\n");
                if (newline !== -1) resolveReceived(buffer.slice(0, newline));
            });
            socket.write("VESKTOP_VOICE_OVERLAY/1.0\n");
        });
        startSocket(undefined as never, socketPath);
        send(undefined as never, JSON.stringify({
            version: 1,
            timestamp: 1,
            self: { username: "x".repeat(70_000) },
            participants: [],
        }));

        await expect(received).resolves.toBe(CLEAR);
    });
});
