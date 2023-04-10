import {inject, Injectable} from '@angular/core';
import {invoke} from '@tauri-apps/api';
import {Observable, of, tap} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {ResolveFn} from '@angular/router';
import {VoiceEngineConfig} from './voice-engine';
import {VoiceVoxSpeaker, VoiceVoxSpeakerInfo} from './voice-vox-engine/voice-vox';

@Injectable({
  providedIn: 'root'
})
export class VoiceEngineService {
  playingSample: string | null = null;
  private speakerInfoCache = new Map<string, VoiceVoxSpeakerInfo>();

  constructor() {
  }

  getVoiceEngineConfig(): Observable<VoiceEngineConfig> {
    return fromPromise<VoiceEngineConfig>(invoke<VoiceEngineConfig>('get_voice_engine_config'));
  }

  saveVoiceEngineConfig(config: VoiceEngineConfig): Observable<boolean> {
    return fromPromise<boolean>(invoke<boolean>('save_voice_engine_config', {config}));
  }

  getVoiceVoxSpeakers(): Observable<VoiceVoxSpeaker[]> {
    return fromPromise<VoiceVoxSpeaker[]>(invoke<VoiceVoxSpeaker[]>('get_voice_vox_speakers'));
  }

  getVoiceVoxSpeakerInfo(speakerUuid: string): Observable<VoiceVoxSpeakerInfo> {
    if (this.speakerInfoCache.has(speakerUuid)) {
      return of(this.speakerInfoCache.get(speakerUuid)!);
    }
    return fromPromise<VoiceVoxSpeakerInfo>(invoke<VoiceVoxSpeakerInfo>('get_voice_vox_speaker_info', {speakerUuid: speakerUuid}))
      .pipe(
        tap(v => {
          this.speakerInfoCache.set(speakerUuid, v);
        })
      );
  }

  isDownloadVoicevoxBin(): Observable<boolean> {
    return fromPromise<boolean>(invoke<boolean>('is_downloading_voicevox_bin'));
  }
}

export const voiceEngineConfigResolver: ResolveFn<VoiceEngineConfig> =
    () => {
    return inject(VoiceEngineService).getVoiceEngineConfig();
  };
