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
