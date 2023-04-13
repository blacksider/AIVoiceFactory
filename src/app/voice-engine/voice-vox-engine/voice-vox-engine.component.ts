import {Component, Input, NgZone, OnChanges, OnDestroy, OnInit, SimpleChanges} from '@angular/core';
import {FormControl, FormGroup} from '@angular/forms';
import {VoiceEngineService} from '../voice-engine.service';
import {VoiceVoxSpeaker} from './voice-vox';
import {VoiceVoxConfigType} from "../voice-engine";
import {Subject, takeUntil} from "rxjs";
import {listen} from "@tauri-apps/api/event";

class DeviceType {
  key!: string;
  label!: string;
}

@Component({
  selector: 'app-voice-vox-engine',
  templateUrl: './voice-vox-engine.component.html',
  styleUrls: ['./voice-vox-engine.component.less']
})
export class VoiceVoxEngineComponent implements OnInit, OnChanges, OnDestroy {
  @Input() config!: FormGroup;
  @Input() initialized!: boolean;

  speakers?: VoiceVoxSpeaker[];

  configTypes = VoiceVoxConfigType;

  availableBins: { [key: string]: boolean } = {};

  deviceTypes: DeviceType[] = [
    {key: "cpu", label: "CPU"},
    {key: "cuda", label: "CUDA"},
    {key: "directml", label: "CPU"},
  ];

  private unListenBinLoad?: () => void;

  private ngUnsub = new Subject();

  constructor(private service: VoiceEngineService,
              private ngZone: NgZone) {
  }

  ngOnInit(): void {
    this.loadSpeakers();
    this.configType.valueChanges
      .pipe(takeUntil(this.ngUnsub))
      .subscribe(value => {
        if (value === VoiceVoxConfigType.BINARY) {
          if (!this.device.value) {
            this.device.setValue("cpu");
          }
        }
      });
    this.loadAvailableBins();
    listen('on_whisper_model_loaded', (_) => {
      this.loadAvailableBins();
    })
      .then((fn) => {
        this.unListenBinLoad = fn;
      });
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['initialized']) {
      this.loadSpeakers();
    }
  }

  ngOnDestroy(): void {
    this.ngUnsub.next({});
    this.ngUnsub.complete();
    if (this.unListenBinLoad) {
      this.unListenBinLoad();
    }
  }

  private loadAvailableBins() {
    this.service.getVoicevoxAvailableBinaries().subscribe(value => {
      this.ngZone.run(() => {
        const bins: { [key: string]: boolean } = {};
        if (!!value) {
          for (let key of value) {
            bins[key] = true;
          }
        }
        this.availableBins = bins;
      });
    });
  }

  get configType(): FormControl {
    return this.config.get('config_type') as FormControl;
  }

  get device(): FormControl {
    return this.config.get('device') as FormControl;
  }

  get speakerUuid(): FormControl {
    return this.config.get('speaker_uuid') as FormControl;
  }

  get speakerStyleId(): FormControl {
    return this.config.get('speaker_style_id') as FormControl;
  }

  loadSpeakers() {
    if (!this.initialized) {
      return;
    }
    this.service.getVoiceVoxSpeakers()
      .subscribe(value => {
        this.speakers = value;
      });
  }
}
