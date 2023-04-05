export class VoiceRecognitionConfig {
    enable!: boolean;
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

export class RecognizeByWhisper extends RecognitionTool {
    apiAddr!: string;

    constructor() {
        super();
        this.type = RecognizerTypes['Whisper'].type;
    }
}
