// Logic that is responsible for downloading or streaming from source.
use reqwest::Client;
use std::error::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia_core::audio::{Audio, GenericAudioBufferRef};
use symphonia_core::codecs::audio::AudioDecoderOptions;
use symphonia_core::formats::probe::Hint;

pub async fn stream_from_source(url: String) -> Result<(), Box<dyn Error>> {
    // let audio_url = get_audio_bytes_from_url(&url).await?;
    let client = Client::new();
    let response = client.get(&url).send().await?;
    println!("response: {:?}", response);
    let bytes = response.bytes().await?;

    let cursor = std::io::Cursor::new(bytes.to_vec());
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let mut format =
        symphonia::default::get_probe().probe(&hint, mss, format_opts, metadata_opts)?;

    let track = format
        .default_track(symphonia_core::formats::TrackType::Audio)
        .ok_or("No default track found")?;
    let track_id = track.id;
    let codec_params = track
        .codec_params
        .as_ref()
        .and_then(|p| p.audio())
        .ok_or("Track has no audio codec parametrs")?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(codec_params, &AudioDecoderOptions::default())
        .expect("Failed to create decoder");

    while let Some(packet) = format.next_packet()? {
        if packet.track_id != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
            	println!("Decoder decode: Ok");
                let interleaved: Vec<f32> = match decoded {
                    GenericAudioBufferRef::F32(buf) => {
                        let mut out = vec![0f32; buf.frames() * buf.num_planes()];
                        buf.copy_to_slice_interleaved(&mut out);
                        out
                    }
                    other => {
                        let mut out = vec![0f32; other.frames() * other.num_planes()];
                        other.copy_to_slice_interleaved(&mut out);
                        out
                    }
                };
                let _ = interleaved;
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => {
            	println!("Decoder decode: Err");
            	continue
            },
            Err(e) => {
            println!("Decoder decode: Err 2");
            return Err(e.into())
            },
        }
    }

    Ok(())
}
