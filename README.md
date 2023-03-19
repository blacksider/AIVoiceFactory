# AI语音工厂

本项目为基于文本自动生成语音的自动化工具。

## 依赖条件

- 默认以 VoiceVox-Engine 作为AI语音后端
- 可以启用自动翻译功能翻译输入文本为目标语言（日语），比如使用百度开放平台API

## 运行 VoiceVox-Engine

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

- 检查文档：

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

- 检查文档：

```
http://localhost:9000/docs
```
