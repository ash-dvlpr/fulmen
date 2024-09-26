use log::*;
// use std::io::Write;

pub fn init() {
    let mut builder = env_logger::builder();

    builder
        .filter_level(LevelFilter::max())
        // .format(|buf, record| {
        //     writeln!(
        //         buf,
        //         // "[{} {} {}] | {}:{} | {}",
        //         // chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
        //         "[{} {}] | {}:{} | {}",
        //         record.level(),
        //         record.target(),

        //         record.file().unwrap_or("unknown"),
        //         record.line().unwrap_or(0),

        //         record.args()
        //     )
        // })
        .format_timestamp_secs()
        .init();
}
