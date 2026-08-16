import { Snapshot, ParticipantSelf, Participant } from './protocol';
import { VoiceState, VoiceMember, CurrentUser, getSelfMember, getVoiceChannelMembers } from './voiceState';

export function buildSnapshot(voiceState: VoiceState, currentUser: CurrentUser): Snapshot | null {
  if (!voiceState || !voiceState.channelId) {
    return null;
  }

  const selfMember = getSelfMember(voiceState, currentUser.id);
  if (!selfMember) {
    return null;
  }

  const self: ParticipantSelf = {
    userId: selfMember.userId,
    username: selfMember.username,
    avatarUrl: selfMember.avatar || '',
    mute: selfMember.mute,
    deaf: selfMember.deaf,
    speaking: selfMember.speaking,
  };

  const participants: Participant[] = [];
  const members = getVoiceChannelMembers(voiceState);
  
  for (const member of members) {
    if (member.userId === currentUser.id) continue;
    participants.push({
      userId: member.userId,
      username: member.username,
      avatarUrl: member.avatar || '',
      speaking: member.speaking,
      volume: member.volume,
    });
  }

  return {
    version: 1,
    timestamp: Date.now(),
    self,
    participants,
  };
}

export function extractVoiceStateFromVencord(): { voiceState: VoiceState | null; currentUser: CurrentUser | null } {
  // This will be called from the plugin entry point where Vencord API is available
  return { voiceState: null, currentUser: null };
}
