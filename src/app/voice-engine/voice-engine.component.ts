import {Component, OnInit} from '@angular/core';
import {EngineTypes, VoiceEngineConfig, VoiceVoxEngineConfig} from './voice-engine';
import {VoiceEngineService} from './voice-engine.service';
import {ActivatedRoute} from '@angular/router';
import {FormBuilder, FormControl, FormGroup} from '@angular/forms';
import {NzNotificationService} from 'ng-zorro-antd/notification';
import {debounceTime, filter} from 'rxjs';

@Component({
  selector: 'app-voice-engine',
  templateUrl: './voice-engine.component.html',
  styleUrls: ['./voice-engine.component.less']
})
export class VoiceEngineComponent implements OnInit {
  engineTypes = EngineTypes;
  engineTypeValues = Object.keys(EngineTypes);

  voiceEngineConfigForm!: FormGroup;

  constructor(private service: VoiceEngineService,
              private activatedRoute: ActivatedRoute,
              private notification: NzNotificationService,
              private fb: FormBuilder) {
  }

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(
      ({config}) => {
        const engineConfig = config as VoiceEngineConfig;
        const voiceEngineConfig = engineConfig.config;
        this.voiceEngineConfigForm = this.fb.group({
          type: [voiceEngineConfig.type]
        });

        const configTypeControl = this.fb.control(voiceEngineConfig.type);
        this.type.valueChanges.subscribe(value => {
          configTypeControl.setValue(value);
        });

        if (voiceEngineConfig.type === EngineTypes.VoiceVox) {
          this.initVoiceVoxForm(configTypeControl, voiceEngineConfig.config as VoiceVoxEngineConfig);
        }
        this.voiceEngineConfigForm.valueChanges
          .pipe(
            filter(() => this.voiceEngineConfigForm.valid),
            debounceTime(500),
          )
          .subscribe(value => {
            console.log('new value: ', value);
            this.service.saveVoiceEngineConfig(value).subscribe((ok) => {
              if (!ok) {
                this.notification.error('警告', '配置更新失败！')
              }
            });
          });
      });
  }

  private initVoiceVoxForm(configTypeControl: FormControl,
                           voiceVoxConfig: VoiceVoxEngineConfig) {
    this.voiceEngineConfigForm.addControl('config', this.fb.group({
      type: configTypeControl,
      config: this.fb.group({
        protocol: [voiceVoxConfig.protocol],
        apiAddr: [voiceVoxConfig.apiAddr],
        speaker_uuid: [voiceVoxConfig.speaker_uuid],
        speaker_style_id: [voiceVoxConfig.speaker_style_id],
      })
    }));
  }

  get config(): FormGroup {
    return this.voiceEngineConfigForm.get('config')?.get('config') as FormGroup;
  }

  get type(): FormControl {
    return this.voiceEngineConfigForm.get('type') as FormControl;
  }
}
