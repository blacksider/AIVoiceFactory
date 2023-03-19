import {Component, OnInit} from '@angular/core';
import {AutoTranslationConfig, BaiduLanguages, TranslateByBaidu, Translator, TranslatorTypes} from './auto-translation';

@Component({
  selector: 'app-auto-translation',
  templateUrl: './auto-translation.component.html',
  styleUrls: ['./auto-translation.component.less']
})
export class AutoTranslationComponent implements OnInit {
  config!: AutoTranslationConfig;

  translatorTypes: { [key: string]: Translator } = TranslatorTypes;
  translators = Object.keys(TranslatorTypes);

  baiduFromLanguageTypes: string[] = [];
  baiduToLanguageTypes: string[] = [];
  baiduLanguages: { [key: string]: string } = {};

  translateByBaidu?: TranslateByBaidu;

  ngOnInit(): void {
    BaiduLanguages.forEach((value, index) => {
      if (index > 0) {
        this.baiduToLanguageTypes.push(value.key);
      }
      this.baiduFromLanguageTypes.push(value.key);
      this.baiduLanguages[value.key] = value.name;
    });

    // init translation config
    this.config = new AutoTranslationConfig();
    this.config.enable = true;
    this.translateByBaidu = new TranslateByBaidu();
    this.translateByBaidu.from = this.baiduFromLanguageTypes[0];
    this.translateByBaidu.to = 'jp';
    this.config.tool = this.translateByBaidu;
  }
}
