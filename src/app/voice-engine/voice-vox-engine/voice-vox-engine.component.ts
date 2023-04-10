import {Component, Input, OnInit} from '@angular/core';
import {FormControl, FormGroup} from '@angular/forms';
import {VoiceEngineService} from '../voice-engine.service';
import {VoiceVoxSpeaker} from './voice-vox';
import {VoiceVoxConfigType} from "../voice-engine";
import {Subject, takeUntil} from "rxjs";

@Component({
  selector: 'app-voice-vox-engine',
  templateUrl: './voice-vox-engine.component.html',
  styleUrls: ['./voice-vox-engine.component.less']
})
export class VoiceVoxEngineComponent implements OnInit {
  @Input() config!: FormGroup;

  speakers?: VoiceVoxSpeaker[];

  configTypes = VoiceVoxConfigType;
  private ngUnsub = new Subject();

  constructor(private service: VoiceEngineService) {
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
          if (!this.cpuArch.value) {
            this.cpuArch.setValue("x64");
          }
        }
      });
  }

  get configType(): FormControl {
    return this.config.get('config_type') as FormControl;
  }

  get device(): FormControl {
    return this.config.get('device') as FormControl;
  }

  get cpuArch(): FormControl {
    return this.config.get('cpu_arch') as FormControl;
  }

  get speakerUuid(): FormControl {
    return this.config.get('speaker_uuid') as FormControl;
  }

  get speakerStyleId(): FormControl {
    return this.config.get('speaker_style_id') as FormControl;
  }

  loadSpeakers() {
    this.service.getVoiceVoxSpeakers()
      .subscribe(value => {
        this.speakers = value;
      });
  }
}
