import {inject, Injectable} from '@angular/core';
import {Observable} from 'rxjs';
import {fromPromise} from 'rxjs/internal/observable/innerFrom';
import {invoke} from '@tauri-apps/api';
import {ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot} from '@angular/router';
import {AutoTranslationConfig} from './auto-translation';

@Injectable({
  providedIn: 'root'
})
export class AutoTranslationService {
  constructor() {
  }

  getAutoTranslationConfig(): Observable<AutoTranslationConfig> {
    return fromPromise<AutoTranslationConfig>(invoke<AutoTranslationConfig>('get_auto_translation_config'));
  }

  saveAutoTranslationConfig(config: AutoTranslationConfig): Observable<boolean> {
    return fromPromise<boolean>(invoke<boolean>('save_auto_translation_config', {config}));
  }
}

export const autoTranslationConfigResolver: ResolveFn<AutoTranslationConfig> =
  (route: ActivatedRouteSnapshot, state: RouterStateSnapshot) => {
    return inject(AutoTranslationService).getAutoTranslationConfig();
  };
