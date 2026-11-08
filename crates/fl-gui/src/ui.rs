//! The design system: tokens (spacing, text sizes, colours, one radius, the focus ring) and the
//! only components the screens draw with. Handheld-first: a 7" 1080p screen at arm's length,
//! driven by a gamepad; the hint bar lists the real controls, so no screen explains them.

use eframe::egui::{
    Align, Button, Color32, Context, FontId, Frame, Label, Layout, Margin, Response, RichText,
    ScrollArea, Sense, Stroke, TextStyle, Ui, Visuals, vec2,
};

use crate::input::Action;
use crate::nav::Nav;

pub const S1: f32 = 8.0;
pub const S2: f32 = 16.0;
pub const S3: f32 = 24.0;
pub const S4: f32 = 32.0;

pub const TITLE: f32 = 26.0;
pub const BODY: f32 = 20.0;
pub const HINT: f32 = 16.0;

pub const BACKGROUND: Color32 = Color32::from_rgb(16, 18, 22);
pub const SURFACE: Color32 = Color32::from_rgb(33, 37, 45);
pub const ACCENT: Color32 = Color32::from_rgb(90, 180, 255);
pub const WARNING: Color32 = Color32::from_rgb(255, 196, 64);
pub const MUTED: Color32 = Color32::from_rgb(150, 158, 172);
/// Body text (near white).
pub const TEXT: Color32 = Color32::from_rgb(236, 239, 244);

pub const RADIUS: u8 = 8;
/// The focus ring: the only focus indicator.
pub const FOCUS: f32 = 3.0;
/// Height reserved below a screen's scrolled body for its action bar.
const BAR: f32 = 48.0 + S2;

/// Button names for the hint bar: the bundled fonts have no circled letters (Ⓐ), so text.
pub const A: &str = "[A]";
pub const B: &str = "[B]";
pub const Y: &str = "[Y]";

/// Installs the tokens as the egui style. Hover and press look like rest: on a handheld the
/// focus ring is the only state a widget shows.
pub fn install(ctx: &Context) {
    ctx.global_style_mut(|s| {
        let mut v = Visuals::dark();
        (v.panel_fill, v.weak_text_color) = (BACKGROUND, Some(MUTED));
        (v.selection.bg_fill, v.selection.stroke) = (SURFACE, Stroke::new(1.0, ACCENT));
        let w = &mut v.widgets;
        (
            w.noninteractive.fg_stroke.color,
            w.noninteractive.bg_stroke.color,
        ) = (TEXT, SURFACE);
        (w.inactive.bg_fill, w.inactive.weak_bg_fill) = (SURFACE, SURFACE);
        (w.inactive.fg_stroke.color, w.inactive.corner_radius) = (TEXT, RADIUS.into());
        (w.hovered, w.active) = (w.inactive, w.inactive);
        s.visuals = v;
        (s.spacing.item_spacing, s.spacing.button_padding) = (vec2(S1, S1), vec2(S2, S1));
        s.spacing.interact_size.y = 44.0;
        use TextStyle::*;
        let sizes = [
            (Heading, TITLE),
            (Body, BODY),
            (Button, BODY),
            (Monospace, BODY),
        ];
        for (style, size) in sizes.into_iter().chain([(Small, HINT)]) {
            let family = style.resolve(s).family;
            s.text_styles.insert(style, FontId::new(size, family));
        }
    });
}

/// A screen's scrolled body: fills the height left above the action bar.
pub fn body<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let height = (ui.available_height() - BAR).max(BAR);
    let area = ScrollArea::vertical()
        .max_height(height)
        .auto_shrink([false; 2]);
    area.show(ui, |ui| {
        ui.set_min_width(ui.available_width() - S1);
        add(ui)
    })
    .inner
}

/// A titled box grouping options.
pub fn card<R>(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    let frame = Frame::new().fill(SURFACE).corner_radius(RADIUS);
    let inner = frame.inner_margin(S2).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        if !title.is_empty() {
            ui.label(RichText::new(title).strong().color(MUTED));
        }
        add(ui)
    });
    ui.add_space(S2);
    inner.inner
}

/// A focusable list row: an optional lead (a [`toggle`]), the label on the left, the meta on
/// the right and one [`badge`] line each below.
#[derive(Default)]
pub struct Row<'a> {
    pub label: String,
    pub meta: String,
    pub badges: &'a [String],
    /// Filled with the accent tint (the mod being moved, the picked generation).
    pub hot: bool,
    /// Drawn in the muted colour (not actionable).
    pub muted: bool,
}

impl Row<'_> {
    pub fn new(label: impl Into<String>, meta: impl Into<String>) -> Self {
        let (label, meta) = (label.into(), meta.into());
        Row {
            label,
            meta,
            ..Default::default()
        }
    }

    /// Draws the row; the response covers all of it and senses clicks.
    pub fn show(self, ui: &mut Ui, lead: impl FnOnce(&mut Ui)) -> Response {
        let fill = [Color32::TRANSPARENT, ACCENT.gamma_multiply(0.3)][usize::from(self.hot)];
        let color = [TEXT, MUTED][usize::from(self.muted)];
        let frame = Frame::new().fill(fill).corner_radius(RADIUS);
        let frame = frame.inner_margin(Margin::symmetric(S2 as i8, S1 as i8));
        // Vertical inside, also when the row sits in a horizontal layout (the drag handle).
        let r = frame.show(ui, |ui| {
            ui.vertical(|ui| {
                let size = vec2(ui.available_width(), ui.spacing().interact_size.y);
                ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
                    if !self.meta.is_empty() {
                        ui.label(RichText::new(&self.meta).color(MUTED));
                    }
                    let size = vec2(ui.available_width(), size.y);
                    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                        lead(ui);
                        ui.add(Label::new(RichText::new(&self.label).color(color)).wrap());
                    });
                });
                for b in self.badges {
                    badge(ui, b);
                }
            })
        });
        r.response.interact(Sense::click())
    }

    /// Draws the row and registers it with the focus layer; true when activated.
    pub fn nav(self, ui: &mut Ui, nav: &mut Nav) -> bool {
        let r = self.show(ui, |_| {});
        nav.item(ui, &r)
    }
}

/// The ON/OFF pill of a mod (the row toggles it).
pub fn toggle(ui: &mut Ui, on: bool) {
    let (text, fill, color) = [
        ("OFF", Color32::TRANSPARENT, MUTED),
        ("ON", ACCENT, BACKGROUND),
    ][usize::from(on)];
    let frame = Frame::new()
        .fill(fill)
        .stroke(Stroke::new(2.0, [MUTED, ACCENT][usize::from(on)]));
    frame.corner_radius(RADIUS).show(ui, |ui| {
        let text = RichText::new(text).size(HINT).strong().color(color);
        ui.add_sized([52.0, 26.0], Label::new(text));
    });
}

/// `label   ◀ value ▶` with `*` while the value is pending; +1 on A, Right or a tap, -1 on
/// Left, else 0.
pub fn stepper(ui: &mut Ui, nav: &mut Nav, label: &str, value: &str, pending: bool) -> i8 {
    let mark = if pending { " *" } else { "   " };
    let r = Row::new(label, format!("◀  {value}  ▶{mark}")).show(ui, |_| {});
    if nav.item(ui, &r) || nav.on_focused(Action::Right) {
        return 1;
    }
    -i8::from(nav.on_focused(Action::Left))
}

/// The action bar, right-aligned, of large buttons; the last is the screen's primary action
/// (accent-filled; it takes the focus on a screen that opens with nothing else to focus). The
/// index of the one pressed.
pub fn actions(ui: &mut Ui, nav: &mut Nav, labels: &[&str]) -> Option<usize> {
    let font = TextStyle::Button.resolve(ui.style());
    let width = |l: &&str| {
        let g = ui
            .painter()
            .layout_no_wrap(l.to_string(), font.clone(), TEXT);
        (g.size().x + 2.0 * S2).max(180.0) + S1
    };
    let (total, last) = (labels.iter().map(width).sum::<f32>(), labels.len() - 1);
    nav.autofocus(nav.next_index() + last);
    ui.add_space(S1);
    let row = ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - total).max(0.0));
        let hits = labels.iter().enumerate().map(|(i, l)| {
            let (fill, color) = [(SURFACE, TEXT), (ACCENT, BACKGROUND)][usize::from(i == last)];
            let b = Button::new(RichText::new(*l).strong().color(color)).fill(fill);
            let r = ui.add(b.corner_radius(RADIUS).min_size(vec2(180.0, 48.0)));
            nav.item(ui, &r)
        });
        hits.collect::<Vec<_>>().iter().position(|h| *h)
    });
    row.inner
}

/// A ⚠ warning line.
pub fn badge(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(format!("⚠ {text}")).size(HINT).color(WARNING));
}

/// One control in the hint bar: the button and what it does here.
pub type Hint = (&'static str, &'static str);

/// The text of the hint bar.
pub fn hint_text(hints: &[Hint]) -> String {
    let parts: Vec<String> = hints.iter().map(|(k, v)| format!("{k} {v}")).collect();
    parts.join("    ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_text_pairs_buttons_with_actions() {
        assert_eq!(
            hint_text(&[(A, "Toggle"), ("L1/R1", "Tabs")]),
            "[A] Toggle    L1/R1 Tabs"
        );
        assert_eq!(hint_text(&[]), "");
    }
}
