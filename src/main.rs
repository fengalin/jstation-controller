mod midi;

mod jstation;

mod ui;
pub use ui::{app, APP_NAME};

pub fn main() -> iced::Result {
    use iced::Application;

    // FIXME use tracer instead?
    env_logger::Builder::new()
        .filter_module("jstation_controller", log::LevelFilter::Debug)
        .init();

    ui::App::run(iced::Settings::default())
}
