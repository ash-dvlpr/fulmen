mod logging;
use log::*;

fn main() {
    logging::init();
    info!("Hello from Main");


    engine::hello();
}
