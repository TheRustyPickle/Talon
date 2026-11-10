use eframe::egui::{
    Color32, CornerRadius, CursorIcon, Rect, Response, Role, Sense, StrokeKind, TextStyle, Ui,
    Vec2, WidgetInfo, pos2, vec2,
};
use std::fmt::Display;

/// Seconds the selection indicator takes to slide to a new tab
const SLIDE_TIME: f64 = 0.22;
/// Seconds for the hover highlight and the text color to fade
const FADE_TIME: f32 = 0.15;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum TabStyle {
    /// Plain text with a sliding accent pill behind the selected tab
    #[default]
    Pill,
    /// Items inside a rounded track with a sliding pill behind the selected one
    Segmented,
}

/// Animated row of mutually exclusive tabs.
///
/// Draws the whole strip as a single widget so the selection indicator can slide
/// between items. Use [`Tabs::show`] which returns the clicked item, if any.
pub struct Tabs<T> {
    id_salt: &'static str,
    items: Vec<T>,
    selected: usize,
    style: TabStyle,
    min_item_width: f32,
    equal_width: bool,
    centered: bool,
    hover_text: Option<HoverText<T>>,
}

type HoverText<T> = Box<dyn Fn(&T) -> String>;

/// Where the indicator was drawn last frame and where it is coming from
#[derive(Clone, Copy)]
struct SlideState {
    target: usize,
    from: (f32, f32),
    last: (f32, f32),
    started_at: f64,
}

impl<T: Display + PartialEq> Tabs<T> {
    pub fn new(id_salt: &'static str, items: impl IntoIterator<Item = T>, selected: &T) -> Self {
        let items: Vec<T> = items.into_iter().collect();
        let selected = items.iter().position(|item| item == selected).unwrap_or(0);

        Self {
            id_salt,
            items,
            selected,
            style: TabStyle::default(),
            min_item_width: 0.0,
            equal_width: false,
            centered: false,
            hover_text: None,
        }
    }

    pub fn style(mut self, style: TabStyle) -> Self {
        self.style = style;
        self
    }

    pub fn min_item_width(mut self, width: f32) -> Self {
        self.min_item_width = width;
        self
    }

    /// Give every item the width of the widest one
    pub fn equal_width(mut self, equal: bool) -> Self {
        self.equal_width = equal;
        self
    }

    /// Center the strip in the row it is placed in. It never moves left of where it
    /// would start otherwise, so widgets placed before it are not overlapped
    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    /// Tooltip shown when hovering an item
    pub fn hover_text(mut self, hover_text: impl Fn(&T) -> String + 'static) -> Self {
        self.hover_text = Some(Box::new(hover_text));
        self
    }

    /// Draws the tabs and returns the item that was clicked this frame
    pub fn show(self, ui: &mut Ui) -> Option<T> {
        let Self {
            id_salt,
            mut items,
            selected,
            style,
            min_item_width,
            equal_width,
            centered,
            hover_text,
        } = self;

        if items.is_empty() {
            return None;
        }

        let id = ui.make_persistent_id(id_salt);
        let pad_x = match style {
            TabStyle::Pill => 14.0,
            TabStyle::Segmented => 10.0,
        };
        let height = ui.spacing().interact_size.y + 4.0;
        let font = TextStyle::Button.resolve(ui.style());

        // Text color is overridden at paint time so it can fade between states
        let galleys: Vec<_> = items
            .iter()
            .map(|item| {
                ui.painter()
                    .layout_no_wrap(item.to_string(), font.clone(), Color32::PLACEHOLDER)
            })
            .collect();

        let widest = galleys
            .iter()
            .map(|galley| galley.size().x)
            .fold(0.0, f32::max);

        let widths: Vec<f32> = galleys
            .iter()
            .map(|galley| {
                let text_width = if equal_width { widest } else { galley.size().x };
                (text_width + pad_x * 2.0).max(min_item_width)
            })
            .collect();

        // The segmented track has a small inset so the pill does not touch the border
        let (inset_x, inset_y) = match style {
            TabStyle::Segmented => (3.0, 3.0),
            TabStyle::Pill => (0.0, 2.0),
        };
        let total_width = widths.iter().sum::<f32>() + inset_x * 2.0;

        let (mut strip, _) = ui.allocate_exact_size(vec2(total_width, height), Sense::hover());

        if centered {
            let row = ui.max_rect();
            let target_left = row.center().x - total_width / 2.0;
            let max_shift = (row.right() - strip.right()).max(0.0);
            let shift = (target_left - strip.left()).clamp(0.0, max_shift);
            strip = strip.translate(vec2(shift, 0.0));
        }

        let mut x = strip.left() + inset_x;
        let rects: Vec<Rect> = widths
            .iter()
            .map(|width| {
                let rect = Rect::from_min_size(
                    pos2(x, strip.top() + inset_y),
                    Vec2::new(*width, height - inset_y * 2.0),
                );
                x += width;
                rect
            })
            .collect();

        // Interaction first, painting afterwards, since the indicator is drawn under the text
        let mut clicked = None;
        let mut responses: Vec<Response> = Vec::with_capacity(items.len());
        for (i, rect) in rects.iter().enumerate() {
            let response = ui
                .interact(*rect, id.with(("item", i)), Sense::click())
                .on_hover_cursor(CursorIcon::PointingHand);

            let response = match &hover_text {
                Some(hover_text) => response.on_hover_text(hover_text(&items[i])),
                None => response,
            };

            let label = items[i].to_string();
            response.widget_info(|| {
                WidgetInfo::selected(Role::Tab, ui.is_enabled(), i == selected, &label)
            });

            if response.clicked() {
                clicked = Some(i);
            }
            responses.push(response);
        }

        if !ui.is_rect_visible(strip) {
            return clicked.map(|i| items.swap_remove(i));
        }

        let visuals = ui.visuals().clone();
        let accent = visuals.selection.bg_fill;
        let ctx = ui.ctx().clone();

        let (track_radius, pill_radius) = (CornerRadius::same(7), CornerRadius::same(5));

        match style {
            // No track or lines, the panel around the tabs already has its own border
            TabStyle::Pill => {}
            TabStyle::Segmented => {
                ui.painter()
                    .rect_filled(strip, track_radius, visuals.extreme_bg_color);
                ui.painter().rect_stroke(
                    strip,
                    track_radius,
                    visuals.widgets.noninteractive.bg_stroke,
                    StrokeKind::Inside,
                );
            }
        }

        // Hover highlight fades in and out on each item independently
        for (i, response) in responses.iter().enumerate() {
            let hover_t = ctx.animate_bool_with_time(
                id.with(("hover", i)),
                response.hovered() && i != selected,
                FADE_TIME,
            );
            if hover_t > 0.0 {
                let color = visuals.widgets.hovered.weak_bg_fill.gamma_multiply(hover_t);
                ui.painter().rect_filled(rects[i], pill_radius, color);
            }
        }

        // Slide the indicator from where it was last drawn to the selected item
        let now = ctx.input(|i| i.time);
        let target = rects[selected];
        // Relative to the strip so the indicator does not slide when the strip itself moves,
        // for example when the window is resized by switching tabs
        let origin = strip.left();
        let target_x = (target.left() - origin, target.right() - origin);

        let mut state = ctx
            .data_mut(|d| d.get_temp::<SlideState>(id))
            .unwrap_or(SlideState {
                target: selected,
                from: target_x,
                last: target_x,
                started_at: now - SLIDE_TIME,
            });

        if state.target != selected {
            state.from = state.last;
            state.target = selected;
            state.started_at = now;
        }

        let progress = (((now - state.started_at) / SLIDE_TIME) as f32).clamp(0.0, 1.0);
        let eased = ease_out_cubic(progress);
        let left = lerp(state.from.0, target_x.0, eased);
        let right = lerp(state.from.1, target_x.1, eased);
        state.last = (left, right);
        let (left, right) = (left + origin, right + origin);

        if progress < 1.0 {
            ctx.request_repaint();
        }
        ctx.data_mut(|d| d.insert_temp(id, state));

        let pill = Rect::from_min_max(pos2(left, target.top()), pos2(right, target.bottom()));
        ui.painter().rect_filled(pill, pill_radius, accent);

        // Text fades between its idle, hovered and selected colors
        let idle = visuals.widgets.inactive.fg_stroke.color;
        let hovered = visuals.widgets.hovered.fg_stroke.color;
        let selected_color = visuals.selection.stroke.color;

        for (i, (galley, rect)) in galleys.into_iter().zip(&rects).enumerate() {
            let select_t =
                ctx.animate_bool_with_time(id.with(("select", i)), i == selected, FADE_TIME);
            let hover_t = ctx.animate_bool_with_time(
                id.with(("text_hover", i)),
                responses[i].hovered(),
                FADE_TIME,
            );

            let color = idle
                .lerp_to_gamma(hovered, hover_t)
                .lerp_to_gamma(selected_color, select_t);

            let text_pos = rect.center() - galley.size() / 2.0;
            ui.painter()
                .galley_with_override_text_color(text_pos, galley, color);
        }

        clicked.map(|i| items.swap_remove(i))
    }
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
