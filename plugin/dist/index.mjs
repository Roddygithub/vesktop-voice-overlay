/* Vesktop Voice Overlay Plugin - GPL-3.0 */
var __require = /* @__PURE__ */ ((x) => typeof require !== "undefined" ? require : typeof Proxy !== "undefined" ? new Proxy(x, {
  get: (a, b) => (typeof require !== "undefined" ? require : a)[b]
}) : x)(function(x) {
  if (typeof require !== "undefined") return require.apply(this, arguments);
  throw Error('Dynamic require of "' + x + '" is not supported');
});

// src/protocol.ts
var PROTOCOL_VERSION = 1;
var PROTOCOL_HEADER = `VESKTOP_VOICE_OVERLAY/${PROTOCOL_VERSION}.0
`;
function serializeSnapshot(snapshot) {
  return JSON.stringify(snapshot);
}
function deserializeSnapshot(line) {
  try {
    const parsed = JSON.parse(line);
    if (parsed.version === 1) {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
}
function getSocketPath() {
  const runtimeDir = process.env.XDG_RUNTIME_DIR || `/tmp/vesktop-voice-overlay-${process.getuid()}`;
  return `${runtimeDir}/vesktop-voice-overlay.sock`;
}

// src/socket.ts
var reconnectAttempts = 0;
var MAX_RECONNECT_ATTEMPTS = 5;
var BASE_RECONNECT_DELAY = 1e3;
var MAX_RECONNECT_DELAY = 3e4;
function startSocketClient(socketPath, onConnect) {
  let socket = null;
  let isConnected = false;
  let sendQueue = [];
  function connect() {
    const net = __require("net");
    socket = net.createConnection(socketPath);
    socket.on("connect", () => {
      console.log("[Vesktop Voice Overlay] Connected to overlay socket");
      isConnected = true;
      reconnectAttempts = 0;
      let headerBuffer = "";
      socket.on("data", (data) => {
        headerBuffer += data.toString();
        if (headerBuffer.includes("\n")) {
          const header = headerBuffer.trim();
          if (header === `VESKTOP_VOICE_OVERLAY/1.0`) {
            console.log("[Vesktop Voice Overlay] Protocol version validated");
            socket.off("data", arguments.callee);
            setupDataHandler();
            onConnect(sendSnapshot2);
            flushQueue();
          } else {
            console.error("[Vesktop Voice Overlay] Invalid protocol header:", header);
            socket.destroy();
          }
        }
      });
    });
    socket.on("error", (err) => {
      console.error("[Vesktop Voice Overlay] Socket error:", err.message);
      scheduleReconnect();
    });
    socket.on("close", () => {
      console.log("[Vesktop Voice Overlay] Socket closed");
      isConnected = false;
      scheduleReconnect();
    });
  }
  function setupDataHandler() {
    let buffer = "";
    socket.on("data", (data) => {
      buffer += data.toString();
      const lines = buffer.split("\n");
      buffer = lines.pop() || "";
      for (const line of lines) {
        if (line.trim()) {
          const snapshot = deserializeSnapshot(line);
          if (snapshot) {
            console.log("[Vesktop Voice Overlay] Received:", snapshot);
          }
        }
      }
    });
  }
  function sendSnapshot2(snapshot) {
    const line = serializeSnapshot(snapshot) + "\n";
    if (isConnected && socket?.writable) {
      socket.write(line);
    } else {
      sendQueue.push(snapshot);
      if (sendQueue.length > 100) {
        sendQueue.shift();
      }
    }
  }
  function flushQueue() {
    while (sendQueue.length > 0 && isConnected && socket?.writable) {
      const snapshot = sendQueue.shift();
      sendSnapshot2(snapshot);
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
    send: sendSnapshot2,
    disconnect
  };
}

// src/voiceState.ts
var vencordApi = null;
function setVencordApi(api) {
  vencordApi = api;
}
var useVoiceState = () => {
  if (!vencordApi) return null;
  const store = vencordApi.getVoiceStateStore();
  return store?.getVoiceState() ?? null;
};
var useCurrentUser = () => {
  if (!vencordApi) return null;
  return vencordApi.getCurrentUser();
};
function subscribeToVoiceStateChanges(callback) {
  if (!vencordApi) return () => {
  };
  const store = vencordApi.getVoiceStateStore();
  if (!store) return () => {
  };
  return store.subscribe(callback);
}
function getVoiceChannelMembers(voiceState) {
  if (!voiceState) return [];
  return Array.from(voiceState.members.values());
}
function getSelfMember(voiceState, currentUserId) {
  return voiceState.members.get(currentUserId);
}

// src/snapshot.ts
function buildSnapshot(voiceState, currentUser) {
  if (!voiceState || !voiceState.channelId) {
    return null;
  }
  const selfMember = getSelfMember(voiceState, currentUser.id);
  if (!selfMember) {
    return null;
  }
  const self = {
    userId: selfMember.userId,
    username: selfMember.username,
    avatarUrl: selfMember.avatar || "",
    mute: selfMember.mute,
    deaf: selfMember.deaf,
    speaking: selfMember.speaking
  };
  const participants = [];
  const members = getVoiceChannelMembers(voiceState);
  for (const member of members) {
    if (member.userId === currentUser.id) continue;
    participants.push({
      userId: member.userId,
      username: member.username,
      avatarUrl: member.avatar || "",
      speaking: member.speaking,
      volume: member.volume
    });
  }
  return {
    version: 1,
    timestamp: Date.now(),
    self,
    participants
  };
}

// src/index.ts
var socketClient = null;
var unsubscribeVoiceState = null;
var plugin = {
  name: "Vesktop Voice Overlay",
  description: "Wayland-native voice activity overlay for Vesktop",
  version: "0.1.0",
  author: "Roddy",
  start: (api) => {
    console.log("[Vesktop Voice Overlay] Starting plugin...");
    setVencordApi(api);
    const socketPath = getSocketPath();
    socketClient = startSocketClient(socketPath, () => {
      sendCurrentSnapshot();
    });
    unsubscribeVoiceState = subscribeToVoiceStateChanges((voiceState) => {
      const currentUser = useCurrentUser();
      if (currentUser) {
        const snapshot = buildSnapshot(voiceState, currentUser);
        if (snapshot) {
          sendSnapshot(snapshot);
        }
      }
    });
    sendCurrentSnapshot();
  },
  stop: () => {
    console.log("[Vesktop Voice Overlay] Stopping plugin...");
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
function sendSnapshot(snapshot) {
  if (socketClient) {
    socketClient.send(snapshot);
  }
}
var index_default = plugin;
export {
  index_default as default,
  plugin
};
//# sourceMappingURL=index.mjs.map