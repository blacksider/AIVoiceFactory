export const EngineTypes = {
  VoiceVox: 'VoiceVox'
};

export interface VoiceEngineConfigData {
}

export class VoiceVoxEngineConfig implements VoiceEngineConfigData {
  protocol!: string;
  apiAddr!: string;
  speaker_uuid!: string;
  speaker_style_id!: number;
}

export class VoiceEngineConfigDetail {
  type!: string;
  config!: VoiceEngineConfigData;
}

export class VoiceEngineConfig {
  type!: string;
  config!: VoiceEngineConfigDetail;
}
