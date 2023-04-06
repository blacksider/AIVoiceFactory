import {Injectable} from '@angular/core';
import {BehaviorSubject, Observable, Subject} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {invoke} from '@tauri-apps/api';
import {AudioCacheDetail, AudioCacheIndex} from './audio-data';
import {LocalStorageService} from "../local-storage.service";

const KEY_SYNC_STATE = "syncOnTextRecognize";

@Injectable({
  providedIn: 'root'
})
export class WindowService {
  private regTextQueue = new BehaviorSubject<string>("");
  private audioGenIndex = new Subject<AudioCacheIndex>();
  private syncOnTextRecognize = false;

  constructor(private localStorage: LocalStorageService) {
    const storedSyncState = this.localStorage.get(KEY_SYNC_STATE);
    if (!!storedSyncState) {
      this.syncOnTextRecognize = new Boolean(storedSyncState).valueOf();
    }
  }

  getSyncOnTextState(): boolean {
    return this.syncOnTextRecognize;
  }

  updateSyncOnTextState(value: boolean) {
    this.syncOnTextRecognize = value;
    if (this.syncOnTextRecognize) {
      this.localStorage.set(KEY_SYNC_STATE, "true");
    } else {
      this.localStorage.set(KEY_SYNC_STATE, "");
    }
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

  listenRegText(): Observable<string> {
    return this.regTextQueue.asObservable();
  }

  listenAudioIndex(): Observable<AudioCacheIndex> {
    return this.audioGenIndex.asObservable();
  }

  doGenerateAudio(text: string) {
    this.generateAudio(text)
      .subscribe((res) => {
        this.audioGenIndex.next(res);
      });
  }

  handleRegText(text: string) {
    this.regTextQueue.next(text);
    if (this.syncOnTextRecognize) {
      this.doGenerateAudio(text);
    }
  }
}
