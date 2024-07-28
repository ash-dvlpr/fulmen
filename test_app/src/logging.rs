use log::*;

pub fn init() {
    let mut builder = env_logger::builder();

    builder
        .filter_level(LevelFilter::max())
        .format_timestamp_secs()
        .init();
}