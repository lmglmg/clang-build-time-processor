mod gui;
mod processing;
mod model;
use crate::gui::App;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(App::title, App::update, App::view)
        .font(include_bytes!("../fonts/JetBrainsMono-Regular.ttf").as_slice())
        .centered()
        .run_with(App::new)
}