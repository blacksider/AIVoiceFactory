export const EngineTypes = {
  VoiceVox: 'VoiceVox'
};

export interface VoiceEngineConfigData {

}

export class VoiceVoxEngineConfig implements VoiceEngineConfigData {
  protocol!: string;
  apiAddr!: string;
}

export class VoiceEngineConfigDetail {
  type!: string;
  config!: VoiceEngineConfigData;
}

export class VoiceEngineConfig {
  type!: string;
  config!: VoiceEngineConfigDetail;
}
