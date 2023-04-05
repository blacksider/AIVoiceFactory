import {Injectable} from '@angular/core';
import {Observable} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {invoke} from '@tauri-apps/api';
import {AudioCacheDetail, AudioCacheIndex} from './audio-data';

@Injectable({
  providedIn: 'root'
})
export class WindowService {

  constructor() {
  }

  listAudios(): Observable<AudioCacheIndex[]> {
    return fromPromise<AudioCacheIndex[]>(invoke<AudioCacheIndex[]>('list_audios'));
  }

  getAudioDetail(index: string): Observable<AudioCacheDetail> {
    return fromPromise<AudioCacheDetail>(invoke<AudioCacheDetail>('get_audio_detail', {index}));
  }

  playAudio(index: string): Observable<any> {
    return fromPromise<any>(invoke<any>('play_audio', {index}));
  }

  generateAudio(text: string): Observable<AudioCacheIndex> {
    return fromPromise<AudioCacheIndex>(invoke<AudioCacheIndex>('generate_audio', {text}));
  }
}
