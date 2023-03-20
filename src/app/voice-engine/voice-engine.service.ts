import {inject, Injectable} from '@angular/core';
import {invoke} from '@tauri-apps/api';
import {Observable} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot} from '@angular/router';
import {VoiceEngineConfig} from './voice-engine';

@Injectable({
  providedIn: 'root'
})
export class VoiceEngineService {
  constructor() {
  }

  getVoiceEngineConfig(): Observable<VoiceEngineConfig> {
    return fromPromise<VoiceEngineConfig>(invoke<VoiceEngineConfig>('get_voice_engine_config'));
  }
}

export const voiceEngineConfigResolver: ResolveFn<VoiceEngineConfig> =
  (route: ActivatedRouteSnapshot, state: RouterStateSnapshot) => {
    return inject(VoiceEngineService).getVoiceEngineConfig();
  };
