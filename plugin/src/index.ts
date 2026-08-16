import { startSocketClient } from './socket';
import { buildSnapshot } from './snapshot';
import { getSocketPath } from './protocol';
import { useVoiceState, useCurrentUser, subscribeToVoiceStateChanges, setVencordApi, getVencordApi } from './voiceState';

let socketClient: ReturnType<typeof startSocketClient> | null = null;
let unsubscribeVoiceState: (() => void) | null = null;

// Vencord plugin entry point
export const plugin = {
  name: 'Vesktop Voice Overlay',
  description: 'Wayland-native voice activity overlay for Vesktop',
  version: '0.1.0',
  author: 'Roddy',
  
  start: (api: any) => {
    console.log('[Vesktop Voice Overlay] Starting plugin...');
    
    // Store Vencord API for later use
    setVencordApi(api);
    
    const socketPath = getSocketPath();
    socketClient = startSocketClient(socketPath, () => {
      // Called when connected and ready to send
      sendCurrentSnapshot();
    });

    // Subscribe to voice state changes
    unsubscribeVoiceState = subscribeToVoiceStateChanges((voiceState) => {
      const currentUser = useCurrentUser();
      if (currentUser) {
        const snapshot = buildSnapshot(voiceState, currentUser);
        if (snapshot) {
          sendSnapshot(snapshot);
        }
      }
    });

    // Send initial snapshot if already in voice channel
    sendCurrentSnapshot();
  },

  stop: () => {
    console.log('[Vesktop Voice Overlay] Stopping plugin...');
    unsubscribeVoiceState?.();
    unsubscribeVoiceState = null;
    socketClient?.disconnect();
    socketClient = null;
  }
};

function sendCurrentSnapshot() {
  const voiceState = useVoiceState();
  const currentUser = useCurrentUser();
  
  if (voiceState && currentUser) {
    const snapshot = buildSnapshot(voiceState, currentUser);
    if (snapshot) {
      sendSnapshot(snapshot);
    }
  }
}

function sendSnapshot(snapshot: any) {
  if (socketClient) {
    socketClient.send(snapshot);
  }
}

// Default export for Vencord
export default plugin;
