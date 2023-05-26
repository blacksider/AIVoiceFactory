export class VoiceRecognitionConfig {
  enable!: boolean;
  generate_after!: boolean;
  recordKey!: string;
  tool!: RecognitionTool;
}

export abstract class RecognitionTool {
  type!: string;
}

export class Recognizer {
  type!: string;
  name!: string;

  constructor(type: string, name: string) {
    this.type = type;
    this.name = name;
  }
}

export const RecognizerTypes: { [key: string]: Recognizer } = {
  Whisper: new Recognizer("Whisper", "Whisper")
};

export const WhisperConfigType = {
  HTTP: 'Http',
  BINARY: 'Binary'
};

export class RecognizeByWhisper extends RecognitionTool {
  config_type!: string;
  use_model!: string;
  api_addr!: string;
  language?: string | null;

  constructor() {
    super();
    this.type = RecognizerTypes['Whisper'].type;
  }
}
