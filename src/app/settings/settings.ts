export abstract class AudioSelection {
  type!: string;
  name!: string;
}

export class SelectDefault extends AudioSelection {

  constructor(name: string) {
    super();
    this.type = 'Default';
    this.name = name;
  }
}

export class SelectByName extends AudioSelection {

  constructor(name: string) {
    super();
    this.type = 'ByName';
    this.name = name;
  }
}

export class AudioSelectionConfig {
  output!: AudioSelection;
  input!: AudioSelection;
}

export class StreamConfig {
  stream_input!: boolean;
  stream_mic_input!: boolean;
}

export class AudioConfigResponseData {
  config!: AudioSelectionConfig;
  stream!: StreamConfig;
  default_output_device!: string;
  output_devices!: string[];
  default_input_device!: string;
  input_devices!: string[];
}
