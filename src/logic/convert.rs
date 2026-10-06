use std::{fmt::Display, str::FromStr};
use slint::{ModelRc, VecModel};

pub fn format_into_time(seconds: f64) -> String {
	let total = seconds.max(0.0).round() as u64;
 	let hours = total / 3600;
  	let minutes = (total % 3600) / 60;
   	let secs = total % 60;

    if hours > 0 {
    	format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

pub fn list_into_string<T>(list: &[T], separator: &str) -> String
where
    T: Display,
{
    list
    	.into_iter()
        .map(|item| item.to_string())
        .collect::<Vec<String>>()
        .join(separator)
}

pub fn to_model_rc<T>(items: Vec<T>) -> ModelRc<T>
where
    T: Clone + 'static,
{
    ModelRc::new(VecModel::from(items))
}

pub fn string_into_list<T>(string: &str, separator: &str) -> Vec<T>
where
     T: FromStr,
{
     string
         .split(&separator as &str)
         .filter_map(|s| s.trim().parse::<T>().ok())
         .collect()
}
