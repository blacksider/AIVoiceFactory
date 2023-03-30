# AI语音工厂

本项目为基于文本自动生成语音的自动化工具。

## 依赖条件

- 默认以 VoiceVox-Engine 作为AI语音后端
- 可以启用自动翻译功能翻译输入文本为目标语言（日语），比如使用百度开放平台API

## 运行 VoiceVox-Engine

VoiceVox为一个日文语音生成AI，主要的输出语言为日语。
可以用 Docker 容器运行 [VoiceVox-Engine](https://github.com/VOICEVOX/voicevox_engine)。

如何运行：

- GPU版本：
```
docker pull voicevox/voicevox_engine:nvidia-ubuntu20.04-latest
docker run --name voicevox -d -t --gpus all --env NVIDIA_DISABLE_REQUIRE=1 -p '127.0.0.1:50021:50021' voicevox/voicevox_engine:nvidia-ubuntu20.04-latest
```

- CPU版本：
```
docker pull voicevox/voicevox_engine:cpu-ubuntu20.04-latest
docker run --name voicevox -d -t -p '127.0.0.1:50021:50021' -e 'VV_CPU_NUM_THREADS=4' voicevox/voicevox_engine:cpu-ubuntu20.04-latest
```

- 检查API文档：

```
http://localhost:50021/docs
```

## 运行 Whisper

可以用 Docker 容器运行 [Whisper](https://github.com/openai/whisper)

如何运行：

- GPU版本：
```
docker pull onerahmet/openai-whisper-asr-webservice:latest-gpu
docker run --name whisper -d -t --gpus all --env NVIDIA_DISABLE_REQUIRE=1 -p 9000:9000 -e ASR_MODEL=base onerahmet/openai-whisper-asr-webservice:latest-gpu
```

- CPU版本：
```
docker pull onerahmet/openai-whisper-asr-webservice:latest
docker run --name whisper -d -t -p 9000:9000 -e ASR_MODEL=base onerahmet/openai-whisper-asr-webservice:latest
```

- 检查API文档：

```
http://localhost:9000/docs
```

## 开发与测试

> 运行 `pnpm install` 安装本地前端依赖  
> 进入 `src-tauri` 目录下，运行 `cargo update` 更新 cargo 依赖  
> 运行 `pnpm run dev` 运行程序

## 如何编译

> 运行 `pnpm run release` 编译程序  

> Windows下如果出现 `wix311-binaries` 无法下载的情况，
> 可以手动下载 `https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311-binaries.zip` ，
> 并将该文件内容解压到 `C:\Users\Your-Username\AppData\Local\tauri\WixTools` 下即可

## TODO

待实现功能：

1. 对接whisper实现语音自动识别  
2. 支持ChatGPT API   
3. 支持其他语音生成AI  
4. 功能组件插件化以支持自定义扩展功能  
......
