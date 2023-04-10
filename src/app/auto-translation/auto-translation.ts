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

export const TranslatorTypes: { [key: string]: Translator } = {
  Baidu: new Translator("Baidu", "百度通用翻译")
};

export class TranslateByBaidu extends AutoTranslateTool {
  api_addr!: string;
  appId!: string;
  secret!: string;
  from!: string;
  to!: string;

  constructor() {
    super();
    this.type = TranslatorTypes['Baidu'].type;
  }
}
