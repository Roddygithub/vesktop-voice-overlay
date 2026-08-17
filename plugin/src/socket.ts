import { Snapshot } from './protocol';
import { startNativeSocketClient, NativeSocketClient } from './native';

export interface SocketClient {
  send: (snapshot: Snapshot) => void;
  disconnect: () => void;
}

type OnConnectCallback = (sendSnapshot: (snapshot: Snapshot) => void) => void;

export function startSocketClient(socketPath: string, onConnect: OnConnectCallback): SocketClient {
  const nativeClient = startNativeSocketClient(socketPath, onConnect);
  return {
    send: nativeClient.send,
    disconnect: nativeClient.disconnect
  };
}