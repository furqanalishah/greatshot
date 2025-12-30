mod ui;
mod editor;

const APP_ID: &str = "io.github.syed.greatshot";

fn main() {
    use adw::prelude::*;

    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui::build_ui);
    app.run();
}
