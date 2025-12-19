use slint::{ModelRc, SharedString};
use crate::logic::data_types::playlist::AudioEntry;

impl From<AudioEntry> for (SharedString, i32) {
    fn from(entry: AudioEntry) -> Self {
        (SharedString::from(entry.added_at), entry.id)
    }
}

pub fn convert_to_slint_model<T, E>(vec: Vec<T>) -> ModelRc<E>
where
	E: From<T> + Clone + 'static
{
	let slint_vec: Vec<E> = vec.into_iter().map(|item| E::from(item)).collect();
    ModelRc::new(slint::VecModel::from(slint_vec))
}
