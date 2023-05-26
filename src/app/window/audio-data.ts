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

  constructor() {
    this.text = '';
  }

  static empty(): AudioRegEvent {
    return new AudioRegEvent();
  }

  static new(text: string): AudioRegEvent {
    let event = new AudioRegEvent();
    event.text = text;
    return event;
  }
}