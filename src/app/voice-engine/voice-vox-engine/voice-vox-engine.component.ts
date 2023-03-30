import {Component, Input, OnInit} from '@angular/core';
import {FormControl, FormGroup} from '@angular/forms';
import {VoiceEngineService} from '../voice-engine.service';
import {VoiceVoxSpeaker} from './voice-vox';

@Component({
  selector: 'app-voice-vox-engine',
  templateUrl: './voice-vox-engine.component.html',
  styleUrls: ['./voice-vox-engine.component.less']
})
export class VoiceVoxEngineComponent implements OnInit {
  @Input() config!: FormGroup;

  speakers?: VoiceVoxSpeaker[];

  constructor(private service: VoiceEngineService) {
  }

  ngOnInit(): void {
    this.loadSpeakers();
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
