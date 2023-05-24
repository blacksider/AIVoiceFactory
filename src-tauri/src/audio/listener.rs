use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use cpal::{BufferSize, Device, SizedSample, Stream, StreamConfig, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, StreamTrait};
use lazy_static::lazy_static;

use crate::controller::errors::{CommonError, ProgramError};

lazy_static! {
    static ref STREAM_AUDIO: Arc<Mutex<StreamAudio>> = Arc::new(Mutex::new(StreamAudio::new()));
    static ref RUNNING: AtomicBool = AtomicBool::new(false);
}

pub struct StreamAudio {
    m_audio: Vec<f32>,
    m_audio_new: Vec<f32>,
    m_audio_pos: usize,
    m_audio_len: usize,
}

impl StreamAudio {
    pub fn new() -> Self {
        StreamAudio {
            m_audio: vec![],
            m_audio_new: vec![],
            m_audio_pos: 0,
            m_audio_len: 0,
        }
    }

    pub fn reset(&mut self) {
        self.m_audio_len = 0;
        self.m_audio_pos = 0;
    }

    pub fn clear(&mut self) {
        self.reset();
        self.m_audio.clear();
        self.m_audio_new.clear();
    }

    pub fn on_data(&mut self, samples: Vec<f32>) {
        let n_samples = samples.len();

        self.m_audio_new.resize(n_samples, 0.0);

        self.m_audio_new.copy_from_slice(samples.as_slice());

        if self.m_audio_pos + n_samples > self.m_audio.len() {
            let n0 = self.m_audio.len() - self.m_audio_pos;

            self.m_audio[self.m_audio_pos..].copy_from_slice(&self.m_audio_new[..n0]);
            self.m_audio[..n_samples - n0].copy_from_slice(&self.m_audio_new[n0..]);

            self.m_audio_pos = (self.m_audio_pos + n_samples) % self.m_audio.len();
            self.m_audio_len = self.m_audio.len();
        } else {
            self.m_audio[self.m_audio_pos..self.m_audio_pos + n_samples].copy_from_slice(&self.m_audio_new);

            self.m_audio_pos = (self.m_audio_pos + n_samples) % self.m_audio.len();
            self.m_audio_len = self.m_audio_len + n_samples.min(self.m_audio.len() - self.m_audio_len);
        }
    }
}

unsafe impl Send for StreamAudio {}

unsafe impl Sync for StreamAudio {}

pub struct Listener {
    m_len_ms: u32,
    sample_rate: u32,
    stream: Option<Stream>,
}

unsafe impl Send for Listener {}

unsafe impl Sync for Listener {}

impl Listener {
    pub fn new(m_len_ms: u32) -> Listener {
        assert!(m_len_ms > 0, "m_len_ms must be greater than 0");
        Listener {
            m_len_ms,
            sample_rate: 0,
            stream: None,
        }
    }

    fn build_input_stream<T>(
        &mut self,
        device: Device,
        config: &SupportedStreamConfig,
    ) -> Result<Stream, ProgramError>
        where
            T: SizedSample + dasp_sample::conv::ToSample<f32> {
        let err_fn = |err|
            log::error!("an error occurred on the stream: {}", err);
        let stream_config = StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: BufferSize::Default,
        };

        let stream = device.build_input_stream(
            &stream_config,
            |data: &[T], _: &_| {
                if RUNNING.load(Ordering::SeqCst) {
                    let buffer: Vec<f32> = data.iter().map(|s| T::to_sample(*s)).collect();
                    let lock = STREAM_AUDIO.clone();
                    let mut stream_audio = lock.lock().unwrap();
                    stream_audio.on_data(buffer);
                }
            },
            err_fn,
            None,
        )?;
        Ok(stream)
    }

    pub fn init(&mut self, device: Device) -> Result<(), ProgramError> {
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let m_audio_len = (sample_rate * self.m_len_ms) / 1000;

        self.sample_rate = sample_rate;

        // resize vector
        {
            let lock = STREAM_AUDIO.clone();
            let mut stream_audio = lock.lock().unwrap();
            stream_audio.m_audio.resize(m_audio_len as usize, 0.0);
        }

        RUNNING.store(false, Ordering::Release);

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => self.build_input_stream::<i8>(device, &config)?,
            cpal::SampleFormat::I16 => self.build_input_stream::<i16>(device, &config)?,
            cpal::SampleFormat::I32 => self.build_input_stream::<i32>(device, &config)?,
            cpal::SampleFormat::I64 => self.build_input_stream::<i64>(device, &config)?,
            cpal::SampleFormat::U8 => self.build_input_stream::<u8>(device, &config)?,
            cpal::SampleFormat::U16 => self.build_input_stream::<u16>(device, &config)?,
            cpal::SampleFormat::U32 => self.build_input_stream::<u32>(device, &config)?,
            cpal::SampleFormat::U64 => self.build_input_stream::<u64>(device, &config)?,
            cpal::SampleFormat::F32 => self.build_input_stream::<f32>(device, &config)?,
            cpal::SampleFormat::F64 => self.build_input_stream::<f64>(device, &config)?,
            sample_format => {
                return Err(ProgramError::wrap(
                    CommonError::new(format!("Unsupported sample format '{}'", sample_format))));
            }
        };
        self.stream = Some(stream);
        Ok(())
    }

    pub fn resume(&self) -> Result<bool, ProgramError> {
        if RUNNING.load(Ordering::Acquire) {
            log::debug!("Listener already running");
            return Ok(false);
        }

        self.stream.as_ref()
            .ok_or("Failed to get stream reference")?
            .play()
            .map_err(|e| format!("Failed to play stream: {}", e))?;

        RUNNING.store(true, Ordering::Release);

        log::debug!("Listener started");

        Ok(true)
    }

    pub fn pause(&self) -> Result<bool, ProgramError> {
        if !RUNNING.load(Ordering::Acquire) {
            log::debug!("Listener already paused");
            return Ok(false);
        }
        self.stream.as_ref()
            .ok_or("Failed to get stream reference")?
            .pause()
            .map_err(|e| format!("Failed to pause stream: {}", e))?;

        RUNNING.store(false, Ordering::Release);

        log::debug!("Listener paused");

        Ok(true)
    }

    pub fn clear(&mut self) -> bool {
        if !RUNNING.load(Ordering::Acquire) {
            log::debug!("Listener not running");
            return false;
        }

        {
            let lock = STREAM_AUDIO.clone();
            let mut stream_audio = lock.lock().unwrap();
            stream_audio.reset();
        }

        true
    }

    pub fn get(&self, ms: u32, result: &mut Vec<f32>) {
        if !RUNNING.load(Ordering::Acquire) {
            log::debug!("Listener not running");
            return;
        }

        let lock = STREAM_AUDIO.clone();
        let stream_audio = lock.lock().unwrap();

        let n_samples = if ms <= 0 {
            self.m_len_ms * self.sample_rate / 1000
        } else {
            self.sample_rate * ms / 1000
        };

        let n_samples = (n_samples as usize).min(stream_audio.m_audio_len);

        result.resize(n_samples, 0.0);

        let s0 = if n_samples > stream_audio.m_audio_pos {
            stream_audio.m_audio_pos + stream_audio.m_audio.len() - n_samples
        } else {
            stream_audio.m_audio_pos - n_samples
        } % stream_audio.m_audio.len();

        if s0 + n_samples > stream_audio.m_audio.len() {
            let n0 = stream_audio.m_audio.len() - s0;

            result[..n0].copy_from_slice(&stream_audio.m_audio[s0..]);
            result[n0..].copy_from_slice(&stream_audio.m_audio[0..(n_samples - n0)]);
        } else {
            result.copy_from_slice(&stream_audio.m_audio[s0..(s0 + n_samples)]);
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        log::debug!("Listener dropped");
        {
            let lock = STREAM_AUDIO.clone();
            let mut stream_audio = lock.lock().unwrap();
            stream_audio.clear();
        }
    }
}

fn high_pass_filter(data: &mut Vec<f32>, cutoff: f32, sample_rate: f32) {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    let mut y = data[0];

    for i in 1..data.len() {
        y = alpha * (y + data[i] - data[i - 1]);
        data[i] = y;
    }
}

pub fn vad_simple(pcmf32: &mut Vec<f32>, sample_rate: u32, last_ms: u32, vad_thold: f32, freq_thold: f32, verbose: bool) -> bool {
    let n_samples = pcmf32.len();
    let n_samples_last = ((sample_rate * last_ms) / 1000) as usize;

    if n_samples_last >= n_samples {
        // not enough samples - assume no speech
        return false;
    }

    if freq_thold > 0.0 {
        high_pass_filter(pcmf32, freq_thold, sample_rate as f32);
    }

    let energy_all = pcmf32.iter().map(|&x| x.abs()).sum::<f32>() / n_samples as f32;
    let energy_last = pcmf32[n_samples - n_samples_last..].iter().map(|&x| x.abs()).sum::<f32>() / n_samples_last as f32;

    if verbose {
        log::info!("{}: energy_all: {}, energy_last: {}, vad_thold: {}, freq_thold: {}", "vad_simple", energy_all, energy_last, vad_thold, freq_thold)
    }

    energy_last <= vad_thold * energy_all
}
