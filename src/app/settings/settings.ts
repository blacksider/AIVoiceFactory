export abstract class AudioSelection {
  type!: string;
}

export class SelectDefault extends AudioSelection {

  constructor() {
    super();
    this.type = 'Default';
  }
}

export class SelectByName extends AudioSelection {
  name!: string;

  constructor() {
    super();
    this.type = 'ByName';
  }
}

export class AudioSelectionConfig {
  output!: AudioSelection;
  input!: AudioSelection;
}

export class AudioConfigResponseData {
  config!: AudioSelectionConfig;
  output_devices!: string[];
  input_devices!: string[];
}
