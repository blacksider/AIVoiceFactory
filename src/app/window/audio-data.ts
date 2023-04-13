export class AudioCacheIndex {
  name!: string;
  time!: string;
  active?: boolean;
}

export class AudioCacheDetail {
  source!: string;
  translated!: string;
}

export class AudioRegEvent {
  text!: string;
  generated!: boolean;

  constructor() {
    this.text = '';
    this.generated = true;
  }

  static empty(): AudioRegEvent {
    return new AudioRegEvent();
  }

  static new(text: string, generated?: boolean): AudioRegEvent {
    let event = new AudioRegEvent();
    event.text = text;
    event.generated = !!generated;
    return event;
  }
}