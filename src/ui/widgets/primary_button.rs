use eframe::egui::{
    Align2, Color32, CornerRadius, CursorIcon, Response, Role, Sense, TextStyle, Ui, Vec2, Widget,
    WidgetInfo,
};

pub struct PrimaryButton {
    text: String,
    size: Vec2,
    danger: bool,
}

impl PrimaryButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: Vec2::new(100.0, 36.0),
            danger: false,
        }
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }
}

impl Widget for PrimaryButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { text, size, danger } = self;

        let enabled = ui.is_enabled();
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };

        let (rect, response) = ui.allocate_exact_size(size, sense);
        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, &text));

        if ui.is_rect_visible(rect) {
            let visuals = ui.visuals();

            let base = if danger {
                Color32::from_rgb(190, 70, 70)
            } else {
                visuals.selection.bg_fill
            };
            let hover = base.lerp_to_gamma(Color32::WHITE, 0.18);
            let pressed = base.lerp_to_gamma(Color32::BLACK, 0.15);

            let hover_t = ui.ctx().animate_bool_with_time(
                response.id.with("hover"),
                response.hovered() && enabled,
                0.12,
            );

            let mut fill = base.lerp_to_gamma(hover, hover_t);
            if response.is_pointer_button_down_on() {
                fill = pressed;
            }

            let mut text_color = visuals.selection.stroke.color;
            if danger {
                text_color = Color32::WHITE;
            }
            if !enabled {
                fill = visuals.widgets.noninteractive.weak_bg_fill;
                text_color = visuals.widgets.noninteractive.fg_stroke.color;
            }

            ui.painter().rect_filled(rect, CornerRadius::same(8), fill);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                &text,
                TextStyle::Button.resolve(ui.style()),
                text_color,
            );
        }

        if enabled {
            response.on_hover_cursor(CursorIcon::PointingHand)
        } else {
            response
        }
    }
}
