export class AutoTranslationConfig {
  enable!: boolean;
  tool!: AutoTranslateTool;
}

export abstract class AutoTranslateTool {
  type!: string;
}

export class Translator {
  type!: string;
  name!: string;

  constructor(type: string, name: string) {
    this.type = type;
    this.name = name;
  }
}

export const TranslatorTypes: {[key: string]: Translator} = {
  Baidu: new Translator("Baidu", "百度通用翻译")
};

export const BaiduLanguages = [
  {"key": "auto", "name": "自动检测"},
  {"key": "zh", "name": "中文"},
  {"key": "en", "name": "英语"},
  {"key": "yue", "name": "粤语"},
  {"key": "wyw", "name": "文言文"},
  {"key": "jp", "name": "日语"},
  {"key": "kor", "name": "韩语"},
  {"key": "fra", "name": "法语"},
  {"key": "spa", "name": "西班牙语"},
  {"key": "th", "name": "泰语"},
  {"key": "ara", "name": "阿拉伯语"},
  {"key": "ru", "name": "俄语"},
  {"key": "pt", "name": "葡萄牙语"},
  {"key": "de", "name": "德语"},
  {"key": "it", "name": "意大利语"},
  {"key": "el", "name": "希腊语"},
  {"key": "nl", "name": "荷兰语"},
  {"key": "pl", "name": "波兰语"},
  {"key": "bul", "name": "保加利亚语"},
  {"key": "est", "name": "爱沙尼亚语"},
  {"key": "dan", "name": "丹麦语"},
  {"key": "fin", "name": "芬兰语"},
  {"key": "cs", "name": "捷克语"},
  {"key": "rom", "name": "罗马尼亚语"},
  {"key": "slo", "name": "斯洛文尼亚语"},
  {"key": "swe", "name": "瑞典语"},
  {"key": "hu", "name": "匈牙利语"},
  {"key": "cht", "name": "繁体中文"}
]

export class TranslateByBaidu extends AutoTranslateTool {
  apiAddr!: string;
  appId!: string;
  secret!: string;
  from!: string;
  to!: string;

  constructor() {
    super();
    this.type = TranslatorTypes['Baidu'].type;
  }
}
