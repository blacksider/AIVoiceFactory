import {Component, OnInit} from '@angular/core';
import {AutoTranslationConfig, BaiduLanguages, TranslateByBaidu, Translator, TranslatorTypes} from './auto-translation';
import {ActivatedRoute} from '@angular/router';

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

  constructor(private activatedRoute: ActivatedRoute) {
  }

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(
      ({config}) => {
        this.config = config as AutoTranslationConfig;
        if (this.config.tool.type === TranslatorTypes['Baidu'].type) {
          this.translateByBaidu = this.config.tool as TranslateByBaidu;
        }
      });
    BaiduLanguages.forEach((value, index) => {
      if (index > 0) {
        this.baiduToLanguageTypes.push(value.key);
      }
      this.baiduFromLanguageTypes.push(value.key);
      this.baiduLanguages[value.key] = value.name;
    });
  }
}
