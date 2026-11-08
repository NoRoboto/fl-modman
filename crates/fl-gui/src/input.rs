//! Gamepad and keyboard input mapped to the app's actions (spike 0(a)): the D-pad and the left
//! stick move with auto-repeat, the face buttons act, the bumpers switch tabs.

use std::time::{Duration, Instant};

use eframe::egui::{Context, Event, Key};
use gilrs::{Axis, Button, EventType, Gilrs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    /// A: activate the focused item.
    Accept,
    /// B: back or cancel.
    Back,
    /// X: toggle the focused mod.
    Toggle,
    /// Y: move mode in the mod list.
    Move,
    PrevTab,
    NextTab,
}
use Action::*;

pub const DEAD_ZONE: f32 = 0.5;
pub const FIRST_REPEAT: Duration = Duration::from_millis(400);
pub const NEXT_REPEAT: Duration = Duration::from_millis(150);

/// The direction of a held D-pad (`[up, down, left, right]`) or of the left stick (y up is
/// positive); the D-pad wins, then the stick's larger axis past the dead zone.
pub fn direction(dpad: [bool; 4], x: f32, y: f32) -> Option<Action> {
    if let Some(i) = dpad.iter().position(|d| *d) {
        return Some([Up, Down, Left, Right][i]);
    }
    let vertical = y.abs() >= x.abs();
    match () {
        _ if vertical && y > DEAD_ZONE => Some(Up),
        _ if vertical && y < -DEAD_ZONE => Some(Down),
        _ if x > DEAD_ZONE => Some(Right),
        _ if x < -DEAD_ZONE => Some(Left),
        _ => None,
    }
}

/// Auto-repeat of a held direction: fires on press, again after [`FIRST_REPEAT`], then every
/// [`NEXT_REPEAT`] until released or changed.
#[derive(Default)]
pub struct Repeat {
    held: Option<Action>,
    next: Option<Instant>,
}

impl Repeat {
    pub fn update(&mut self, dir: Option<Action>, now: Instant) -> Option<Action> {
        if dir != self.held {
            (self.held, self.next) = (dir, Some(now + FIRST_REPEAT));
            return dir;
        }
        if dir.is_some() && self.next.is_some_and(|t| now >= t) {
            self.next = Some(now + NEXT_REPEAT);
            return dir;
        }
        None
    }
}

pub fn button_action(b: Button) -> Option<Action> {
    Some(match b {
        Button::South => Accept,
        Button::East => Back,
        Button::West => Toggle,
        Button::North => Move,
        Button::LeftTrigger => PrevTab,
        Button::RightTrigger => NextTab,
        _ => return None,
    })
}

/// Desktop keys: arrows, Enter (A), Escape/Backspace (B), Space (X), M (Y), Q/E or PageUp/Down.
pub fn key_action(k: Key) -> Option<Action> {
    Some(match k {
        Key::ArrowUp => Up,
        Key::ArrowDown => Down,
        Key::ArrowLeft => Left,
        Key::ArrowRight => Right,
        Key::Enter => Accept,
        Key::Escape | Key::Backspace => Back,
        Key::Space => Toggle,
        Key::M => Move,
        Key::Q | Key::PageUp => PrevTab,
        Key::E | Key::PageDown => NextTab,
        _ => return None,
    })
}

pub struct Input {
    pub gilrs: Option<Gilrs>,
    repeat: Repeat,
}

impl Default for Input {
    fn default() -> Self {
        let (gilrs, repeat) = (Gilrs::new().ok(), Repeat::default());
        Input { gilrs, repeat }
    }
}

impl Input {
    /// This frame's actions, keyboard first.
    pub fn poll(&mut self, ctx: &Context) -> Vec<Action> {
        let mut out: Vec<Action> = ctx.input(|i| {
            let keys = i.events.iter().filter_map(|e| match e {
                Event::Key {
                    key, pressed: true, ..
                } => key_action(*key),
                _ => None,
            });
            keys.collect()
        });
        let Some(g) = self.gilrs.as_mut() else {
            return out;
        };
        while let Some(ev) = g.next_event() {
            if let EventType::ButtonPressed(b, _) = ev.event {
                out.extend(button_action(b));
            }
        }
        let mut dir = None;
        for (_, pad) in g.gamepads() {
            use Button::{DPadDown, DPadLeft, DPadRight, DPadUp};
            let dpad = [DPadUp, DPadDown, DPadLeft, DPadRight];
            let (x, y) = (pad.value(Axis::LeftStickX), pad.value(Axis::LeftStickY));
            dir = dir.or(direction(dpad.map(|b| pad.is_pressed(b)), x, y));
        }
        out.extend(self.repeat.update(dir, Instant::now()));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_needs_the_dead_zone_and_dpad_wins() {
        assert_eq!(direction([false; 4], 0.0, 0.49), None);
        assert_eq!(direction([false; 4], 0.0, 0.6), Some(Up));
        assert_eq!(direction([false; 4], 0.2, -0.9), Some(Down));
        assert_eq!(direction([false; 4], -0.8, 0.6), Some(Left));
        assert_eq!(
            direction([false, false, false, true], 0.0, 0.9),
            Some(Right)
        );
    }

    #[test]
    fn repeat_fires_on_press_then_after_400_then_every_150_ms() {
        let (mut r, t0) = (Repeat::default(), Instant::now());
        let at = |ms| t0 + Duration::from_millis(ms);
        assert_eq!(r.update(Some(Down), at(0)), Some(Down));
        assert_eq!(r.update(Some(Down), at(399)), None);
        assert_eq!(r.update(Some(Down), at(400)), Some(Down));
        assert_eq!(r.update(Some(Down), at(549)), None);
        assert_eq!(r.update(Some(Down), at(550)), Some(Down));
        assert_eq!(r.update(None, at(560)), None);
        assert_eq!(r.update(None, at(2000)), None);
        // A new direction fires at once and restarts the first delay.
        assert_eq!(r.update(Some(Up), at(2001)), Some(Up));
        assert_eq!(r.update(Some(Left), at(2002)), Some(Left));
        assert_eq!(r.update(Some(Left), at(2300)), None);
    }

    #[test]
    fn buttons_and_keys_map_to_the_same_actions() {
        assert_eq!(button_action(Button::South), key_action(Key::Enter));
        assert_eq!(button_action(Button::East), Some(Back));
        assert_eq!(button_action(Button::North), Some(Move));
        assert_eq!(button_action(Button::RightTrigger), Some(NextTab));
        assert_eq!(button_action(Button::Start), None);
    }
}
