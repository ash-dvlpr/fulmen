use test_app::app::Application;

fn main() {
    let (app, e_loop) = Application::new();
    app.run(e_loop);
}
