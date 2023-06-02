import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {MainPageComponent} from './main-page/main-page.component';
import {WindowComponent} from './window/window.component';
import {VoiceEngineComponent} from './voice-engine/voice-engine.component';
import {AutoTranslationComponent} from './auto-translation/auto-translation.component';
import {SettingsComponent} from './settings/settings.component';
import {voiceEngineConfigResolver} from './voice-engine/voice-engine.service';
import {autoTranslationConfigResolver} from './auto-translation/auto-translation.service';
import {audioConfigResolver, httpProxyConfigResolver} from './settings/settings.service';
import {VoiceRecognitionComponent} from "./voice-recognition/voice-recognition.component";
import {voiceRecognitionConfigResolver} from "./voice-recognition/voice-recognition.service";
import {RecordingPopupComponent} from "./recording-popup/recording-popup.component";

const routes: Routes = [
  {
    path: '',
    component: MainPageComponent,
    children: [
      {path: '', component: WindowComponent},
      {
        path: 'voice-engine',
        component: VoiceEngineComponent,
        resolve: {
          config: voiceEngineConfigResolver
        }
      },
      {
        path: 'voice-rec',
        component: VoiceRecognitionComponent,
        resolve: {
          config: voiceRecognitionConfigResolver
        }
      },
      {
        path: 'auto-translate',
        component: AutoTranslationComponent,
        resolve: {
          config: autoTranslationConfigResolver
        }
      },
      {
        path: 'settings',
        component: SettingsComponent,
        resolve: {
          audioConfig: audioConfigResolver,
          httpProxyConfig: httpProxyConfigResolver,
        }
      }
    ]
  },
  {
    path: 'recording',
    component: RecordingPopupComponent
  }
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule {
}
