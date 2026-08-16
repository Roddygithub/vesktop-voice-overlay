import { VoiceState, VoiceMember, CurrentUser, VencordAPI, UseVoiceState, UseCurrentUser } from './vencord/api';

// Re-export types for compatibility
export type { VoiceState, VoiceMember, CurrentUser };

// Vencord API access - these will be injected at runtime by Vencord
let vencordApi: VencordAPI | null = null;

export function setVencordApi(api: VencordAPI) {
  vencordApi = api;
}

export function getVencordApi(): VencordAPI | null {
  return vencordApi;
}

// Hook: useVoiceState - returns current voice state
export const useVoiceState: UseVoiceState = () => {
  if (!vencordApi) return null;
  const store = vencordApi.getVoiceStateStore();
  return store?.getVoiceState() ?? null;
};

// Hook: useCurrentUser - returns current user info
export const useCurrentUser: UseCurrentUser = () => {
  if (!vencordApi) return null;
  return vencordApi.getCurrentUser();
};

// Subscribe to voice state changes
export function subscribeToVoiceStateChanges(callback: (state: VoiceState) => void): () => void {
  if (!vencordApi) return () => {};
  const store = vencordApi.getVoiceStateStore();
  if (!store) return () => {};
  return store.subscribe(callback);
}

// Get voice channel members as array
export function getVoiceChannelMembers(voiceState: VoiceState): VoiceMember[] {
  if (!voiceState) return [];
  return Array.from(voiceState.members.values());
}

// Extract self member from voice state
export function getSelfMember(voiceState: VoiceState, currentUserId: string): VoiceMember | undefined {
  return voiceState.members.get(currentUserId);
}

// Check if user is in a voice channel
export function isInVoiceChannel(voiceState: VoiceState | null): boolean {
  return voiceState !== null && voiceState.channelId !== null;
}

// Check if user is speaking
export function isSpeaking(voiceState: VoiceState | null): boolean {
  return voiceState?.speaking === true;
}

// Check if user is muted
export function isMuted(voiceState: VoiceState | null): boolean {
  return voiceState?.selfMute === true;
}

// Check if user is deafened
export function isDeafened(voiceState: VoiceState | null): boolean {
  return voiceState?.selfDeaf === true;
}

// Get member count in voice channel
export function getMemberCount(voiceState: VoiceState | null): number {
  if (!voiceState) return 0;
  return voiceState.members.size;
}
