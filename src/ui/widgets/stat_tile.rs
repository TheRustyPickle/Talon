use eframe::egui::{
    Align2, CornerRadius, Response, Sense, StrokeKind, TextStyle, Ui, Widget, vec2,
};

pub struct StatTile {
    label: String,
    value: String,
    width: f32,
    height: f32,
}

impl StatTile {
    pub fn new(label: impl Into<String>, value: impl ToString, width: f32) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            width,
            height: 54.0,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl Widget for StatTile {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            label,
            value,
            width,
            height,
        } = self;

        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        if ui.is_rect_visible(rect) {
            let visuals = ui.visuals();
            let radius = CornerRadius::same(8);

            ui.painter()
                .rect_filled(rect, radius, visuals.faint_bg_color);
            ui.painter().rect_stroke(
                rect,
                radius,
                visuals.widgets.noninteractive.bg_stroke,
                StrokeKind::Inside,
            );

            let mut caption = TextStyle::Body.resolve(ui.style());
            caption.size = 13.0;
            let number = TextStyle::Heading.resolve(ui.style());
            let weak = visuals.text_color();
            let strong = visuals.strong_text_color();

            ui.painter().text(
                rect.center_top() + vec2(0.0, 7.0),
                Align2::CENTER_TOP,
                label,
                caption,
                weak,
            );
            ui.painter().text(
                rect.center_bottom() - vec2(0.0, 7.0),
                Align2::CENTER_BOTTOM,
                value,
                number,
                strong,
            );
        }

        response
    }
}
