import {Component, OnInit} from '@angular/core';
import {FormBuilder, FormControl, FormGroup} from "@angular/forms";
import {RecognizeByWhisper, RecognizerTypes, VoiceRecognitionConfig} from "./voice-recognition";
import {ActivatedRoute} from "@angular/router";
import {VoiceRecognitionService} from "./voice-recognition.service";
import {debounceTime, filter} from "rxjs";

@Component({
  selector: 'app-voice-recognition',
  templateUrl: './voice-recognition.component.html',
  styleUrls: ['./voice-recognition.component.less']
})
export class VoiceRecognitionComponent implements OnInit {
  recognizerTypes = RecognizerTypes;
  recognizers= Object.keys(RecognizerTypes);
  configForm!: FormGroup;

  constructor(private activatedRoute: ActivatedRoute,
              private service: VoiceRecognitionService,
              private fb: FormBuilder) {
  }


  ngOnInit(): void {
    this.activatedRoute.data.subscribe(
        ({config}) => {
          const configData = config as VoiceRecognitionConfig;

          this.configForm = this.fb.group({
            enable: [configData.enable],
            recordKey: [configData.recordKey],
          });

          if (configData.tool.type === RecognizerTypes['Whisper'].type) {
            const recognizeByWhisper = configData.tool as RecognizeByWhisper;
            this.configForm.addControl('tool', this.fb.group({
              type: [recognizeByWhisper.type],
              apiAddr: [recognizeByWhisper.apiAddr],
            }));
          }

          this.configForm.valueChanges
              .pipe(
                  filter(() => this.configForm.valid),
                  debounceTime(500),
              )
              .subscribe(value => {
                console.log(value);
                this.service.saveVoiceRecognitionConfig(value).subscribe(() => {
                });
              });
        });
  }

  get enable(): FormControl {
    return this.configForm.get('enable') as FormControl;
  }

  get type(): FormControl {
    return this.configForm.get('tool')?.get('type') as FormControl;
  }
}
