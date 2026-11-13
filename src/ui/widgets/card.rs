use eframe::egui::{CornerRadius, Frame, InnerResponse, Margin, RichText, Stroke, Ui};

pub struct Card {
    title: Option<String>,
}

impl Card {
    pub fn new() -> Self {
        Self { title: None }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let visuals = ui.visuals();
        let frame = Frame::new()
            .fill(visuals.faint_bg_color)
            .stroke(Stroke::new(
                1.0,
                visuals.widgets.noninteractive.bg_stroke.color,
            ))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(12, 10));

        frame.show(ui, |ui| {
            // Frames only take the width they need, fill the whole row instead
            ui.set_width(ui.available_width());

            // Inputs sit on the card color instead of the panel color and lose their
            // contrast, so give idle widgets a visible outline
            let outline = ui.visuals().widgets.noninteractive.bg_stroke.color;
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, outline);

            if let Some(title) = self.title {
                ui.label(RichText::new(title).size(13.0).strong());
                ui.add_space(6.0);
            }
            add_contents(ui)
        })
    }
}
