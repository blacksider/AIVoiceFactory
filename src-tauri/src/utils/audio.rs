use anyhow::{anyhow, Result};
use cpal::traits::DeviceTrait;

/// convert source samples to mono samples if source is duo-channels.
/// assume source channel is 2, the data is like \[l, r, l, r...]
/// all we need to do is average every \[l, r] data
pub fn convert_to_mono(samples: &Vec<f32>, channels: u16) -> Vec<f32> {
    let mut converted = Vec::new();
    for i in (0..samples.len()).step_by(channels as usize) {
        let mut total = 0.0;
        for it in 0..channels {
            total += samples[i + it as usize];
        }
        // average all channels
        let mono_sample = total / (channels as f32);
        converted.push(mono_sample);
    }
    converted
}

fn build_cpal_input_stream<T, D>(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    mut data_callback: D,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample + dasp_sample::conv::ToSample<f32>,
    D: FnMut(Vec<f32>) + Send + 'static,
{
    let err_fn = |err| log::error!("an error occurred on the input stream: {}", err);
    let stream_config = cpal::StreamConfig {
        channels: config.channels(),
        sample_rate: config.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };
    let stream = device.build_input_stream(
        &stream_config,
        move |data: &[T], _: &_| {
            let buffer: Vec<f32> = data.iter().map(|s| T::to_sample(*s)).collect();
            data_callback(buffer);
        },
        err_fn,
        None,
    )?;
    Ok(stream)
}

pub fn build_input_stream<D>(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    data_callback: D,
) -> Result<cpal::Stream>
where
    D: FnMut(Vec<f32>) + Send + 'static,
{
    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => build_cpal_input_stream::<i8, D>(device, config, data_callback)?,
        cpal::SampleFormat::I16 => {
            build_cpal_input_stream::<i16, D>(device, config, data_callback)?
        }
        cpal::SampleFormat::I32 => {
            build_cpal_input_stream::<i32, D>(device, config, data_callback)?
        }
        cpal::SampleFormat::I64 => {
            build_cpal_input_stream::<i64, D>(device, config, data_callback)?
        }
        cpal::SampleFormat::U8 => build_cpal_input_stream::<u8, D>(device, config, data_callback)?,
        cpal::SampleFormat::U16 => {
            build_cpal_input_stream::<u16, D>(device, config, data_callback)?
        }
        cpal::SampleFormat::U32 => {
            build_cpal_input_stream::<u32, D>(device, config, data_callback)?
        }
        cpal::SampleFormat::U64 => {
            build_cpal_input_stream::<u64, D>(device, config, data_callback)?
        }
        cpal::SampleFormat::F32 => {
            build_cpal_input_stream::<f32, D>(device, config, data_callback)?
        }
        cpal::SampleFormat::F64 => {
            build_cpal_input_stream::<f64, D>(device, config, data_callback)?
        }
        sample_format => {
            return Err(anyhow!("Unsupported sample format '{}'", sample_format));
        }
    };
    Ok(stream)
}
