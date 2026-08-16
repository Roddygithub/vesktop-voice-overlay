// Vencord API Type Definitions
// Based on Vencord's internal API structure

export interface VoiceState {
  channelId: string | null;
  guildId: string | null;
  selfMute: boolean;
  selfDeaf: boolean;
  selfVideo: boolean;
  selfStream: boolean;
  speaking: boolean;
  members: Map<string, VoiceMember>;
}

export interface VoiceMember {
  userId: string;
  username: string;
  discriminator: string;
  avatar: string | null;
  mute: boolean;
  deaf: boolean;
  speaking: boolean;
  volume?: number;
  nick?: string;
}

export interface CurrentUser {
  id: string;
  username: string;
  discriminator: string;
  avatar: string | null;
  email?: string;
}

export interface VoiceStateStore {
  getVoiceState(): VoiceState | null;
  getCurrentUser(): CurrentUser | null;
  subscribe(callback: (state: VoiceState) => void): () => void;
}

export interface VencordAPI {
  getVoiceStateStore(): VoiceStateStore;
  getCurrentUser(): CurrentUser | null;
}

// Hook types for Vencord plugins
export type UseVoiceState = () => VoiceState | null;
export type UseCurrentUser = () => CurrentUser | null;

// Vencord plugin types
export interface VencordPlugin {
  name: string;
  description: string;
  version: string;
  author: string;
  start(): void;
  stop(): void;
}

// Settings API
export interface SettingsStore {
  get<T>(key: string): T | undefined;
  set<T>(key: string, value: T): void;
  subscribe(key: string, callback: (value: any) => void): () => void;
}

// Logger
export interface Logger {
  log(...args: any[]): void;
  warn(...args: any[]): void;
  error(...args: any[]): void;
  debug(...args: any[]): void;
}

// Patch API
export interface PatchAPI {
  before(target: any, method: string, patch: (args: any[]) => any[]): void;
  after(target: any, method: string, patch: (result: any, args: any[]) => any): void;
  instead(target: any, method: string, patch: (args: any[]) => any): void;
  undo(target: any, method: string): void;
}

// Webpack modules
export interface WebpackModules {
  findByUniqueProperties(properties: string[]): any;
  findByDisplayName(name: string): any;
  findByProps(...props: string[]): any;
  getByKeys(keys: string[]): any;
}

// React components
export interface ReactComponent {
  (props: any): JSX.Element;
}

// Flux dispatcher
export interface Dispatcher {
  subscribe(store: string, callback: (action: any) => void): () => void;
  dispatch(action: any): void;
}

// Module exports
export const VencordApi = {
  getVoiceStateStore: (): VoiceStateStore => {
    // This would be implemented by Vencord
    throw new Error('Not implemented - Vencord provides this at runtime');
  },
  getCurrentUser: (): CurrentUser | null => {
    throw new Error('Not implemented - Vencord provides this at runtime');
  },
  settings: {} as SettingsStore,
  logger: {} as Logger,
  patches: {} as PatchAPI,
  webpackModules: {} as WebpackModules,
  dispatcher: {} as Dispatcher,
};

// Type guards
export function isVoiceState(obj: any): obj is VoiceState {
  return obj && typeof obj === 'object' && 'channelId' in obj && 'members' in obj;
}

export function isVoiceMember(obj: any): obj is VoiceMember {
  return obj && typeof obj === 'object' && 'userId' in obj && 'username' in obj;
}
