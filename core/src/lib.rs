use std::path::PathBuf;

use once_cell::sync::Lazy;

pub static BOX_PATH: Lazy<PathBuf> = Lazy::new(|| home::home_dir().expect("Home dir").join(".box"));
