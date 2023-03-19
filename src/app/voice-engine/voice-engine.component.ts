import {Component, OnInit} from '@angular/core';
import {EngineTypes, VoiceVoxEngineConfig} from './voice-engine';

@Component({
  selector: 'app-voice-engine',
  templateUrl: './voice-engine.component.html',
  styleUrls: ['./voice-engine.component.less']
})
export class VoiceEngineComponent implements OnInit {
  engineTypes = EngineTypes;
  engineTypeValues = Object.keys(EngineTypes);
  selectedEngine = EngineTypes.VoiceVox;

  voiceVoxEngineConfig!: VoiceVoxEngineConfig;

  ngOnInit(): void {
    this.voiceVoxEngineConfig = {
      protocol: "http",
      apiAddr: ''
    };
  }

}
