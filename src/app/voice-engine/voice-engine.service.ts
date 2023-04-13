import {inject, Injectable} from '@angular/core';
import {invoke} from '@tauri-apps/api';
import {Observable, of} from 'rxjs';
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
  private loadingQueue = 0;
  private loadingQueueMax = 3;
  private waitingQueue: (() => void)[] = [];

  constructor() {
  }

  resetLoadingQueue() {
    this.loadingQueue = 0;
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

  private doGetVoicevoxSpeakerInfo(speakerUuid: string): Promise<VoiceVoxSpeakerInfo> {
    return invoke<VoiceVoxSpeakerInfo>('get_voice_vox_speaker_info', {speakerUuid: speakerUuid})
      .then((v) => {
        this.speakerInfoCache.set(speakerUuid, v);
        return v;
      });
  }

  private getVoicevoxSpeakerInfoQueued(speakerUuid: string): Promise<VoiceVoxSpeakerInfo> {
    return new Promise<VoiceVoxSpeakerInfo>((resolve) => {
      // if current loading request is under max request count, process request
      if (this.loadingQueue < this.loadingQueueMax) {
        this.loadingQueue++;
        this.doGetVoicevoxSpeakerInfo(speakerUuid).then((response) => {
          this.loadingQueue--;
          // if queue is not empty, handle it first
          if (this.waitingQueue.length > 0) {
            const nextRequest = this.waitingQueue.shift();
            nextRequest?.();
          }
          resolve(response);
        });
      } else {
        // else put it to waiting queue
        this.waitingQueue.push(() => {
          this.loadingQueue++;
          this.doGetVoicevoxSpeakerInfo(speakerUuid).then((response) => {
            this.loadingQueue--;
            if (this.waitingQueue.length > 0) {
              const nextRequest = this.waitingQueue.shift();
              nextRequest?.();
            }
            resolve(response);
          });
        });
      }
    });
  }

  getVoiceVoxSpeakerInfo(speakerUuid: string): Observable<VoiceVoxSpeakerInfo> {
    if (this.speakerInfoCache.has(speakerUuid)) {
      return of(this.speakerInfoCache.get(speakerUuid)!);
    }
    return fromPromise(this.getVoicevoxSpeakerInfoQueued(speakerUuid));
  }

  isVoicevoxEngineInitialized(): Observable<any> {
    return fromPromise<any>(invoke<any>('is_voicevox_engine_initialized'));
  }

  isLoadingVoicevoxEngine(): Observable<boolean> {
    return fromPromise<boolean>(invoke<boolean>('is_loading_voicevox_engine'));
  }

  stopLoadingVoicevoxEngine(): Observable<any> {
    return fromPromise<any>(invoke<any>('stop_loading_voicevox_engine'));
  }

  checkVoicevoxEngine(): Observable<any> {
    return fromPromise<any>(invoke<any>('check_voicevox_engine'));
  }

  getVoicevoxAvailableBinaries(): Observable<string[]> {
    return fromPromise<string[]>(invoke<string[]>('available_voicevox_binaries'));
  }
}

export const voiceEngineConfigResolver: ResolveFn<VoiceEngineConfig> =
  () => {
    return inject(VoiceEngineService).getVoiceEngineConfig();
  };
