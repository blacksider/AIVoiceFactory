use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;

use rodio::{Decoder, OutputStream, Sink, Source};

use cpal::traits::{DeviceTrait, HostTrait};

#[test]
fn asio() {
    let host: cpal::Host;
    #[cfg(target_os = "windows")]
    {
        host = cpal::host_from_id(cpal::HostId::Asio).expect("Unable to init asio host");
    }
    let devices = host.output_devices().expect("no outputs?");
    let mut output = None;
    for (i, device) in devices.enumerate() {
        let device_name: String = device.name().unwrap();
        println!("{}: {}", i, device_name);
        // try to locate ASIO4ALL
        if device_name.starts_with("ASIO4ALL") {
            output = Some(device);
            break;
        }
    }
    if output.is_none() {
        // default output is the first device(asio has no concept of default device)
        output = Some(host.default_output_device().unwrap());
    }
    let output = output.unwrap();
    let file = File::open("audio/beep.wav").unwrap();
    let source = BufReader::new(file);
    let decoder = Decoder::new(source).unwrap();
    let (_stream, stream_handle) = OutputStream::try_from_device(&output).expect("Output device not ok");
    let sink = Sink::try_new(&stream_handle).expect("Unable to stream");
    sink.append(decoder.convert_samples::<f32>());
    sink.play();
    sink.sleep_until_end();
}
