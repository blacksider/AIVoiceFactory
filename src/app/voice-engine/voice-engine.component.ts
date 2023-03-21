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

        if (voiceEngineConfig.type === EngineTypes.VoiceVox) {
          const voiceVoxConfig = voiceEngineConfig.config as VoiceVoxEngineConfig;
          const configTypeControl = this.fb.control(voiceEngineConfig.type);
          this.type.valueChanges.subscribe(value => {
            configTypeControl.setValue(value);
          });
          this.voiceEngineConfigForm.addControl('config', this.fb.group({
            type: configTypeControl,
            config: this.fb.group({
              protocol: [voiceVoxConfig.protocol],
              apiAddr: [voiceVoxConfig.apiAddr]
            })
          }));
        }
        this.voiceEngineConfigForm.valueChanges
          .pipe(
            filter(() => this.voiceEngineConfigForm.valid),
            debounceTime(500),
          )
          .subscribe(value => {
            this.service.saveVoiceEngineConfig(value).subscribe((ok) => {
              if (!ok) {
                this.notification.error('警告', '配置更新失败！')
              }
            });
          });
      });
  }

  get type(): FormControl {
    return this.voiceEngineConfigForm.get('type') as FormControl;
  }
}
