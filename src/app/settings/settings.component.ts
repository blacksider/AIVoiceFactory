import {Component, NgZone, OnDestroy, OnInit} from '@angular/core';
import {SettingsService} from './settings.service';
import {ActivatedRoute} from '@angular/router';
import {AudioConfigResponseData, AudioSelection, SelectByName, SelectDefault} from './settings';
import {listen} from '@tauri-apps/api/event'

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrls: ['./settings.component.less']
})
export class SettingsComponent implements OnInit, OnDestroy {
  audioConfig!: AudioConfigResponseData;

  audioOutputs!: AudioSelection[];
  selectAudioOutput!: AudioSelection;
  audioInputs!: AudioSelection[];
  selectAudioInput!: AudioSelection;
  private unListenChanges?: () => void;

  constructor(private service: SettingsService,
              private activatedRoute: ActivatedRoute,
              private ngZone: NgZone) {
  }

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(
      ({audioConfig}) => {
        this.audioConfig = audioConfig as AudioConfigResponseData;
        this.initAudioOutputs();
        this.initAudioInputs();
      });
    listen('on_audio_config_change', this.updateAudioConfig.bind(this))
      .then((fn) => {
        this.unListenChanges = fn;
      });
  }

  ngOnDestroy(): void {
    if (this.unListenChanges) {
      this.unListenChanges();
    }
  }

  private updateAudioConfig() {
    this.service.getAudioConfig().subscribe(config => {
      this.ngZone.run(() => {
        this.audioConfig = config;
        this.initAudioOutputs();
        this.initAudioInputs();
      });
    });
  }

  private initAudioOutputs(): void {
    this.audioOutputs = [new SelectDefault()];
    if (this.audioConfig?.output_devices?.length > 0) {
      this.audioConfig.output_devices.forEach(value => {
        const selByName = new SelectByName();
        selByName.name = value;
        this.audioOutputs.push(selByName)
      });
    }
    this.selectAudioOutput = this.audioOutputs[0];
    if (this.audioConfig?.config.output.type == 'ByName') {
      const byName = this.audioConfig?.config.output as SelectByName;
      for (let output of this.audioOutputs) {
        if (output instanceof SelectByName && output.name === byName.name) {
          this.selectAudioOutput = output;
        }
      }
    }
  }

  private initAudioInputs(): void {
    this.audioInputs = [new SelectDefault()];
    if (this.audioConfig?.input_devices?.length > 0) {
      this.audioConfig.input_devices.forEach(value => {
        const selByName = new SelectByName();
        selByName.name = value;
        this.audioInputs.push(selByName)
      });
    }
    this.selectAudioInput = this.audioInputs[0];
    if (this.audioConfig?.config.input.type == 'ByName') {
      const byName = this.audioConfig?.config.input as SelectByName;
      for (let input of this.audioInputs) {
        if (input instanceof SelectByName && input.name === byName.name) {
          this.selectAudioInput = input;
        }
      }
    }
  }

  onChangeAudioOutput() {
    this.service.changeOutputDevice(this.selectAudioOutput)
      .subscribe(config => {
        this.audioConfig = config;
        this.initAudioOutputs();
        this.initAudioInputs();
      });
  }

  onChangeAudioInput() {
    this.service.changeInputDevice(this.selectAudioInput)
      .subscribe(config => {
        this.audioConfig = config;
        this.initAudioOutputs();
        this.initAudioInputs();
      });
  }

  getDeviceLabel(device: AudioSelection): string {
    return device instanceof SelectByName ? device.name : '默认设备';
  }
}
