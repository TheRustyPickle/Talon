use eframe::egui::{Response, Sense, TextStyle, Ui, Widget, WidgetInfo, WidgetText, WidgetType};
use eframe::emath::NumExt;
use eframe::epaint::Color32;

/// Copy of SelectableLabel source with minor modifications
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
            WidgetInfo::selected(WidgetType::SelectableLabel, selected, text.text())
        });

        if ui.is_rect_visible(response.rect) {
            let text_pos = ui
                .layout()
                .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                .min;

            let visuals = ui.style().interact_selectable(&response, selected);

            if selected
                || response.highlighted()
                || response.has_focus()
                || (response.hovered() && !whitelisted)
            {
                let rect = rect.expand(visuals.expansion);

                ui.painter().rect(
                    rect,
                    visuals.rounding,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                );
            } else if response.hovered() && whitelisted {
                let rect = rect.expand(visuals.expansion);
                ui.painter().rect(
                    rect,
                    visuals.rounding,
                    Color32::from_rgb(255, 127, 80),
                    visuals.bg_stroke,
                );
            } else if whitelisted {
                let rect = rect.expand(visuals.expansion);
                let is_dark_theme = ui.visuals().dark_mode;

                let color = if is_dark_theme {
                    Color32::from_rgb(168, 100, 66)
                } else {
                    Color32::from_rgb(255, 160, 122)
                };

                ui.painter()
                    .rect(rect, visuals.rounding, color, visuals.bg_stroke);
            }

            ui.painter().galley(text_pos, text, visuals.text_color())
        }

        response
    }
}
