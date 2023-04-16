import {Injectable} from '@angular/core';
import {BehaviorSubject, Observable} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {invoke} from '@tauri-apps/api';
import {AudioCacheDetail, AudioCacheIndex, AudioRegEvent} from './audio-data';
import {LocalStorageService} from "../local-storage.service";

@Injectable({
  providedIn: 'root'
})
export class WindowService {
  private regTextQueue = new BehaviorSubject<AudioRegEvent>(AudioRegEvent.empty());

  constructor(private localStorage: LocalStorageService) {
  }

  listAudios(): Observable<AudioCacheIndex[]> {
    return fromPromise<AudioCacheIndex[]>(invoke<AudioCacheIndex[]>('list_audios'));
  }

  getAudioDetail(index: string): Observable<AudioCacheDetail> {
    return fromPromise<AudioCacheDetail>(invoke<AudioCacheDetail>('get_audio_detail', {index}));
  }

  deleteAudio(index: string): Promise<boolean> {
    return invoke<boolean>('delete_audio', {index})
  }

  playAudio(index: string): Observable<any> {
    return fromPromise<any>(invoke<any>('play_audio', {index}));
  }

  generateAudio(text: string): Observable<AudioCacheIndex> {
    return fromPromise<AudioCacheIndex>(invoke<AudioCacheIndex>('generate_audio', {text}));
  }

  listenRegText(): Observable<AudioRegEvent> {
    return this.regTextQueue.asObservable();
  }

  handleRegText(text: string) {
    this.regTextQueue.next(AudioRegEvent.new(text));
  }
}
