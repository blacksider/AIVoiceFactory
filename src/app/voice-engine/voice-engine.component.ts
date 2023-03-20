import {Component, OnInit} from '@angular/core';
import {EngineTypes, VoiceEngineConfig, VoiceEngineConfigDetail, VoiceVoxEngineConfig} from './voice-engine';
import {VoiceEngineService} from './voice-engine.service';
import {ActivatedRoute} from '@angular/router';

@Component({
  selector: 'app-voice-engine',
  templateUrl: './voice-engine.component.html',
  styleUrls: ['./voice-engine.component.less']
})
export class VoiceEngineComponent implements OnInit {
  engineTypes = EngineTypes;
  engineTypeValues = Object.keys(EngineTypes);
  voiceEngineConfig!: VoiceEngineConfigDetail;
  voiceVoxEngineConfig!: VoiceVoxEngineConfig;

  constructor(private service: VoiceEngineService,
              private activatedRoute: ActivatedRoute) {
  }

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(
      ({config}) => {
        const engineConfig = config as VoiceEngineConfig;
        this.voiceEngineConfig = engineConfig.config;
        if (this.voiceEngineConfig.type === EngineTypes.VoiceVox) {
          this.voiceVoxEngineConfig = this.voiceEngineConfig.config as VoiceVoxEngineConfig;
        }
      });
  }

}
