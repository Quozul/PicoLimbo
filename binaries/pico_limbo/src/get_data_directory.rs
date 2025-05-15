use std::path::PathBuf;

pub fn get_data_directory() -> PathBuf {
    std::env::var_os("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./assets"))
}
