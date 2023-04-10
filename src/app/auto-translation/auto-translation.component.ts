import {Component, OnInit} from '@angular/core';
import {AutoTranslationConfig, TranslateByBaidu, Translator, TranslatorTypes} from './auto-translation';
import {ActivatedRoute} from '@angular/router';
import {AutoTranslationService} from './auto-translation.service';
import {FormBuilder, FormControl, FormGroup} from '@angular/forms';
import {debounceTime, filter} from 'rxjs';
import {BaiduLanguages} from "./baidu_languages";

@Component({
  selector: 'app-auto-translation',
  templateUrl: './auto-translation.component.html',
  styleUrls: ['./auto-translation.component.less']
})
export class AutoTranslationComponent implements OnInit {
  translatorTypes: { [key: string]: Translator } = TranslatorTypes;
  translators = Object.keys(TranslatorTypes);

  baiduFromLanguageTypes: string[] = [];
  baiduToLanguageTypes: string[] = [];
  baiduLanguages: { [key: string]: string } = {};

  configForm!: FormGroup;

  constructor(private activatedRoute: ActivatedRoute,
              private service: AutoTranslationService,
              private fb: FormBuilder) {
  }

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(
      ({config}) => {
        const configData = config as AutoTranslationConfig;

        this.configForm = this.fb.group({
          enable: [configData.enable]
        });

        if (configData.tool.type === TranslatorTypes['Baidu'].type) {
          const translateByBaidu = configData.tool as TranslateByBaidu;
          this.configForm.addControl('tool', this.fb.group({
            type: [translateByBaidu.type],
            api_addr: [translateByBaidu.api_addr],
            appId: [translateByBaidu.appId],
            secret: [translateByBaidu.secret],
            from: [translateByBaidu.from],
            to: [translateByBaidu.to]
          }));
        }

        this.configForm.valueChanges
          .pipe(
            filter(() => this.configForm.valid),
            debounceTime(500),
          )
          .subscribe(value => {
            this.service.saveAutoTranslationConfig(value).subscribe(() => {
            });
          });
      });
    BaiduLanguages.forEach((value, index) => {
      if (index > 0) {
        this.baiduToLanguageTypes.push(value.key);
      }
      this.baiduFromLanguageTypes.push(value.key);
      this.baiduLanguages[value.key] = value.name;
    });
  }

  get enable(): FormControl {
    return this.configForm.get('enable') as FormControl;
  }

  get type(): FormControl {
    return this.configForm.get('tool')?.get('type') as FormControl;
  }
}
