import {NgModule} from "@angular/core";
import {BrowserModule} from "@angular/platform-browser";

import {AppComponent} from "./app.component";
import {AppRoutingModule} from './app-routing.module';
import {MainPageComponent} from './main-page/main-page.component';

import {registerLocaleData} from '@angular/common';
import en from '@angular/common/locales/en';
import {en_US, NZ_DATE_CONFIG, NZ_I18N} from 'ng-zorro-antd/i18n';
import {NzTabsModule} from 'ng-zorro-antd/tabs';
import {WindowComponent} from './window/window.component';
import {VoiceEngineComponent} from './voice-engine/voice-engine.component';
import {AutoTranslationComponent} from './auto-translation/auto-translation.component';
import {NzSpaceModule} from 'ng-zorro-antd/space';
import {NzGridModule} from 'ng-zorro-antd/grid';
import {FormsModule, ReactiveFormsModule} from '@angular/forms';
import {NzInputModule} from 'ng-zorro-antd/input';
import {NzCardModule} from 'ng-zorro-antd/card';
import {NzButtonModule} from 'ng-zorro-antd/button';
import {NzLayoutModule} from 'ng-zorro-antd/layout';
import {SettingsComponent} from './settings/settings.component';
import {NzSelectModule} from 'ng-zorro-antd/select';
import {BrowserAnimationsModule} from '@angular/platform-browser/animations';
import {NzFormModule} from 'ng-zorro-antd/form';
import {NzDividerModule} from 'ng-zorro-antd/divider';
import {NzCheckboxModule} from 'ng-zorro-antd/checkbox';
import {NzNotificationModule} from 'ng-zorro-antd/notification';
import {NzListModule} from 'ng-zorro-antd/list';

registerLocaleData(en);

@NgModule({
  declarations: [
    AppComponent,
    MainPageComponent,
    WindowComponent,
    VoiceEngineComponent,
    AutoTranslationComponent,
    SettingsComponent
  ],
  imports: [
    BrowserModule,
    BrowserAnimationsModule,
    ReactiveFormsModule,
    AppRoutingModule,
    NzTabsModule,
    NzSpaceModule,
    NzGridModule,
    NzInputModule,
    FormsModule,
    NzCardModule,
    NzButtonModule,
    NzLayoutModule,
    NzSelectModule,
    NzFormModule,
    NzDividerModule,
    NzCheckboxModule,
    NzNotificationModule,
    NzListModule
  ],
  providers: [
    {provide: NZ_I18N, useValue: en_US},
    {
      provide: NZ_DATE_CONFIG, useValue: {
        firstDayOfWeek: 1
      }
    }
  ],
  bootstrap: [AppComponent],
})
export class AppModule {
}
