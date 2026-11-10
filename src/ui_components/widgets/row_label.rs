use eframe::egui::{
    Color32, CornerRadius, Response, Role, Sense, StrokeKind, TextStyle, Ui, Widget, WidgetInfo,
    WidgetText,
};
use eframe::emath::NumExt;

const FADE_TIME: f32 = 0.12;

pub struct RowLabel {
    text: WidgetText,
    selected: bool,
    whitelisted: bool,
}

impl RowLabel {
    pub fn new(selected: bool, whitelisted: bool, text: impl Into<WidgetText>) -> Self {
        Self {
            selected,
            text: text.into(),
            whitelisted,
        }
    }
}

impl Widget for RowLabel {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            selected,
            text,
            whitelisted,
        } = self;
        let button_padding = ui.spacing().button_padding;
        let total_extra = button_padding + button_padding;

        let wrap_width = ui.available_width() - total_extra.x;
        let text = text.into_galley(ui, None, wrap_width, TextStyle::Button);

        let mut desired_size = total_extra + text.size();
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());
        response.widget_info(|| {
            WidgetInfo::selected(Role::Button, ui.is_enabled(), selected, text.text())
        });

        if ui.is_rect_visible(response.rect) {
            let text_pos = ui
                .layout()
                .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                .min;

            let visuals = ui.visuals();
            let dark = visuals.dark_mode;
            let accent = if dark {
                Color32::from_rgb(232, 140, 90)
            } else {
                Color32::from_rgb(214, 106, 52)
            };

            let hover_t = ui.ctx().animate_bool_with_time(
                response.id.with("hover"),
                response.hovered() || response.highlighted() || response.has_focus(),
                FADE_TIME,
            );

            // Cover the spacing between cells so adjacent cells join up
            let band = rect.expand2(ui.spacing().item_spacing / 2.0);
            let radius = CornerRadius::same(4);
            let hover_fill = visuals.widgets.hovered.weak_bg_fill;

            if whitelisted {
                let tint = if dark { 0.20 } else { 0.18 };
                ui.painter()
                    .rect_filled(band, radius, accent.gamma_multiply(tint));
            }

            if hover_t > 0.0 && !selected {
                let fill = if whitelisted {
                    accent.gamma_multiply(0.18 * hover_t)
                } else {
                    hover_fill.gamma_multiply(hover_t)
                };
                ui.painter().rect_filled(band, radius, fill);
            }

            if selected {
                let selection = visuals.selection;
                ui.painter()
                    .rect_filled(band, radius, selection.bg_fill.gamma_multiply(0.85));
                ui.painter().rect_stroke(
                    band,
                    radius,
                    eframe::egui::Stroke::new(1.0, selection.stroke.color.gamma_multiply(0.6)),
                    StrokeKind::Inside,
                );
            }

            let text_color = if selected {
                visuals.selection.stroke.color
            } else {
                visuals.widgets.inactive.text_color()
            };
            ui.painter().galley(text_pos, text, text_color);
        }

        response
    }
}
