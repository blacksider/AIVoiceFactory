export const EngineTypes = {
  VoiceVox: 'VoiceVox'
};

export const VoiceVoxConfigType = {
  HTTP: 'Http',
  BINARY: 'Binary'
};

export interface VoiceEngineConfigData {
}

export class VoiceVoxEngineConfig implements VoiceEngineConfigData {
  config_type!: string;
  device!: string;
  cpu_arch?: string;
  protocol!: string;
  api_addr!: string;
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
