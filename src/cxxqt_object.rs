use super::logic::audio::media_player::MediaPlayer;
use super::logic::ui_events::audio_control_events;
use core::pin::Pin;
use cxx_qt_lib::QString;
use main::State;
use std::path::PathBuf;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        // include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        // #[qobject]
        // #[qml_element]
        // #[qproperty(i32, number)]
        // #[qproperty(QString, string)]
        // #[namespace = "my_object"]
        // type MyObject = super::MyObjectRust;

        #[qobject]
        #[qproperty(QString, index_json)]
        #[qproperty(QString, queue_json)]
        #[qproperty(QString, playlists_json)]
        #[namespace = "q_state"]
        type QState = super::State;

        #[qobject]
        #[qml_element]
        #[namespace = "q_media_controller"]
        type QMediaController = super::MediaPlayer;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_name = "handleMediaStart"]
        fn handle_media_start(self: Pin<&mut QMediaController>, id: i32);

        #[qinvokable]
        #[cxx_name = "handleMediaChange"]
        fn handle_media_change(self: Pin<&mut QMediaController>, index: i32);

        #[qinvokable]
        #[cxx_name = "handleMediaToggle"]
        fn handle_media_toggle(self: Pin<&mut QMediaController>);

        #[qinvokable]
        #[cxx_name = "handleMediaChangeVolume"]
        fn handle_media_change_volume(self: Pin<&mut QMediaController>, volume: i32);

        #[qinvokable]
        #[cxx_name = "handleMediaChangeTrackPosition"]
        fn handle_media_change_track_position(self: Pin<&mut QMediaController>, position: f32);

        #[qinvokable]
        #[cxx_name = "handleMediaGetTrackPosition"]
        fn handle_media_get_track_position(self: &QMediaController) -> i32;

        #[qinvokable]
        #[cxx_name = "handleMediaMix"]
        fn handle_media_mix(self: Pin<&mut QMediaController>);

        #[qinvokable]
        #[cxx_name = "handleNewLocalSource"]
        fn handle_new_local_source(self: Pin<&mut QMediaController>);

        #[qinvokable]
        #[cxx_name = "handleAddToPlaylist"]
        fn handle_add_to_playlist(
            self: Pin<&mut QMediaController>,
            playlist_id: i32,
            media_id: i32,
        );
    }
}

impl gobject::QState {
    pub fn index_json(&self) -> QString {
        let json = serde_json::to_string(&self.index).unwrap();
        QString::from(&json)
    }

    pub fn queue_json(&self) -> QString {
        let json = serde_json::to_string(&self.queue).unwrap();
        QString::from(&json)
    }

    pub fn playlists_json(&self) -> QString {
        let json = serde_json::to_string(&self.playlists).unwrap();
        QString::from(&json)
    }
}

impl qobject::QMediaController {
    pub fn handle_media_start(self: Pin<&mut Self>, id: i32) {
        let mut state = self.state.lock().unwrap();
        let mut player = self.player.lock().unwrap();
        if let Some((index, media)) = state
            .index
            .iter()
            .enumerate()
            .find(|(_, item)| item.id == id)
        {
            audio_control_events::handle_media_start(&mut *player, media);
            state.add_to_queue(media);
            state.playing.media_index = Some(index);
            if let Some((queue_index, _)) = state
                .queue
                .iter()
                .enumerate()
                .find(|(_, queue_item)| queue_item.media_id == media.id)
            {
                state.playing.queue_index = Some(queue_index);
            }
        }
    }

    pub fn handle_media_change(self: Pin<&mut Self>, index: i32) {
        println!("(UI Events) Media changed");
        let mut state = self.state.lock().unwrap();
        let mut player = self.player.lock().unwrap();
        let queue_result = state.update_playing_audio_in_queue(index);
        if let Some((previous_index, target_index)) = queue_result {
            if !state.queue.is_empty() {
                let id = state.queue[target_index].media_id;
                if let Some((_, media)) = state.find_source_by_id(id) {
                    audio_control_events::handle_media_change(
                        &mut *player,
                        media,
                        (previous_index, target_index),
                    );
                }
            }
        }
    }

    pub fn handle_media_toggle(self: Pin<&mut Self>) {
        let mut player = self.player.lock().unwrap();
        audio_control_events::handle_media_toggle(&mut *player);
    }

    pub fn handle_media_change_volume(self: Pin<&mut Self>, volume: i32) {
        let mut player = self.player.lock().unwrap();
        audio_control_events::handle_media_volume(&mut *player, volume);
    }

    pub fn handle_media_change_track_position(self: Pin<&mut Self>, position: f32) {
        let mut player = self.player.lock().unwrap();
        let duration_position = std::time::Duration::from_secs_f32(position);
        audio_control_events::change_current_track_position(&mut *player, duration_position);
    }

    pub fn handle_media_get_track_position(&self) -> i32 {
        println!("(Event) Get track triggered");
        let player = self.player.lock().unwrap();
        let value = audio_control_events::get_current_track_position(&*player);
        self.state
            .lock()
            .unwrap()
            .playing
            .update_position(value as i32);
        value as i32
    }

    pub fn handle_media_mix(self: Pin<&mut Self>) {
        let mut state = self.state.lock().unwrap();
        let player = self.player.lock().unwrap();
        state.shuffle();
        audio_control_events::handle_media_mix(&*player);
    }

    pub fn handle_new_local_source(self: Pin<&mut Self>) {
        let mut state = self.state.lock().unwrap();
        let source: Option<PathBuf> = source::new_local_source();
        match source {
            Some(source) => {
                let path_string = source.to_str().unwrap().to_string();
                database::add_record(Source {
                    origin: String::from("local"),
                    path: path_string,
                });
                println!("Directory fetched correctly {:?}", source);
                let records = source::read_source(source).expect("Couldn't fetch all files");
                database::add_records(records.clone());
                state.merge_to_index(records);
            }
            None => println!("Didn't receive a path. Result should be None: {:?}", source),
        }
    }

    pub fn handle_add_to_playlist(self: Pin<&mut Self>, playlist_id: i32, media_id: i32) {
        let mut state = self.state.lock().unwrap();
        state.add_to_playlist(playlist_id, media_id);
    }
}
