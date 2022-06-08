use test_app::app::Application;

fn main() {
    let (e_loop, app) = Application::new();
    app.run(e_loop);
}
