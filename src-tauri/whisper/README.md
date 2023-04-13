## whisper binary 使用说明

> whisper 二进制使用的是 [ggerganov/whisper.cpp](https://github.com/ggerganov/whisper.cpp)，当前使用的release版本为 `v1.2.1`  
> 在使用whisper的binary模式时，会加载此目录下的 `whisper.dll` 文件，相关模型文件会从 
> [huggingface/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp) 下动态下载，
> 当然也可以使用手动放到 `whiser` 目录下的 `models` 目录下
