import {inject, Injectable} from '@angular/core';
import {invoke} from '@tauri-apps/api';
import {Observable} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {ResolveFn} from '@angular/router';
import {VoiceRecognitionConfig} from "./voice-recognition";

@Injectable({
  providedIn: 'root'
})
export class VoiceRecognitionService {
  constructor() {
  }

  getVoiceRecognitionConfig(): Observable<VoiceRecognitionConfig> {
    return fromPromise<VoiceRecognitionConfig>(invoke<VoiceRecognitionConfig>('get_voice_recognition_config'));
  }

  saveVoiceRecognitionConfig(config: VoiceRecognitionConfig): Observable<boolean> {
    return fromPromise<boolean>(invoke<boolean>('save_voice_recognition_config', {config}));
  }

  isRecorderRecording(): Observable<boolean> {
    return fromPromise<boolean>(invoke<boolean>('is_recorder_recording'));
  }
}

export const voiceRecognitionConfigResolver: ResolveFn<VoiceRecognitionConfig> =
  () => {
    return inject(VoiceRecognitionService).getVoiceRecognitionConfig();
  };
