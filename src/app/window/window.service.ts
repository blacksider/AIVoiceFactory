import {Injectable} from '@angular/core';
import {Observable} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {invoke} from '@tauri-apps/api';
import {AudioCacheIndex} from './audio-data';

@Injectable({
  providedIn: 'root'
})
export class WindowService {

  constructor() {
  }

  checkAudioCaches(): void {
    invoke<any>('check_audio_caches').then(_ => {
    });
  }

  listAudios(): Observable<AudioCacheIndex[]> {
    return fromPromise<AudioCacheIndex[]>(invoke<AudioCacheIndex[]>('list_audios'));
  }

  generateAudio(text: string): Observable<AudioCacheIndex> {
    return fromPromise<AudioCacheIndex>(invoke<AudioCacheIndex>('generate_audio', {text}));
  }
}
