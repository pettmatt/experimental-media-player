// Logic that is responsible for downloading or streaming from source.
use reqwest::Client;
use symphonia::core::audio::SampleBuffer;
// use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia_core::codecs::audio::AudioDecoderOptions as DecoderOptions;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use futures::StreamExt;
use std::{error::Error, time::Duration};
use tokio::io::BufReader;

#[tokio::main]
pub async fn stream_from_source() -> Result<(), Box<dyn Error>> {
	let url = "";

	let client = Client::new();
	let response = client.get(url).send().await?;
	let stream = response.chunk(); // .bytes_stream();

	let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    // Set up CPAL for audio playback
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config()?;

    // Create a Symphonia decoder
    let mut mss = MediaSourceStream::new(
        Box::new(BufReader::new(stream.into_async_read())),
        Default::default(),
    );

    // Previously named "probed"
    let format = symphonia::default::get_probe()
        .probe(&hint, mss, format_opts, metadata_opts)
        .expect("Failed to probe format");

    // let mut format = probed.format;
    let track = format.default_track().expect("No default track");
    let decoder = symphonia::default::get_codecs()
        .make_audio_decoder(&track.codec_params, &DecoderOptions::default())
        .expect("Failed to create decoder");

    // Set up CPAL stream
    let sample_rate = track.codec_params.sample_spec().rate;
    let channels = track.codec_params.sample_spec().channels.count();

    let config = cpal::StreamConfig {
        channels: channels as u16,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let (mut stream, stream_handle) = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Fill the buffer with audio data (simplified)
            for sample in data.iter_mut() {
                *sample = 0.0; // Replace with actual audio data
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        Some(Duration::from_secs(10))
    )?;

    stream.play()?;

    // 7. Decode and play the stream
    while let Some(packet) = format.next_packet() {
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let duration = decoded.capacity() as u64;
                let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
                sample_buf.copy_interleaved_ref(decoded);
                // TODO: Send samples to CPAL stream
            }
            Err(_) => continue,
        }
    }

    Ok(())
}
