import {inject, Injectable} from '@angular/core';
import {invoke} from '@tauri-apps/api';
import {Observable} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {AudioConfigResponseData, AudioSelection, StreamConfig} from './settings';
import {ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot} from '@angular/router';

@Injectable({
  providedIn: 'root'
})
export class SettingsService {
  constructor() {
  }

  getAudioConfig(): Observable<AudioConfigResponseData> {
    return fromPromise<AudioConfigResponseData>(invoke<AudioConfigResponseData>('get_audio_config'));
  }

  changeOutputDevice(selection: AudioSelection): Observable<AudioConfigResponseData> {
    return fromPromise<AudioConfigResponseData>(invoke<AudioConfigResponseData>('change_output_device', {selection}));
  }

  changeInputDevice(selection: AudioSelection): Observable<AudioConfigResponseData> {
    return fromPromise<AudioConfigResponseData>(invoke<AudioConfigResponseData>('change_input_device', {selection}));
  }

  changeStreamConfig(stream: StreamConfig): Observable<AudioConfigResponseData> {
    return fromPromise<AudioConfigResponseData>(invoke<AudioConfigResponseData>('change_stream_config', {stream}));
  }
}

export const audioConfigResolver: ResolveFn<AudioConfigResponseData> =
  (route: ActivatedRouteSnapshot, state: RouterStateSnapshot) => {
    return inject(SettingsService).getAudioConfig();
  };
