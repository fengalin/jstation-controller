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
        window: iced::window::Settings {
            size: (800, 780),
            ..Default::default()
        },
        ..Default::default()
    })
}
