use engine::prelude::*;

mod logging;

fn main() {
    logging::init();

    App::default()
        .with_app_name("test_app")
        .run();
}

