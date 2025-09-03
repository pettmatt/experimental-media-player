use rodio::{self, Decoder, OutputStream, Sink};
use std::{fs::File, io::BufReader, path::Path, time::Duration};

// Rodio docs: https://docs.rs/rodio/latest/rodio/

pub fn open_stream() -> (OutputStream, Sink) {
	let output_stream = rodio::OutputStreamBuilder::open_default_stream()
		.expect("Open default audio stream");
	let sink = Sink::connect_new(output_stream.mixer());
	(output_stream, sink)
}

// Note: The sound plays in a separate audio thread,
// so we need to keep the main thread alive while it's playing.

pub fn start_playing_audio(sink: Sink, audio_path: &Path) {
	if let Ok(file) = File::open(audio_path) {
		let source = Decoder::try_from(file).unwrap();
		sink.append(source);
		sink.play();
	} else {
		println!("Couldn't open audio file: {:?}", audio_path);
	}
}

pub fn continue_audio(sink: &Sink) {
	if sink.is_paused() {
		sink.play();
	}
}

pub fn pause_audio(sink: &Sink) {
	if !sink.is_paused() {
		sink.pause();
	}
}

pub fn add_to_queue(sink: &Sink, source: Decoder<BufReader<File>>) {
	sink.append(source);
}

pub fn current_track_position(sink: &Sink) -> u128 {
	sink.get_pos().as_millis()
}

pub fn clear_queue(sink: &Sink) {
	sink.clear();
}

pub fn set_volume(sink: &Sink, value: f32) {
	sink.set_volume(value);
}

pub fn skip_to_next(sink: &Sink) {
	sink.skip_one();
}

pub fn current_track_position_change(sink: &Sink, position: Duration) {
	sink.try_seek(position);
}

pub fn destroy_sink(sink: Sink) {
	sink.detach();
}
