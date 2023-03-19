import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {MainPageComponent} from './main-page/main-page.component';
import {WindowComponent} from './window/window.component';
import {VoiceEngineComponent} from './voice-engine/voice-engine.component';
import {AutoTranslationComponent} from './auto-translation/auto-translation.component';
import {SettingsComponent} from './settings/settings.component'; // CLI imports router

const routes: Routes = [
  {
    path: '',
    component: MainPageComponent,
    children: [
      {path: '', component: WindowComponent},
      {path: 'voice-engine', component: VoiceEngineComponent},
      {path: 'auto-translate', component: AutoTranslationComponent},
      {path: 'settings', component: SettingsComponent}
    ]
  }
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule {
}
