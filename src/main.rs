// Prevent console window from showing up on Windows
#![windows_subsystem = "windows"]

mod midi;

mod jstation;

mod ui;
pub use ui::{app, APP_NAME};

pub fn main() -> iced::Result {
    // FIXME use tracer instead?
    env_logger::Builder::new()
        .filter_module("jstation_controller", log::LevelFilter::Debug)
        .init();

    use iced::Application;
    ui::App::run(iced::Settings {
        id: Some("org.fengalin.jstation-controller".to_string()),
        window: iced::window::Settings {
            size: (800, 800),
            ..Default::default()
        },
        ..Default::default()
    })
}
