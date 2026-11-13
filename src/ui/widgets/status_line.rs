use eframe::egui::{Align, Color32, Label, Layout, Response, Sense, Ui, UiBuilder, Widget, vec2};

/// Single line status text with a colored dot in front. Long text is cut off and the
/// full text is shown on hover
pub struct StatusLine {
    text: String,
    dot: Color32,
    width: f32,
}

impl StatusLine {
    pub fn new(text: impl Into<String>, dot: Color32, width: f32) -> Self {
        Self {
            text: text.into(),
            dot,
            width,
        }
    }
}

impl Widget for StatusLine {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { text, dot, width } = self;
        let size = vec2(width.max(0.0), ui.spacing().interact_size.y);

        // The full width is reserved up front. Sizing it by its content would make whatever
        // comes next move around every time the text changes
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());

        ui.scope_builder(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                let (dot_rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 4.0, dot);

                ui.add(Label::new(text).truncate());
            },
        );

        response
    }
}
