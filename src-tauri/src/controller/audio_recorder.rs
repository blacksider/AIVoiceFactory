// TODO remove this after recorder finished
#![allow(dead_code)]

use std::io::Cursor;
use std::sync::{Arc, Mutex};

use cpal::{FromSample, Sample, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::controller::errors::{CommonError, ProgramError};

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterMemoryHandle = Arc<Mutex<Option<hound::WavWriter<Cursor<Vec<u8>>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterMemoryHandle)
    where
        T: Sample,
        U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

// TODO currently just for testing if recording works, change to use keyboard event to trigger record later
fn record() -> Result<(), ProgramError> {
    // TODO change to use audio_manager input config
    let device = cpal::default_host()
        .default_input_device()
        .ok_or(ProgramError::wrap(CommonError::new("Failed to get default input device".to_string())))?;
    let config = device
        .default_input_config()?;
    let spec = wav_spec_from_config(&config);
    // write WAV data to memory buffer
    let writer = hound::WavWriter::new(
        Cursor::new(Vec::new()),
        spec,
    )?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // run the input stream on a separate thread.
    let writer_stream = writer.clone();

    let err_fn = move |err| {
        log::error!("An error occurred on input stream: {}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i8, i8>(data, &writer_stream),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &writer_stream),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i32, i32>(data, &writer_stream),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_stream),
            err_fn,
            None,
        )?,
        sample_format => {
            return Err(ProgramError::wrap(CommonError::new(format!("Unsupported sample format '{}'", sample_format))));
        }
    };

    stream.play()?;

    // recording for 5 seconds.
    std::thread::sleep(std::time::Duration::from_secs(5));
    drop(stream);
    writer.lock().unwrap().take().unwrap().finalize().unwrap();
    Ok(())
}
