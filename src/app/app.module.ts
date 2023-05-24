import {NgModule} from "@angular/core";
import {BrowserModule} from "@angular/platform-browser";

import {AppComponent} from "./app.component";
import {AppRoutingModule} from './app-routing.module';
import {MainPageComponent} from './main-page/main-page.component';

import {registerLocaleData} from '@angular/common';
import en from '@angular/common/locales/en';
import zh from '@angular/common/locales/zh';
import {NZ_DATE_CONFIG, NZ_I18N, zh_CN} from 'ng-zorro-antd/i18n';
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
import {NzCollapseModule} from 'ng-zorro-antd/collapse';
import {NzSpinModule} from 'ng-zorro-antd/spin';
import {NzTableModule} from 'ng-zorro-antd/table';
import {NzResizableModule} from 'ng-zorro-antd/resizable';
import {IconDefinition} from '@ant-design/icons-angular';
import {NzIconModule} from 'ng-zorro-antd/icon';
import {NzTimelineModule} from 'ng-zorro-antd/timeline';

import {
  CheckCircleOutline,
  DeleteOutline,
  EyeInvisibleOutline,
  EyeOutline,
  LoadingOutline,
  PauseCircleOutline,
  PlayCircleOutline
} from '@ant-design/icons-angular/icons';
import {VoiceVoxEngineComponent} from './voice-engine/voice-vox-engine/voice-vox-engine.component';
import {VoiceVoxSpeakerComponent} from './voice-engine/voice-vox-speaker/voice-vox-speaker.component';
import {NzImageModule} from 'ng-zorro-antd/image';
import {VoiceRecognitionComponent} from './voice-recognition/voice-recognition.component';
import {KeyRecorderComponent} from './key-recorder/key-recorder.component';
import {RecordingPopupComponent} from './recording-popup/recording-popup.component';
import {NzModalModule} from 'ng-zorro-antd/modal';
import {NzAlertModule} from "ng-zorro-antd/alert";

const icons: IconDefinition[] = [
  PlayCircleOutline,
  PauseCircleOutline,
  DeleteOutline,
  CheckCircleOutline,
  LoadingOutline,
  EyeInvisibleOutline,
  EyeOutline
];

registerLocaleData(en);
registerLocaleData(zh);

@NgModule({
  declarations: [
    AppComponent,
    MainPageComponent,
    WindowComponent,
    VoiceEngineComponent,
    AutoTranslationComponent,
    SettingsComponent,
    VoiceVoxEngineComponent,
    VoiceVoxSpeakerComponent,
    VoiceRecognitionComponent,
    KeyRecorderComponent,
    RecordingPopupComponent
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
    NzCollapseModule,
    NzSpinModule,
    NzTableModule,
    NzResizableModule,
    NzIconModule.forRoot(icons),
    NzImageModule,
    NzModalModule,
    NzAlertModule,
    NzTimelineModule
  ],
  providers: [
    {provide: NZ_I18N, useValue: zh_CN},
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
