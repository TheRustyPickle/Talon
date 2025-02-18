use eframe::egui::{
    CornerRadius, Id, LayerId, Order, Response, Sense, Stroke, StrokeKind, TextStyle, Ui, Vec2,
    Widget, WidgetInfo, WidgetText, WidgetType,
};

pub struct AnimatedLabel {
    text: WidgetText,
    selected: bool,
    selected_position: Id,
    hover_position: Id,
    x_size: f32,
    y_size: f32,
    rounding: Option<CornerRadius>,
    separator_position: (bool, bool),
}

#[allow(clippy::too_many_arguments)]
impl AnimatedLabel {
    pub fn new(
        selected: bool,
        text: impl Into<WidgetText>,
        selected_position: Id,
        hover_position: Id,
        x_size: f32,
        y_size: f32,
        rounding: Option<CornerRadius>,
        separator_position: (bool, bool),
    ) -> Self {
        Self {
            selected,
            text: text.into(),
            selected_position,
            hover_position,
            x_size,
            y_size,
            rounding,
            separator_position,
        }
    }
}

impl Widget for AnimatedLabel {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            selected,
            text,
            selected_position,
            hover_position,
            x_size,
            y_size,
            rounding,
            separator_position,
        } = self;

        ui.spacing_mut().item_spacing.x = 0.0;

        let (separator_left, separator_right) = separator_position;
        let button_padding = ui.spacing().button_padding;

        let desired_size = Vec2::new(x_size, y_size);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        let separator_y_1 = rect.min.y - 5.0;
        let separator_y_2 = rect.max.y + 5.0;

        if separator_left {
            let painter = ui.painter();
            let fixed_line_x = painter.round_to_pixel_center(rect.left());
            let color = ui.visuals().widgets.noninteractive.fg_stroke.color;
            let stroke = Stroke::new(1.0, color);
            painter.vline(fixed_line_x, separator_y_1..=separator_y_2, stroke);
        }

        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::SelectableLabel,
                ui.is_enabled(),
                selected,
                text.text(),
            )
        });

        if ui.is_rect_visible(response.rect) {
            let visuals = ui.style().interact_selectable(&response, selected);
            let rounding = rounding.unwrap_or(visuals.corner_radius);

            let text_galley = ui.painter().layout_no_wrap(
                text.text().to_string(),
                TextStyle::Button.resolve(ui.style()),
                visuals.text_color(),
            );

            let text_pos = ui
                .layout()
                .align_size_within_rect(text_galley.size(), rect.shrink2(button_padding))
                .min;

            let mut background_rect = rect.expand(visuals.expansion);

            // Maintain a consistent y size
            if !separator_left && !separator_right {
                let y_difference = background_rect.height();
                let remaining = y_size - y_difference;
                background_rect.min.y -= remaining / 2.0;
                background_rect.max.y += remaining / 2.0;
            } else {
                background_rect.min.y = separator_y_1;
                background_rect.max.y = separator_y_2;
            }

            let half_x_size = x_size / 2.0;

            if selected {
                let x_selected =
                    ui.ctx()
                        .animate_value_with_time(selected_position, rect.center().x, 0.5);
                background_rect.min.x = x_selected - half_x_size;
                background_rect.max.x = x_selected + half_x_size;

                ui.painter().rect(
                    background_rect,
                    rounding,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                    StrokeKind::Inside,
                );
            }

            if response.highlighted() || response.has_focus() || response.hovered() {
                let x_hover =
                    ui.ctx()
                        .animate_value_with_time(hover_position, rect.center().x, 0.5);
                background_rect.min.x = x_hover - half_x_size + 0.5;
                background_rect.max.x = x_hover + half_x_size - 0.5;

                ui.painter().rect(
                    background_rect,
                    rounding,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                    StrokeKind::Inside,
                );
            }

            ui.painter()
                .clone()
                .with_layer_id(LayerId::new(Order::Background, Id::new("text_layer")))
                .galley(text_pos, text_galley, visuals.text_color());

            if separator_right {
                let painter = ui.painter();
                let fixed_line_x = painter.round_to_pixel_center(rect.right());
                let color = ui.visuals().widgets.noninteractive.fg_stroke.color;
                let stroke = Stroke::new(1.0, color);
                painter.vline(fixed_line_x, separator_y_1..=separator_y_2, stroke);
            }
        }

        response
    }
}
