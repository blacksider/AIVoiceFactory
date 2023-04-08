# AI语音工厂

本项目为基于文本自动生成语音的自动化工具，支持TextToSpeech，语音识别，文本翻译等功能。

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

## cpal 本地库

> 仓库内置了一个 [cpal 本地库](src-tauri%2Fcpal)，是为了支持ASIO。  
> 因为参照官方设置下载[asiosdk_2.3.3](https://download.steinberg.net/sdk_downloads/asiosdk_2.3.3_2019-06-14.zip)
> 后，无法编译出 `asio.h` 中的匿名枚举变量，需要修改 [asio-sys](src-tauri%2Fcpal%2Fasio-sys) 的 `bindgen` 版本至 `0.64.0`，
> 才可以正常工作，因此将官方的master分支代码复制到了本地，修改 `asio-sys` 中的 `bindgen` 的版本，
> 并修改 `build.rs` 脚本以兼容 `asiosdk_2.3.3`

> 本地兼容支持操作如下：

- 下载 [ASIO SDK](https://download.steinberg.net/sdk_downloads/asiosdk_2.3.3_2019-06-14.zip)
- 解压 SDK 到任意位置，设置环境变量 `CPAL_ASIO_DIR` 为该位置
- 下载编译器 [LLVM](https://objects.githubusercontent.com/github-production-release-asset-2e65be/75821432/d9c31b94-2703-44fe-bd38-4c1a031a6590?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIWNJYAX4CSVEH53A%2F20230406%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20230406T083628Z&X-Amz-Expires=300&X-Amz-Signature=ba075a45827ea3086e1708b53a1b31f67b01490fb5b3b528dcbaea42ae126e6d&X-Amz-SignedHeaders=host&actor_id=12133944&key_id=0&repo_id=75821432&response-content-disposition=attachment%3B%20filename%3DLLVM-16.0.1-win64.exe&response-content-type=application%2Foctet-stream)
- 安装 LLVM 后，设置环境变量 `LIBCLANG_PATH` 为安装目录下的 `bin` 目录，如：`"C:\Program Files\LLVM\bin"`
- 如果本机没有任何ASIO驱动设备（像一般的支持ASIO的声卡，或者voicemeeter之类的软件都会支持），可以下载平民版驱动 [ASIO4ALL](https://www.asio4all.org/downloads_11/ASIO4ALL_2_15_SCN.exe)
- 加载 `VCVARS`，例如装了 `Microsoft Visual Studio Community 2022`，则进入安装目录找到 `vcvarsall.bat`， 
例如 `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat`
- 64位环境下运行
```
./vcvarsall.bat amd64
```

- 32位下运行：

```
./vcvarsall.bat x86
```

## 如何编译

> 运行 `pnpm run release` 编译程序  

> Windows下如果出现 `wix311-binaries` 无法下载的情况，
> 可以手动下载 `https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311-binaries.zip` ，
> 并将该文件内容解压到 `C:\Users\Your-Username\AppData\Local\tauri\WixTools` 下即可

## TODO

待实现功能：

1. 支持ChatGPT API   
2. 支持其他语音生成AI  
3. 支持ASIO
4. 功能组件插件化以支持自定义扩展功能

......
