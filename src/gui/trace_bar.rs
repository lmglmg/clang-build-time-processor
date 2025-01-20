use iced::{mouse, widget::canvas, Renderer, Theme};

#[derive(Debug)]
pub struct TraceBar {
    pub start: f32,
    pub end: f32,
}

impl<Message> canvas::Program<Message> for TraceBar {
    type State = ();


    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let bg_line = canvas::Path::rounded_rectangle(
            iced::Point::new(0.0, bounds.height * 0.4),
            iced::Size::new(bounds.width, bounds.height * 0.2),
            iced::border::Radius {
                top_left: 2.0,
                top_right: 2.0,
                bottom_right: 2.0,
                bottom_left: 2.0
            }
        );

        frame.fill(&bg_line, theme.extended_palette().background.strong.color);

        let duration_bar = canvas::Path::rounded_rectangle(
            iced::Point::new(self.start * bounds.width, 0.0),
            iced::Size::new((self.end - self.start) * bounds.width, bounds.height),
            iced::border::Radius {
                top_left: 2.0,
                top_right: 2.0,
                bottom_right: 2.0,
                bottom_left: 2.0
            }
        );

        frame.fill(&duration_bar, theme.extended_palette().primary.strong.color);

        vec![frame.into_geometry()]
    }
}
