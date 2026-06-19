// Logic that is responsible for downloading or streaming from source.
use reqwest::Client;
use symphonia::core::audio::sample::SampleBytes;
use symphonia::core::codecs::registry::CodecRegistry;
// use symphonia::core::audio::SampleBuffer;
// use symphonia::core::codecs::DecoderOptions;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use futures::{FutureExt, StreamExt};
use std::{error::Error, time::Duration};
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Probe;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia_core::codecs::audio::AudioDecoderOptions as DecoderOptions;
use symphonia_core::formats::probe::Hint;
use tokio::io::BufReader;
use futures::Stream;

#[tokio::main]
pub async fn stream_from_source(url: String) -> Result<(), Box<dyn Error>> {
    // let audio_url = get_yt_audio_url(&url).await?;

    let client = Client::new();
    let response = client.get(&url).send().await?;
    let bytes = response.bytes().await?;

    let cursor = std::io::Cursor::new(bytes.to_vec());
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let hint = Hint::new();
    let probe = Probe::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let probed = probe.probe(&hint, mss, format_opts, metadata_opts)?;

    let track = probed
        .default_track(symphonia_core::formats::TrackType::Audio)
        .expect("No default track");
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
        Some(Duration::from_secs(10)),
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
