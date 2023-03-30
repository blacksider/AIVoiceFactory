export class VoiceVoxSpeakerStyle {
  id!: number;
  name!: string;
}

export class VoiceVoxSpeaker {
  name!: string;
  speaker_uuid!: string;
  version!: string;
  styles!: VoiceVoxSpeakerStyle[];
}

export class VoiceVoxSpeakerStyleInfo {
  id!: number;
  icon!: string;
  portrait!: string;
  voice_samples!: string[];
}

export class VoiceVoxSpeakerInfo {
  policy!: string;
  portrait!: string;
  style_infos!: VoiceVoxSpeakerStyleInfo[];
}
