use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Capture microphone and provide PCM16 samples via a callback
pub fn capture<F>(mut callback: F) -> Result<()>
where
    F: FnMut(&[i16]) + Send + 'static,
{
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device");
    let config = device.default_input_config()?;

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| callback(data),
            move |err| eprintln!("Input error: {:?}", err),
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                let pcm: Vec<i16> = data.iter().map(|s| (*s * i16::MAX as f32) as i16).collect();
                callback(&pcm)
            },
            move |err| eprintln!("Input error: {:?}", err),
        )?,
        _ => panic!("Unsupported input format"),
    };

    stream.play()?;
    std::thread::park(); // keep running
    Ok(())
}

/// Play PCM16 samples to default output device
pub fn play(samples: &[i16]) -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config()?;

    match config.sample_format() {
        cpal::SampleFormat::I16 => {
            let stream = device.build_output_stream(
                &config.into(),
                move |out: &mut [i16], _| {
                    for (i, sample) in out.iter_mut().enumerate() {
                        *sample = samples.get(i).copied().unwrap_or(0);
                    }
                },
                move |err| eprintln!("Output error: {:?}", err),
            )?;
            stream.play()?;
        }
        cpal::SampleFormat::F32 => {
            let stream = device.build_output_stream(
                &config.into(),
                move |out: &mut [f32], _| {
                    for (i, sample) in out.iter_mut().enumerate() {
                        let s = samples.get(i).copied().unwrap_or(0);
                        *sample = s as f32 / i16::MAX as f32;
                    }
                },
                move |err| eprintln!("Output error: {:?}", err),
            )?;
            stream.play()?;
        }
        _ => panic!("Unsupported output format"),
    }

    Ok(())
}
