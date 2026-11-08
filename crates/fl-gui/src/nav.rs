//! The focus layer shared by gamepad and keyboard. Every focusable widget registers in drawing
//! order each frame; A activates the focused one, and the directions left over once the screen
//! has drawn (a screen may take them first, e.g. in move mode) move the focus.

use eframe::egui::{Response, Stroke, StrokeKind, Ui};

use crate::input::Action;
use crate::ui::{ACCENT, FOCUS, RADIUS};

#[derive(Default)]
pub struct Nav {
    pub focus: usize,
    count: usize,
    actions: Vec<Action>,
    scroll: bool,
    last_focused: bool,
    /// The screen was just entered: [`Nav::autofocus`] may still move the focus.
    fresh: bool,
}

impl Nav {
    pub fn begin(&mut self, actions: Vec<Action>) {
        (self.actions, self.count) = (actions, 0);
    }

    /// Takes one pending `a`, if any.
    pub fn take(&mut self, a: Action) -> bool {
        let i = self.actions.iter().position(|x| *x == a);
        i.map(|i| self.actions.remove(i)).is_some()
    }

    pub fn set_focus(&mut self, i: usize) {
        (self.focus, self.scroll) = (i, true);
    }

    pub fn reset(&mut self) {
        self.set_focus(0);
        self.fresh = true;
    }

    /// Once per entered screen: focuses item `i` if nothing was registered before it.
    pub fn autofocus(&mut self, i: usize) {
        if std::mem::take(&mut self.fresh) && self.count == 0 {
            self.set_focus(i);
        }
    }

    /// The index the next registered item gets.
    pub fn next_index(&self) -> usize {
        self.count
    }

    /// Registers an item; returns true if it was clicked, or is focused and A was pressed.
    pub fn register(&mut self, clicked: bool) -> bool {
        let i = self.count;
        self.count += 1;
        if clicked {
            self.focus = i;
        }
        self.last_focused = i == self.focus;
        clicked || self.last_focused && self.take(Action::Accept)
    }

    /// Takes `a` if the last registered item has the focus.
    pub fn on_focused(&mut self, a: Action) -> bool {
        self.last_focused && self.take(a)
    }

    /// Registers a widget, outlining and scrolling to it when focused.
    pub fn item(&mut self, ui: &Ui, r: &Response) -> bool {
        let activated = self.register(r.clicked());
        if self.last_focused {
            let (stroke, rect) = (Stroke::new(FOCUS, ACCENT), r.rect.expand(FOCUS));
            ui.painter()
                .rect_stroke(rect, RADIUS, stroke, StrokeKind::Outside);
            if self.scroll {
                r.scroll_to_me(None);
            }
        }
        activated
    }

    /// Moves the focus with the directions nobody took; Left/Right step like Up/Down.
    pub fn end(&mut self) {
        self.scroll = false;
        for a in std::mem::take(&mut self.actions) {
            match a {
                Action::Up | Action::Left if self.focus > 0 => self.set_focus(self.focus - 1),
                Action::Down | Action::Right if self.focus + 1 < self.count => {
                    self.set_focus(self.focus + 1)
                }
                _ => {}
            }
        }
        self.focus = self.focus.min(self.count.saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Action::*;

    fn frame(nav: &mut Nav, actions: Vec<Action>, items: usize) -> Vec<bool> {
        nav.begin(actions);
        let hits = (0..items).map(|_| nav.register(false)).collect();
        nav.end();
        hits
    }

    #[test]
    fn directions_move_the_focus_within_the_items() {
        let mut nav = Nav::default();
        frame(&mut nav, vec![Down, Down, Right], 3);
        assert_eq!(nav.focus, 2);
        frame(&mut nav, vec![Down], 3);
        assert_eq!(nav.focus, 2);
        frame(&mut nav, vec![Up, Left, Up, Up], 3);
        assert_eq!(nav.focus, 0);
        // A screen with fewer items clamps the focus.
        nav.set_focus(5);
        frame(&mut nav, vec![], 2);
        assert_eq!(nav.focus, 1);
    }

    #[test]
    fn accept_activates_only_the_focused_item_and_clicks_take_the_focus() {
        let mut nav = Nav::default();
        nav.set_focus(1);
        assert_eq!(frame(&mut nav, vec![Accept], 3), [false, true, false]);
        assert_eq!(frame(&mut nav, vec![], 3), [false; 3]);
        nav.begin(vec![]);
        assert!(!nav.register(false));
        assert!(nav.register(true));
        nav.end();
        assert_eq!(nav.focus, 1);
    }

    #[test]
    fn a_screen_can_take_directions_before_the_focus_moves() {
        let mut nav = Nav::default();
        nav.begin(vec![Down, Toggle]);
        assert!(nav.take(Down) && !nav.take(Down));
        nav.register(false);
        assert!(nav.on_focused(Toggle));
        nav.register(false);
        nav.end();
        assert_eq!(nav.focus, 0);
    }

    #[test]
    fn autofocus_applies_once_and_only_on_a_screen_opening_with_buttons() {
        let mut nav = Nav::default();
        nav.reset();
        nav.begin(vec![]);
        nav.autofocus(1);
        nav.register(false);
        nav.register(false);
        nav.end();
        assert_eq!(nav.focus, 1);
        frame(&mut nav, vec![Up], 2);
        nav.begin(vec![]);
        nav.autofocus(1);
        assert_eq!(nav.focus, 0);
        // Items before the action bar (a list) keep the focus on the first one.
        nav.reset();
        nav.register(false);
        nav.autofocus(1);
        assert_eq!(nav.focus, 0);
    }
}
