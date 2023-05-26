import {Component, NgZone, OnDestroy, OnInit} from '@angular/core';
import {FormBuilder, FormControl, FormGroup} from "@angular/forms";
import {RecognizeByWhisper, RecognizerTypes, VoiceRecognitionConfig, WhisperConfigType} from "./voice-recognition";
import {ActivatedRoute} from "@angular/router";
import {VoiceRecognitionService} from "./voice-recognition.service";
import {debounceTime, filter} from "rxjs";
import {WhisperLanguages, WhisperModels} from "./whisper_data";
import {listen} from "@tauri-apps/api/event";

@Component({
  selector: 'app-voice-recognition',
  templateUrl: './voice-recognition.component.html',
  styleUrls: ['./voice-recognition.component.less']
})
export class VoiceRecognitionComponent implements OnInit, OnDestroy {
  recognizerTypes = RecognizerTypes;
  recognizers = Object.keys(RecognizerTypes);
  configForm!: FormGroup;

  whisperLanguages: { [key: string]: string } = {};
  whisperLanguageTypes: string[] = [];
  whisperConfigTypes = WhisperConfigType;
  whisperModels = WhisperModels;
  whisperAvailableModels: { [key: string]: boolean } = {};

  private unListenWhisperModelLoad?: () => void;

  constructor(private activatedRoute: ActivatedRoute,
              private service: VoiceRecognitionService,
              private ngZone: NgZone,
              private fb: FormBuilder) {
  }


  ngOnInit(): void {
    this.loadWhisperAvailableModel();
    this.activatedRoute.data.subscribe(
      ({config}) => {
        this.loadAndInitByConfig(config as VoiceRecognitionConfig);
      });
    WhisperLanguages.forEach((value, _) => {
      this.whisperLanguages[value.key] = value.name;
      this.whisperLanguageTypes.push(value.key);
    });
    listen('on_whisper_model_loaded', (_) => {
      this.loadWhisperAvailableModel();
    })
      .then((fn) => {
        this.unListenWhisperModelLoad = fn;
      });
  }

  ngOnDestroy(): void {
    if (this.unListenWhisperModelLoad) {
      this.unListenWhisperModelLoad();
    }
  }

  private loadAndInitByConfig(configData: VoiceRecognitionConfig) {
    this.configForm = this.fb.group({
      enable: [configData.enable],
      generate_after: [configData.generate_after],
      recordKey: [configData.recordKey],
    });

    if (configData.tool.type === RecognizerTypes['Whisper'].type) {
      const recognizeByWhisper = configData.tool as RecognizeByWhisper;
      // translate null to auto
      if (recognizeByWhisper.language === null) {
        recognizeByWhisper.language = 'auto';
      }
      this.configForm.addControl('tool', this.fb.group({
        type: [recognizeByWhisper.type],
        api_addr: [recognizeByWhisper.api_addr],
        config_type: [recognizeByWhisper.config_type],
        use_model: [recognizeByWhisper.use_model],
        language: [recognizeByWhisper.language]
      }));
    }

    this.configForm.valueChanges
      .pipe(
        filter(() => this.configForm.valid),
        debounceTime(500),
      )
      .subscribe(value => {
        if (value.tool.type === RecognizerTypes['Whisper'].type) {
          // translate auto to null
          if (value.tool.language === 'auto') {
            value.tool.language = null;
          }
        }
        this.service.saveVoiceRecognitionConfig(value).subscribe(() => {
        });
      });
  }

  private loadWhisperAvailableModel() {
    this.service.getWhisperAvailableModels().subscribe(value => {
      this.ngZone.run(() => {
        const models: { [key: string]: boolean } = {};
        if (!!value) {
          for (let key of value) {
            models[key] = true;
          }
        }
        this.whisperAvailableModels = models;
      });
    });
  }

  get enable(): FormControl {
    return this.configForm.get('enable') as FormControl;
  }

  get type(): FormControl {
    return this.configForm.get('tool')?.get('type') as FormControl;
  }

  get whisperConfigType(): FormControl {
    return this.configForm.get('tool')?.get('config_type') as FormControl;
  }

  get whisperUseModel(): FormControl {
    return this.configForm.get('tool')?.get('use_model') as FormControl;
  }
}
