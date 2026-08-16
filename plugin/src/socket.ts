import { PROTOCOL_HEADER, serializeSnapshot, deserializeSnapshot, getSocketPath, Snapshot } from './protocol';
import net from 'net';

export interface SocketClient {
  send: (snapshot: Snapshot) => void;
  disconnect: () => void;
}

type OnConnectCallback = (sendSnapshot: (snapshot: Snapshot) => void) => void;

let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 5;
const BASE_RECONNECT_DELAY = 1000;
const MAX_RECONNECT_DELAY = 30000;

export function startSocketClient(socketPath: string, onConnect: OnConnectCallback): SocketClient {
  let socket: net.Socket | null = null;
  let isConnected = false;
  const sendQueue: Snapshot[] = [];

  function connect() {
    const newSocket = net.createConnection(socketPath);
    socket = newSocket;

    newSocket.on('connect', () => {
      console.log('[Vesktop Voice Overlay] Connected to overlay socket');
      isConnected = true;
      reconnectAttempts = 0;
      
      // Read version header from server
      let headerBuffer = '';
      const onData = (data: Buffer) => {
        headerBuffer += data.toString();
        if (headerBuffer.includes('\n')) {
          const header = headerBuffer.trim();
          if (header === `VESKTOP_VOICE_OVERLAY/1.0`) {
            console.log('[Vesktop Voice Overlay] Protocol version validated');
            newSocket.off('data', onData);
            setupDataHandler(newSocket);
            onConnect(sendSnapshot);
            flushQueue();
          } else {
            console.error('[Vesktop Voice Overlay] Invalid protocol header:', header);
            newSocket.destroy();
          }
        }
      };
      newSocket.on('data', onData);
    });

    newSocket.on('error', (err: Error) => {
      console.error('[Vesktop Voice Overlay] Socket error:', err.message);
      scheduleReconnect();
    });

    newSocket.on('close', () => {
      console.log('[Vesktop Voice Overlay] Socket closed');
      isConnected = false;
      scheduleReconnect();
    });
  }

  function setupDataHandler(sock: net.Socket) {
    let buffer = '';
    sock.on('data', (data: Buffer) => {
      buffer += data.toString();
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';
      
      for (const line of lines) {
        if (line.trim()) {
          const snapshot = deserializeSnapshot(line);
          if (snapshot) {
            // Handle incoming snapshots if needed (server -> client)
            console.log('[Vesktop Voice Overlay] Received:', snapshot);
          }
        }
      }
    });
  }

  function sendSnapshot(snapshot: Snapshot) {
    const line = serializeSnapshot(snapshot) + '\n';
    if (isConnected && socket?.writable) {
      socket.write(line);
    } else {
      sendQueue.push(snapshot);
      if (sendQueue.length > 100) {
        sendQueue.shift(); // Prevent unbounded growth
      }
    }
  }

  function flushQueue() {
    while (sendQueue.length > 0 && isConnected && socket?.writable) {
      const snapshot = sendQueue.shift()!;
      sendSnapshot(snapshot);
    }
  }

  function scheduleReconnect() {
    if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
      const delay = MAX_RECONNECT_DELAY;
      console.log(`[Vesktop Voice Overlay] Max retries reached, waiting ${delay}ms before retry`);
      setTimeout(() => {
        reconnectAttempts = 0;
        connect();
      }, delay);
    } else {
      const delay = BASE_RECONNECT_DELAY * Math.pow(2, reconnectAttempts);
      console.log(`[Vesktop Voice Overlay] Reconnecting in ${delay}ms (attempt ${reconnectAttempts + 1})`);
      setTimeout(() => {
        reconnectAttempts++;
        connect();
      }, delay);
    }
  }

  function disconnect() {
    socket?.destroy();
    socket = null;
    isConnected = false;
  }

  connect();

  return {
    send: sendSnapshot,
    disconnect
  };
}
